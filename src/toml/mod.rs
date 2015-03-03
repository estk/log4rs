//! Support for log4rs configuration from TOML files.
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
use Append;

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
    pub refresh_rate: Option<Duration>,
    pub config: config::Config,
    _p: ()
}

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

    let mut appenders = vec![];
    for (name, appender) in raw_appenders {
        let appender = match creator.create_appender(&appender.kind, &appender.config) {
            Ok(appender) => appender,
            Err(err) => return Err(Error::Creation(err)),
        };
        appenders.push(config::Appender::new(name, appender))
    }

    let root = match raw_root {
        Some(raw_root) => {
            let mut root = config::Root::new(raw_root.level);
            if let Some(appenders) = raw_root.appenders {
                root.appenders.extend(appenders.into_iter());
            }
            root
        }
        None => config::Root::new(LogLevelFilter::Debug),
    };

    let mut loggers = vec![];
    for logger in raw_loggers {
        let raw::Logger { name, level, appenders, additive } = logger;
        let mut logger = config::Logger::new(name, level);
        logger.appenders = appenders.unwrap_or(vec![]);
        logger.additive = additive.unwrap_or(true);
        loggers.push(logger);
    }

    match config::Config::new(appenders, root, loggers) {
        Ok(config) => Ok(Config {
            refresh_rate: refresh_rate,
            config: config,
            _p: (),
        }),
        Err(err) => Err(Error::Config(err))
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
