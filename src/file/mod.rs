//! Support for log4rs configuration from files.
//!
//! Multiple file formats are supported, each requiring a Cargo feature to be
//! enabled. YAML support requires the `yaml` feature, JSON support requires
//! the `JSON` feature, and TOML support requires the `toml` feature.
//!
//! # Syntax
//!
//! All file formats currently share the same structure. The example below is
//! of the YAML format, but JSON and TOML should follow.
//!
//! ```yaml
//! # If set, log4rs will scan the file at the specified rate in seconds for
//! # changes and automatically reconfigure the logger.
//! refresh_rate: 30
//!
//! # The "appenders" map contains the set of appenders, indexed by their names.
//! appenders:
//!
//!   foo:
//!
//!     # All appenders must specify a "kind", which will be used to look up the
//!     # logic to construct the appender in the `Builder` passed to the
//!     # deserialization function.
//!     kind: console
//!
//!     # Filters attached to an appender are specified inside the "filters"
//!     # array.
//!     filters:
//!
//!       -
//!         # Like appenders, filters are identified by their "kind".
//!         kind = threshold
//!         # The remainder of the configuration is passed along to the
//!         # filter's builder, and will vary based on the kind of filter.
//!         level = error
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
//!       pattern = "%d [%t] %m"
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
//!     # list if not specified
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
use serde::Deserialize;

use append::Append;
use append::file::FileAppenderDeserializer;
use append::console::ConsoleAppenderDeserializer;
use filter::Filter;
use filter::threshold::ThresholdFilterBuilder;
use config;
use encode::pattern::PatternEncoderBuilder;
use PrivateConfigErrorsExt;

pub mod raw;

struct KeyAdaptor<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized + Any> Key for KeyAdaptor<T> {
    type Value = HashMap<String, Box<Build<Trait = T>>>;
}

/// A trait for objects that can create log4rs components out of a config.
pub trait Build: Send + Sync + 'static {
    /// The trait that this builder will create.
    type Trait: ?Sized;

    /// Create a new trait object based on the provided config.
    fn build(&self,
             config: Value,
             builder: &Builder)
             -> Result<Box<Self::Trait>, Box<error::Error>>;
}

/// A type that can create appenders.
///
/// `Builder` implements `Default`, which returns a `Builder` with the
/// following mappings:
///
/// * Appenders
///     * "file" -> `FileAppenderDeserializer`
///     * "console" -> `ConsoleAppenderDeserializer`
/// * Filters
///     * "threshold" -> `ThresholdFilterBuilder`
/// * Encoders
///     * "pattern" -> `PatternEncoderBuilder`
pub struct Builder {
    builders: ShareMap,
}

impl Default for Builder {
    fn default() -> Builder {
        let mut creator = Builder::new();
        creator.insert("file".to_owned(), Box::new(FileAppenderDeserializer));
        creator.insert("console".to_owned(), Box::new(ConsoleAppenderDeserializer));
        creator.insert("threshold".to_owned(), Box::new(ThresholdFilterBuilder));
        creator.insert("pattern".to_owned(), Box::new(PatternEncoderBuilder));
        creator
    }
}

impl Builder {
    /// Creates a new `Builder` with no mappings.
    pub fn new() -> Builder {
        Builder { builders: ShareMap::custom() }
    }

    /// Adds a mapping from the specified `kind` to a builder.
    pub fn insert<T: ?Sized + Any>(&mut self, kind: String, builder: Box<Build<Trait = T>>) {
        self.builders.entry::<KeyAdaptor<T>>().or_insert(HashMap::new()).insert(kind, builder);
    }

    /// Retrieves the builder of the specified `kind`.
    pub fn get<T: ?Sized + Any>(&self, kind: &str) -> Option<&Build<Trait = T>> {
        self.builders.get::<KeyAdaptor<T>>().and_then(|m| m.get(kind)).map(|b| &**b)
    }

    /// A utility method that deserializes a value.
    pub fn build<T: ?Sized + Any>(&self,
                                  trait_: &str,
                                  kind: &str,
                                  config: Value)
                                  -> Result<Box<T>, Box<error::Error>> {
        match self.get(kind) {
            Some(b) => b.build(config, self),
            None => {
                Err(format!("no {} builder for kind `{}` registered", trait_, kind).into())
            }
        }
    }
}

/// An error returned when deserializing a TOML configuration into a log4rs `Config`.
#[derive(Debug)]
pub enum Error {
    /// An error instantiating an appender.
    AppenderCreation(String, Box<error::Error>),
    /// An error instantiating a filter.
    FilterCreation(String, Box<error::Error>),
    /// An error when creating the log4rs `Config`.
    Config(config::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::AppenderCreation(ref appender, ref err) => {
                write!(fmt, "Error creating appender `{}`: {}", appender, err)
            }
            Error::FilterCreation(ref appender, ref err) => {
                write!(fmt,
                       "Error creating filter for appender `{}`: {}",
                       appender,
                       err)
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
            Error::AppenderCreation(_, ref err) |
            Error::FilterCreation(_, ref err) => Some(&**err),
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
    /// Creates a log4rs `Config` from the specified config string and `Builder`.
    pub fn parse(config: &str,
                 format: Format,
                 creator: &Builder)
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
                config::Root::builder(raw_root.level.0)
                    .appenders(raw_root.appenders)
                    .build()
            }
            None => config::Root::builder(LogLevelFilter::Debug).build(),
        };

        let mut config = config::Config::builder(root);

        for (name, raw::Appender { kind, config: raw_config, filters }) in raw_appenders {
            match creator.build("appender", &kind, raw_config) {
                Ok(appender_obj) => {
                    let mut builder = config::Appender::builder(name.clone(), appender_obj);
                    for raw::Filter { kind, config } in filters {
                        match creator.build("filter", &kind, config) {
                            Ok(filter) => builder = builder.filter(filter),
                            Err(err) => errors.push(Error::FilterCreation(name.clone(), err)),
                        }
                    }
                    config = config.appender(builder.build());
                }
                Err(err) => errors.push(Error::AppenderCreation(name, err)),
            }
        }

        for (name, logger) in raw_loggers {
            let raw::Logger { level, appenders, additive, .. } = logger;
            let mut logger = config::Logger::builder(name, level.0).appenders(appenders);
            if let Some(additive) = additive {
                logger = logger.additive(additive);
            }
            config = config.logger(logger.build());
        }

        let (config, config_errors) = config.build_lossy();
        if let Err(config_errors) = config_errors {
            for error in config_errors.unpack() {
                errors.push(Error::Config(error));
            }
        }

        let config = Config {
            refresh_rate: refresh_rate.map(|r| r.0),
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
    use std::borrow::ToOwned;
    use std::collections::{HashMap, BTreeMap};
    use std::time::Duration;
    use log::LogLevelFilter;
    use serde_value::Value;

    use super::*;
    use super::parse;
    use priv_serde::{Undeserializable, DeLogLevelFilter, DeDuration};

    #[test]
    #[cfg(feature = "yaml")]
    fn full_deserialize() {
        let cfg = r#"
refresh_rate: 60

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
        let config = Config::parse(cfg, Format::Yaml, &Builder::default()).unwrap();
        assert!(config.errors().is_empty());
    }

    #[allow(dead_code)]
    fn expected() -> raw::Config {
        raw::Config {
            refresh_rate: Some(DeDuration(Duration::from_secs(60))),
            appenders: {
                let mut m = HashMap::new();
                m.insert("console".to_owned(),
                         raw::Appender {
                             kind: "console".to_owned(),
                             config: Value::Map(BTreeMap::new()),
                             filters: vec![raw::Filter {
                                               kind: "threshold".to_string(),
                                               config: {
                                                   let mut m = BTreeMap::new();
                                                   m.insert(Value::String("level".to_string()),
                                                            Value::String("debug".to_string()));
                                                   Value::Map(m)
                                               },
                                           }],
                         });
                m.insert("baz".to_owned(),
                         raw::Appender {
                             kind: "file".to_owned(),
                             config: {
                                 let mut m = BTreeMap::new();
                                 m.insert(Value::String("file".to_owned()),
                                          Value::String("log/baz.log".to_owned()));
                                 Value::Map(m)
                             },
                             filters: vec![],
                         });
                m
            },
            root: Some(raw::Root {
                level: DeLogLevelFilter(LogLevelFilter::Info),
                appenders: vec!["console".to_owned()],
            }),
            loggers: {
                let mut m = HashMap::new();
                m.insert("foo::bar::baz".to_owned(),
                         raw::Logger {
                             level: DeLogLevelFilter(LogLevelFilter::Warn),
                             appenders: vec!["baz".to_owned()],
                             additive: Some(false),
                         });
                m
            },
        }
    }

    #[test]
    #[cfg(feature = "yaml")]
    fn basic_yaml() {
        let cfg = r#"
refresh_rate: 60

appenders:
  console:
    kind: console
    filters:
      - kind: threshold
        level: debug
  baz:
    kind: file
    file: log/baz.log

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

        let actual = parse(Format::Yaml, cfg).unwrap();
        let expected = expected();
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg(feature = "json")]
    fn basic_json() {
        let cfg = r#"
{
    "refresh_rate": 60,
    "appenders": {
        "console": {
            "kind": "console",
            "filters": [
                {
                    "kind": "threshold",
                    "level": "debug"
                }
            ]
        },
        "baz": {
            "kind": "file",
            "file": "log/baz.log"
        }
    },
    "root": {
        "appenders": ["console"],
        "level": "info"
    },
    "loggers": {
        "foo::bar::baz": {
            "level": "warn",
            "appenders": ["baz"],
            "additive": false
        }
    }
}"#;

        let actual = parse(Format::Json, cfg).unwrap();
        let expected = expected();
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg(feature = "toml")]
    fn basic_toml() {
        let cfg = r#"
refresh_rate = 60

[appenders.console]
kind = "console"
[[appenders.console.filters]]
kind = "threshold"
level = "debug"

[appenders.baz]
kind = "file"
file = "log/baz.log"

[root]
appenders = ["console"]
level = "info"

[loggers."foo::bar::baz"]
level = "warn"
appenders = ["baz"]
additive = false
"#;

        let actual = parse(Format::Toml, cfg).unwrap();
        let expected = expected();
        assert_eq!(expected, actual);
    }
}
