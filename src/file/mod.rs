//! Support for log4rs configuration from files.
//!
//! Multiple file formats are supported, each requiring a Cargo feature to be
//! enabled. YAML support requires the `yaml` feature, JSON support requires
//! the `JSON` feature, and TOML support requires the `toml` feature.
//!
//! # Syntax
//!
//! All file formats currently share the same structure. The example below is
//! of the YAML format, but the JSON and TOML formats consist of the same
//! structure.
//!
//! ```yaml
//! # If set, log4rs will scan the file at the specified rate for changes and
//! # automatically reconfigure the logger. The input string is parsed by the
//! # humantime crate.
//! refresh_rate: 30 seconds
//!
//! # The "appenders" map contains the set of appenders, indexed by their names.
//! appenders:
//!
//!   foo:
//!
//!     # All appenders must specify a "kind", which will be used to look up the
//!     # logic to construct the appender in the `Deserializers` passed to the
//!     # deserialization function.
//!     kind: console
//!
//!     # Filters attached to an appender are specified inside the "filters"
//!     # array.
//!     filters:
//!
//!       -
//!         # Like appenders, filters are identified by their "kind".
//!         kind: threshold
//!
//!         # The remainder of the configuration is passed along to the
//!         # filter's builder, and will vary based on the kind of filter.
//!         level: error
//!
//!     # The remainder of the configuration is passed along to the appender's
//!     # builder, and will vary based on the kind of appender.
//!     # Appenders will commonly be associated with an encoder.
//!     encoder:
//!
//!       # Like appenders, encoders are identified by their "kind". If no kind
//!       # is specified, it will default to "pattern".
//!       kind: pattern
//!
//!       # The remainder of the configuration is passed along to the
//!       # encoder's builder, and will vary based on the kind of encoder.
//!       pattern: "{d} [{t}] {m}{n}"
//!
//! # The root logger is configured by the "root" map. It is optional.
//! root:
//!
//!   # The maximum log level for the root logger.
//!   level: warn
//!
//!   # The list of appenders attached to the root logger. Defaults to an empty
//!   # list if not specified.
//!   appenders:
//!     - foo
//!
//! # The "loggers" map contains the set of configured loggers, indexed by their
//! # names.
//! loggers:
//!
//!   foo::bar::baz:
//!
//!     # The maximum log level. Defaults to the level of the logger's parent if
//!     # not specified.
//!     level: trace
//!
//!     # The list of appenders attached to the logger. Defaults to an empty
//!     # list if not specified.
//!     appenders:
//!       - foo
//!
//!     # The additivity of the logger. If true, appenders attached to the
//!     # logger's parent will also be attached to this logger. Defauts to true
//!     # if not specified.
//!     additive: false
//! ```

use log::LogLevelFilter;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::error;
use std::fmt;
use std::time::Duration;
use typemap::{Key, ShareMap};
use serde_value::Value;
use serde::Deserialize as SerdeDeserialize;

use append::file::FileAppenderDeserializer;
use append::console::ConsoleAppenderDeserializer;
use append::syslog::SyslogAppenderDeserializer;
use filter::Filter;
use filter::threshold::ThresholdFilterDeserializer;
use config;
use encode::pattern::PatternEncoderDeserializer;
use PrivateConfigErrorsExt;

pub mod raw;

struct KeyAdaptor<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized + Any> Key for KeyAdaptor<T> {
    type Value = HashMap<String, Box<Deserialize<Trait = T>>>;
}

/// A trait for objects that can deserialize log4rs components out of a config.
pub trait Deserialize: Send + Sync + 'static {
    /// The trait that this builder will create.
    type Trait: ?Sized;

    /// Create a new trait object based on the provided config.
    fn deserialize(&self,
                   config: Value,
                   deserializers: &Deserializers)
                   -> Result<Box<Self::Trait>, Box<error::Error>>;
}

/// A container of `Deserialize`rs.
pub struct Deserializers(ShareMap);

/// Creates a `Deserializers` with the following mappings:
///
/// * Appenders
///     * "file" -> `FileAppenderDeserializer`
///     * "console" -> `ConsoleAppenderDeserializer`
///     * "syslog" -> `SyslogAppenderDeserializer`
/// * Filters
///     * "threshold" -> `ThresholdFilterDeserializer`
/// * Encoders
///     * "pattern" -> `PatternEncoderDeserializer`
impl Default for Deserializers {
    fn default() -> Deserializers {
        let mut deserializers = Deserializers::new();
        deserializers.insert("file".to_owned(), Box::new(FileAppenderDeserializer));
        deserializers.insert("console".to_owned(), Box::new(ConsoleAppenderDeserializer));
        deserializers.insert("syslog".to_owned(), Box::new(SyslogAppenderDeserializer));
        deserializers.insert("threshold".to_owned(),
                             Box::new(ThresholdFilterDeserializer));
        deserializers.insert("pattern".to_owned(), Box::new(PatternEncoderDeserializer));
        deserializers
    }
}

impl Deserializers {
    /// Creates a new `Deserializers` with no mappings.
    pub fn new() -> Deserializers {
        Deserializers(ShareMap::custom())
    }

    /// Adds a mapping from the specified `kind` to a deserializer.
    pub fn insert<T: ?Sized + Any>(&mut self, kind: String, builder: Box<Deserialize<Trait = T>>) {
        self.0.entry::<KeyAdaptor<T>>().or_insert_with(|| HashMap::new()).insert(kind, builder);
    }

    /// Retrieves the deserializer of the specified `kind`.
    pub fn get<T: ?Sized + Any>(&self, kind: &str) -> Option<&Deserialize<Trait = T>> {
        self.0.get::<KeyAdaptor<T>>().and_then(|m| m.get(kind)).map(|b| &**b)
    }

    /// A utility method that deserializes a value.
    pub fn deserialize<T: ?Sized + Any>(&self,
                                        trait_: &str,
                                        kind: &str,
                                        config: Value)
                                        -> Result<Box<T>, Box<error::Error>> {
        match self.get(kind) {
            Some(b) => b.deserialize(config, self),
            None => Err(format!("no {} builder for kind `{}` registered", trait_, kind).into()),
        }
    }
}

/// An error returned when deserializing a TOML configuration into a log4rs `Config`.
#[derive(Debug)]
pub enum Error {
    /// An error deserializing a component.
    Deserialization(Box<error::Error>),
    /// An error creating the log4rs `Config`.
    Config(config::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Deserialization(ref err) => {
                write!(fmt, "Error deserializing component: {}", err)
            }
            Error::Config(ref err) => write!(fmt, "Error creating config: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "An error encountered when deserializing a configuration file into a log4rs `Config`"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Deserialization(ref err) => Some(&**err),
            Error::Config(ref err) => Some(err),
        }
    }
}

/// Specifies the format of a configuration file.
#[derive(Copy, Clone)]
pub enum Format {
    /// YAML.
    ///
    /// Requires the `yaml` feature.
    #[cfg(feature = "yaml")]
    Yaml,
    /// JSON.
    ///
    /// Requires the `json` feature.
    #[cfg(feature = "json")]
    Json,
    /// TOML.
    ///
    /// Requires the `toml` feature.
    #[cfg(feature = "toml")]
    Toml,
}

/// A deserialized log4rs configuration file.
pub struct Config {
    refresh_rate: Option<Duration>,
    config: config::Config,
    errors: Vec<Error>,
}

impl Config {
    /// Creates a log4rs `Config` from the specified config string and `Deserializers`.
    pub fn parse(config: &str,
                 format: Format,
                 deserializers: &Deserializers)
                 -> Result<Config, Box<error::Error>> {
        let mut errors = vec![];

        let config = try!(parse(format, config));

        let raw::Config { refresh_rate,
                          root: raw_root,
                          appenders: raw_appenders,
                          loggers: raw_loggers,
                          .. } = config;

        let root = match raw_root {
            Some(raw_root) => {
                config::Root::builder()
                    .appenders(raw_root.appenders)
                    .build(raw_root.level)
            }
            None => config::Root::builder().build(LogLevelFilter::Debug),
        };

        let mut config = config::Config::builder();

        for (name, raw::Appender { kind, config: raw_config, filters }) in raw_appenders {
            match deserializers.deserialize("appender", &kind, raw_config) {
                Ok(appender_obj) => {
                    let mut builder = config::Appender::builder();
                    for raw::Filter { kind, config } in filters {
                        match deserializers.deserialize("filter", &kind, config) {
                            Ok(filter) => builder = builder.filter(filter),
                            Err(err) => errors.push(Error::Deserialization(err)),
                        }
                    }
                    config = config.appender(builder.build(name.clone(), appender_obj));
                }
                Err(err) => errors.push(Error::Deserialization(err)),
            }
        }

        for (name, logger) in raw_loggers {
            let raw::Logger { level, appenders, additive, .. } = logger;
            let mut logger = config::Logger::builder().appenders(appenders);
            if let Some(additive) = additive {
                logger = logger.additive(additive);
            }
            config = config.logger(logger.build(name, level));
        }

        let (config, config_errors) = config.build_lossy(root);
        if let Err(config_errors) = config_errors {
            for error in config_errors.unpack() {
                errors.push(Error::Config(error));
            }
        }

        let config = Config {
            refresh_rate: refresh_rate.map(|r| r),
            config: config,
            errors: errors,
        };

        Ok(config)
    }

    /// Returns the requested refresh rate.
    pub fn refresh_rate(&self) -> Option<Duration> {
        self.refresh_rate
    }

    /// Returns the log4rs `Config`.
    pub fn into_config(self) -> config::Config {
        self.config
    }

    /// Returns any nonfatal errors encountered when deserializing the config.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }
}

fn parse(format: Format, _config: &str) -> Result<raw::Config, Box<error::Error>> {
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => ::serde_yaml::from_str(_config).map_err(Into::into),
        #[cfg(feature = "json")]
        Format::Json => ::serde_json::from_str(_config).map_err(Into::into),
        #[cfg(feature = "toml")]
        Format::Toml => {
            let mut parser = ::toml::Parser::new(_config);
            let table = match parser.parse() {
                Some(table) => ::toml::Value::Table(table),
                None => return Err(parser.errors.pop().unwrap().into()),
            };
            raw::Config::deserialize(&mut ::toml::Decoder::new(table)).map_err(Into::into)
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "yaml")]
    fn full_deserialize() {
        let cfg = r#"
refresh_rate: 60 seconds

appenders:
  console:
    kind: console
    filters:
      - kind: threshold
        level: debug
  baz:
    kind: file
    path: /tmp/baz.log
    encoder:
      pattern: "%m"

root:
  appenders:
    - console
  level: info

loggers:
  foo::bar::baz:
    level: warn
    appenders:
      - baz
    additive: false
"#;
        let config = Config::parse(cfg, Format::Yaml, &Deserializers::default()).unwrap();
        assert!(config.errors().is_empty());
    }

    #[test]
    #[cfg(feature = "yaml")]
    fn empty() {
        let config = Config::parse("{}",
                                   Format::Yaml,
                                   &Deserializers::default()).unwrap();
        assert!(config.errors().is_empty());
    }

    #[test]
    #[cfg(feature = "yaml")]
    fn integer_refresh_yaml() {
        let config = Config::parse("refresh_rate: 60",
                                   Format::Yaml,
                                   &Deserializers::default()).unwrap();
        assert!(config.errors().is_empty());
    }

    #[test]
    #[cfg(feature = "json")]
    fn integer_refresh_json() {
        let config = Config::parse(r#"{"refresh_rate": 60}"#,
                                   Format::Json,
                                   &Deserializers::default()).unwrap();
        assert!(config.errors().is_empty());
    }

    #[test]
    #[cfg(feature = "toml")]
    fn integer_refresh_toml() {
        let config = Config::parse("refresh_rate = 60",
                                   Format::Toml,
                                   &Deserializers::default()).unwrap();
        assert!(config.errors().is_empty());
    }
}
