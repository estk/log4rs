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
//! # The maximum . If it is not present, the level of the logger's parent is used.
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
use std::collections::HashMap;
use std::default::Default;
use std::error;
use std::fmt;
use std::time::Duration;
use toml_parser::{self, Value};

use appender::{FileAppender, ConsoleAppender};
use config;
use pattern::PatternLayout;
use {Append, PrivateTomlConfigExt};

mod raw;

/// A trait implemented by types that can create appenders.
pub trait CreateAppender: Send+'static {
    /// Creates an appender with the specified config.
    fn create_appender(&self, config: &toml_parser::Table)
                       -> Result<Box<Append>, Box<error::Error>>;
}

/// A type that can create appenders.
///
/// `Creator` implementes `Default`, which returns a `Creator` with mappings
/// from "file" to `FileAppenderCreator` and "console" to
/// `ConsoleAppenderCreator`.
pub struct Creator {
    appenders: HashMap<String, Box<CreateAppender>>,
}

impl Default for Creator {
    fn default() -> Creator {
        let mut creator = Creator::new();
        creator.add_appender("file", Box::new(FileAppenderCreator));
        creator.add_appender("console", Box::new(ConsoleAppenderCreator));
        creator
    }
}

impl Creator {
    /// Creates a new `Creator` with no appender mappings.
    pub fn new() -> Creator {
        Creator {
            appenders: HashMap::new(),
        }
    }

    /// Adds a mapping from the specified `kind` to the specified appender
    /// creator.
    pub fn add_appender(&mut self, kind: &str, creator: Box<CreateAppender>) {
        self.appenders.insert(kind.to_string(), creator);
    }

    fn create_appender(&self, kind: &str, config: &toml_parser::Table)
                       -> Result<Box<Append>, Box<error::Error>> {
        match self.appenders.get(kind) {
            Some(creator) => creator.create_appender(config),
            None => Err(Box::new(StringError(format!("No creator registered for appender kind \"{}\"", kind))))
        }
    }
}

/// An error returned when deserializing a TOML configuration into a log4rs `Config`.
pub enum Error {
    /// An error during TOML parsing.
    Parse(Vec<String>),
    /// An error instantiating appenders.
    Creation(Box<error::Error>),
    /// An error when creating the log4rs `Config`.
    Config(config::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Parse(ref errs) => {
                try!(writeln!(fmt, "Parse errors:"));
                for err in errs {
                    try!(writeln!(fmt, "{}", err));
                }
                Ok(())
            }
            Error::Creation(ref err) => write!(fmt, "Error creating appender: {}", err),
            Error::Config(ref err) => write!(fmt, "Error creating config: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "An error creating a log4rs `Config` from a TOML file"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Creation(ref err) => Some(&**err),
            Error::Config(ref err) => Some(err),
            _ => None
        }
    }
}

/// A deserialized TOML log4rs configuration.
pub struct Config {
    refresh_rate: Option<Duration>,
    config: config::Config,
}

impl Config {
    /// Creates a log4rs `Config` from the specified TOML config string and `Creator`.
    pub fn parse(config: &str, creator: &Creator) -> Result<Config, Error> {
        let config = match raw::parse(config) {
            Ok(config) => config,
            Err(err) => return Err(Error::Parse(err)),
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

        for (name, appender) in raw_appenders {
            let appender = match creator.create_appender(&appender.kind, &appender.config) {
                Ok(appender) => appender,
                Err(err) => return Err(Error::Creation(err)),
            };
            config = config.appender(config::Appender::builder(name, appender).build());
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

        match config.build() {
            Ok(config) => {
                Ok(Config {
                    refresh_rate: refresh_rate,
                    config: config,
                })
            }
            Err(err) => Err(Error::Config(err))
        }
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

impl error::FromError<String> for StringError {
    fn from_error(s: String) -> StringError {
        StringError(s)
    }
}

/// An appender creator for the `FileAppender`.
///
/// The `path` key is required, and specifies the path to the log file. The
/// `pattern` key is optional and specifies a `PatternLayout` pattern to be
/// used for output.
pub struct FileAppenderCreator;

impl CreateAppender for FileAppenderCreator {
    fn create_appender(&self, config: &toml_parser::Table)
                       -> Result<Box<Append>, Box<error::Error>> {
        let path = match config.get("path") {
            Some(&Value::String(ref path)) => path,
            Some(_) => return Err(Box::new(StringError("`path` must be a string".to_string()))),
            None => return Err(Box::new(StringError("`path` is required".to_string()))),
        };
        let mut appender = FileAppender::builder(path);
        match config.get("pattern") {
            Some(&Value::String(ref pattern)) => {
                appender = appender.pattern(try!(PatternLayout::new(pattern)));
            }
            Some(_) => return Err(Box::new(StringError("`pattern` must be a string".to_string()))),
            None => {}
        }

        match appender.build() {
            Ok(appender) => Ok(Box::new(appender)),
            Err(err) => Err(Box::new(err))
        }
    }
}

/// An appender creator for the `ConsoleAppender`.
///
/// The `pattern` key is optional and specifies a `PatternLayout` pattern to be
/// used for output.
pub struct ConsoleAppenderCreator;

impl CreateAppender for ConsoleAppenderCreator {
    fn create_appender(&self, config: &toml_parser::Table)
                       -> Result<Box<Append>, Box<error::Error>> {
        let mut appender = ConsoleAppender::builder();
        match config.get("pattern") {
            Some(&Value::String(ref pattern)) => {
                appender = appender.pattern(try!(PatternLayout::new(pattern)));
            }
            Some(_) => return Err(Box::new(StringError("`pattern` must be a string".to_string()))),
            None => {}
        }

        Ok(Box::new(appender.build()))
    }
}
