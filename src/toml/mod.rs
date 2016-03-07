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
//! # appender mapping provided to the `Creator` used to deserialize the
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
use toml_parser::{self, Value};
use typemap::{Key, ShareMap};

use appender::{FileAppender, ConsoleAppender};
use filter::ThresholdFilter;
use config;
use pattern::PatternLayout;
use {Append, Filter, PrivateTomlConfigExt, PrivateConfigErrorsExt};

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
             config: toml_parser::Table)
             -> Result<Box<Self::Trait>, Box<error::Error>>;
}

/// A type that can create appenders.
///
/// `Creator` implements `Default`, which returns a `Creator` with the
/// following mappings:
///
/// * Appenders
///     * "file" -> `FileAppenderCreator`
///     * "console" -> `ConsoleAppenderCreator`
/// * Filters
///     * "threshold" -> `ThresholdFilterCreator`
pub struct Creator {
    builders: ShareMap,
}

impl Default for Creator {
    fn default() -> Creator {
        let mut creator = Creator::new();
        creator.insert("file", Box::new(FileAppenderBuilder));
        creator.insert("console", Box::new(ConsoleAppenderBuilder));
        creator.insert("threshold", Box::new(ThresholdFilterBuilder));
        creator
    }
}

impl Creator {
    /// Creates a new `Creator` with no appender or filter mappings.
    pub fn new() -> Creator {
        Creator {
            builders: ShareMap::custom(),
        }
    }

    /// Adds a mapping from the specified `kind` to a builder.
    pub fn insert<T: ?Sized + Any>(&mut self, kind: &str, builder: Box<Build<Trait = T>>) {
        self.builders.entry::<KeyAdaptor<T>>().or_insert(HashMap::new()).insert(kind.to_owned(), builder);
    }

    fn build<T: ?Sized + Any>(&self,
                              kind: &str,
                              config: toml_parser::Table)
                              -> Result<Box<T>, Box<error::Error>> {
        match self.builders.get::<KeyAdaptor<T>>().and_then(|m| m.get(kind)) {
            Some(builder) => builder.build(config),
            None => {
                Err(Box::new(StringError(format!("No builder registered for kind `{}`", kind))))
            }
        }
    }
}

/// Errors encountered when parsing a log4rs TOML config.
#[derive(Debug)]
pub struct ParseErrors {
    errors: Vec<String>,
}

impl fmt::Display for ParseErrors {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.errors {
            try!(writeln!(fmt, "{}", error));
        }
        Ok(())
    }
}

impl error::Error for ParseErrors {
    fn description(&self) -> &str {
        "Errors encountered when parsing a log4rs TOML config"
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
        "An error encountered when deserializing a TOML configuration into a log4rs `Config`"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::AppenderCreation(_, ref err) => Some(&**err),
            Error::FilterCreation(_, ref err) => Some(&**err),
            Error::Config(ref err) => Some(err),
        }
    }
}

/// Errors encountered when deserializing a TOML configuration into a log4rs `Config`.
#[derive(Debug)]
pub struct Errors {
    errors: Vec<Error>,
}

impl Errors {
    /// Returns the list of errors encountered.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.errors {
            try!(writeln!(fmt, "{}", error));
        }
        Ok(())
    }
}

impl error::Error for Errors {
    fn description(&self) -> &str {
        "Errors encountered when deserializing a TOML configuration into a log4rs `Config`"
    }
}

/// A deserialized TOML log4rs configuration.
pub struct Config {
    refresh_rate: Option<Duration>,
    config: config::Config,
}

impl Config {
    /// Creates a log4rs `Config` from the specified TOML config string and `Creator`.
    pub fn parse(config: &str,
                 creator: &Creator)
                 -> Result<(Config, Result<(), Errors>), ParseErrors> {
        let mut errors = vec![];

        let config = match raw::parse(config) {
            Ok(config) => config,
            Err(errors) => return Err(ParseErrors { errors: errors }),
        };

        let raw::Config {
            refresh_rate,
            root: raw_root,
            appenders: raw_appenders,
            loggers: raw_loggers,
        } = config;

        let root = match raw_root {
            Some(raw_root) => {
                let mut root = config::Root::builder(raw_root.level);
                if let Some(appenders) = raw_root.appenders {
                    root = root.appenders(appenders);
                }
                root.build()
            }
            None => config::Root::builder(LogLevelFilter::Debug).build(),
        };

        let mut config = config::Config::builder(root);

        for (name, raw::Appender { kind, config: raw_config, filters }) in raw_appenders {
            match creator.build(&kind, raw_config) {
                Ok(appender_obj) => {
                    let mut builder = config::Appender::builder(name.clone(), appender_obj);
                    for raw::Filter { kind, config } in filters.unwrap_or(vec![]) {
                        match creator.build(&kind, config) {
                            Ok(filter) => builder = builder.filter(filter),
                            Err(err) => errors.push(Error::FilterCreation(name.clone(), err)),
                        }
                    }
                    config = config.appender(builder.build());
                }
                Err(err) => errors.push(Error::AppenderCreation(name, err)),
            }
        }

        for logger in raw_loggers {
            let raw::Logger { name, level, appenders, additive } = logger;
            let mut logger = config::Logger::builder(name, level);
            if let Some(appenders) = appenders {
                logger = logger.appenders(appenders);
            }
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
            refresh_rate: refresh_rate,
            config: config,
        };

        let errors = if errors.is_empty() {
            Ok(())
        } else {
            Err(Errors { errors: errors })
        };

        Ok((config, errors))
    }

    /// Returns the requested refresh rate.
    pub fn refresh_rate(&self) -> Option<Duration> {
        self.refresh_rate
    }

    /// Returns the log4rs `Config`.
    pub fn config(&self) -> &config::Config {
        &self.config
    }
}

impl PrivateTomlConfigExt for Config {
    fn unpack(self) -> (Option<Duration>, config::Config) {
        let Config { refresh_rate, config } = self;
        (refresh_rate, config)
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

fn ensure_empty(config: &toml_parser::Table) -> Result<(), Box<error::Error>> {
    let remaining_keys: Vec<_> = config.keys().collect();
    if remaining_keys.is_empty() {
        Ok(())
    } else {
        Err(Box::new(StringError(format!("Unknown keys: {:?}", remaining_keys))))
    }
}

/// An builder for the `FileAppender`.
///
/// The `path` key is required, and specifies the path to the log file. The
/// `pattern` key is optional and specifies a `PatternLayout` pattern to be
/// used for output. The `append` key is optional and specifies whether the
/// output file should be truncated or appended to.
pub struct FileAppenderBuilder;

impl Build for FileAppenderBuilder {
    type Trait = Append;

    fn build(&self, mut config: toml_parser::Table) -> Result<Box<Append>, Box<error::Error>> {
        let path = match config.remove("path") {
            Some(Value::String(path)) => path,
            Some(_) => return Err(Box::new(StringError("`path` must be a string".to_string()))),
            None => return Err(Box::new(StringError("`path` is required".to_string()))),
        };

        let mut appender = FileAppender::builder(&path);
        match config.remove("pattern") {
            Some(Value::String(pattern)) => {
                appender = appender.pattern(try!(PatternLayout::new(&pattern)));
            }
            Some(_) => return Err(Box::new(StringError("`pattern` must be a string".to_string()))),
            None => {}
        }

        match config.remove("append") {
            Some(Value::Boolean(append)) => appender = appender.append(append),
            None => {}
            Some(_) => return Err(Box::new(StringError("`append` must be a bool".to_string()))),
        }

        try!(ensure_empty(&config));
        match appender.build() {
            Ok(appender) => Ok(Box::new(appender)),
            Err(err) => Err(Box::new(err)),
        }
    }
}

/// An builder for the `ConsoleAppender`.
///
/// The `pattern` key is optional and specifies a `PatternLayout` pattern to be
/// used for output.
pub struct ConsoleAppenderBuilder;

impl Build for ConsoleAppenderBuilder {
    type Trait = Append;

    fn build(&self, mut config: toml_parser::Table) -> Result<Box<Append>, Box<error::Error>> {
        let mut appender = ConsoleAppender::builder();
        match config.remove("pattern") {
            Some(Value::String(pattern)) => {
                appender = appender.pattern(try!(PatternLayout::new(&pattern)));
            }
            Some(_) => return Err(Box::new(StringError("`pattern` must be a string".to_string()))),
            None => {}
        }

        try!(ensure_empty(&config));
        Ok(Box::new(appender.build()))
    }
}

/// A builder for the `ThresholdFilter`.
///
/// The `level` key is required and specifies the threshold for the filter.
pub struct ThresholdFilterBuilder;

impl Build for ThresholdFilterBuilder {
    type Trait = Filter;

    fn build(&self, mut config: toml_parser::Table) -> Result<Box<Filter>, Box<error::Error>> {
        let level = match config.remove("level") {
            Some(Value::String(level)) => level,
            Some(_) => return Err(Box::new(StringError("`level` must be a string".to_string()))),
            None => return Err(Box::new(StringError("`level` must be provided".to_string()))),
        };

        let level = match level.parse() {
            Ok(level) => level,
            Err(_) => return Err(Box::new(StringError(format!("Invalid `level` \"{}\"", level)))),
        };

        try!(ensure_empty(&config));
        Ok(Box::new(ThresholdFilter::new(level)))
    }
}
