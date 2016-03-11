//! Support for log4rs configuration from TOML files.
//!
//! # Syntax
//!
//! ```toml
//! # If set, log4rs will scan the file at the specified rate in seconds for
//! # changes and automatically reconfigure the logger.
//! refresh_rate = 30
//!
//! # Appenders are configured as tables inside the "appender" table. This
//! # appender is named "foo".
//! [appender.foo]
//! # All appenders must specify a "kind", which must match the kind of an
//! # appender mapping provided to the `Builder` used to deserialize the
//! # config file.
//! kind = "console"
//!
//! # Arbitrary fields may be added to appender configurations. Remaining
//! # entries will be passed to the `CreateAppender` object associated with
//! # the specified kind.
//! pattern = "%d [%t] %m"
//!
//! # Filters attached to an appender are configured inside the "filter" array.
//! [[appender.foo.filter]]
//! # Like appenders, filters must specify a "kind".
//! kind = "threshold"
//!
//! # Also like appenders, arbitrary fields may be added to filter
//! # configurations.
//! level = "error"
//!
//! # The root logger is configured by the "root" table. It is optional.
//! # If the "root" table is not specified, the root will default to a level of
//! # "debug" and no appenders.
//! [root]
//! # The maximum log level for the root logger. Must be specified if the
//! # "root" table is defined.
//! level = "warn"
//!
//! # The list of names of appenders attached to the root logger. If not
//! # specified, defaults to an empty list.
//! appenders = ["foo"]
//!
//! # Loggers are configured as tables inside of the "logger" array.
//! [[logger]]
//! # The name of the logger. Must be specified.
//! name = "foo::bar::baz"
//!
//! # The maximum log level. If it is not present, the level of the logger's
//! # parent is used.
//! level = "trace"
//!
//! # A list of names of appenders attached to the logger. If not specified,
//! # defaults to an empty list.
//! appenders = ["foo"]
//!
//! # The additivity of the logger. If true, the appenders attached to this
//! # logger's parent will also be attached to this logger. If not specified,
//! # defaults to true.
//! additive = false
//! ```
use log::LogLevelFilter;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::error;
use std::fmt;
use time::Duration;
use typemap::{Key, ShareMap};
use serde_value::Value;

use appender::{Append, FileAppender, ConsoleAppender};
use filter::{Filter, ThresholdFilter};
use config;
use encoder::Encode;
use encoder::pattern::PatternEncoder;
use PrivateConfigErrorsExt;

mod raw;

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
///     * "file" -> `FileAppenderBuilder`
///     * "console" -> `ConsoleAppenderBuilder`
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
        creator.insert("file", Box::new(FileAppenderBuilder));
        creator.insert("console", Box::new(ConsoleAppenderBuilder));
        creator.insert("threshold", Box::new(ThresholdFilterBuilder));
        creator.insert("pattern", Box::new(PatternEncoderBuilder));
        creator
    }
}

impl Builder {
    /// Creates a new `Builder` with no appender or filter mappings.
    pub fn new() -> Builder {
        Builder { builders: ShareMap::custom() }
    }

    /// Adds a mapping from the specified `kind` to a builder.
    pub fn insert<T: ?Sized + Any>(&mut self, kind: &str, builder: Box<Build<Trait = T>>) {
        self.builders
            .entry::<KeyAdaptor<T>>()
            .or_insert(HashMap::new())
            .insert(kind.to_owned(), builder);
    }

    /// Retrieves the builder of the specified `kind`.
    pub fn get<T: ?Sized + Any>(&self, kind: &str) -> Option<&Build<Trait = T>> {
        self.builders.get::<KeyAdaptor<T>>().and_then(|m| m.get(kind)).map(|b| &**b)
    }

    fn build<T: ?Sized + Any>(&self,
                              trait_: &str,
                              kind: &str,
                              config: Value)
                              -> Result<Box<T>, Box<error::Error>> {
        match self.get(kind) {
            Some(b) => b.build(config, self),
            None => {
                Err(Box::new(StringError(format!("no {} builder for kind `{}` registered",
                                                 trait_,
                                                 kind))))
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
            Error::AppenderCreation(_, ref err) => Some(&**err),
            Error::FilterCreation(_, ref err) => Some(&**err),
            Error::Config(ref err) => Some(err),
        }
    }
}

/// Specifies the format of a configuration file.
#[derive(Copy, Clone)]
pub enum Format {
    /// YAML.
    Yaml,
}

/// A deserialized log4rs configuration file.
pub struct Config {
    refresh_rate: Option<Duration>,
    config: config::Config,
    errors: Vec<Error>,
}

impl Config {
    /// Creates a log4rs `Config` from the specified TOML config string and `Builder`.
    pub fn parse(config: &str,
                 format: Format,
                 creator: &Builder)
                 -> Result<Config, Box<error::Error>> {
        let mut errors = vec![];

        let config = try!(raw::parse(format, config));

        let raw::Config { refresh_rate,
                          root: raw_root,
                          appenders: raw_appenders,
                          loggers: raw_loggers } = config;

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
            let raw::Logger { level, appenders, additive } = logger;
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

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.0)
    }
}

impl error::Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl From<String> for StringError {
    fn from(s: String) -> StringError {
        StringError(s)
    }
}

/// An builder for the `FileAppender`.
///
/// The `path` key is required, and specifies the path to the log file. The
/// `pattern` key is optional and specifies a `PatternEncoder` pattern to be
/// used for output. The `append` key is optional and specifies whether the
/// output file should be truncated or appended to.
pub struct FileAppenderBuilder;

impl Build for FileAppenderBuilder {
    type Trait = Append;

    fn build(&self, config: Value, builder: &Builder) -> Result<Box<Append>, Box<error::Error>> {
        let config = try!(config.deserialize_into::<raw::FileAppenderConfig>());
        let mut appender = FileAppender::builder(&config.path);
        if let Some(append) = config.append {
            appender = appender.append(append);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(try!(builder.build("encoder",
                                                           &encoder.kind,
                                                           encoder.config)));
        }
        Ok(Box::new(try!(appender.build())))
    }
}

/// An builder for the `ConsoleAppender`.
///
/// The `pattern` key is optional and specifies a `PatternEncoder` pattern to be
/// used for output.
pub struct ConsoleAppenderBuilder;

impl Build for ConsoleAppenderBuilder {
    type Trait = Append;

    fn build(&self, config: Value, builder: &Builder) -> Result<Box<Append>, Box<error::Error>> {
        let config = try!(config.deserialize_into::<raw::ConsoleAppenderConfig>());
        let mut appender = ConsoleAppender::builder();
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(try!(builder.build("encoder",
                                                           &encoder.kind,
                                                           encoder.config)));
        }
        Ok(Box::new(appender.build()))
    }
}

/// A builder for the `ThresholdFilter`.
///
/// The `level` key is required and specifies the threshold for the filter.
pub struct ThresholdFilterBuilder;

impl Build for ThresholdFilterBuilder {
    type Trait = Filter;

    fn build(&self, config: Value, _: &Builder) -> Result<Box<Filter>, Box<error::Error>> {
        let config = try!(config.deserialize_into::<raw::ThresholdFilterConfig>());
        Ok(Box::new(ThresholdFilter::new(config.level.0)))
    }
}

/// A builder for the `PatternEncoder`.
///
/// The `pattern` key is required and specifies the pattern for the encoder.
pub struct PatternEncoderBuilder;

impl Build for PatternEncoderBuilder {
    type Trait = Encode;

    fn build(&self, config: Value, _: &Builder) -> Result<Box<Encode>, Box<error::Error>> {
        let config = try!(config.deserialize_into::<raw::PatternEncoderConfig>());
        let encoder = match config.pattern {
            Some(pattern) => try!(PatternEncoder::new(&pattern)),
            None => PatternEncoder::default(),
        };
        Ok(Box::new(encoder))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
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
}
