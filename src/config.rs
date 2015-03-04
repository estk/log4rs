//! log4rs configuration

use std::collections::HashSet;
use std::fmt;
use std::iter::IntoIterator;
use std::error;
use log::LogLevelFilter;

use {Append, ConfigPrivateExt};

/// Configuration for the root logger.
#[derive(Debug)]
pub struct Root {
    level: LogLevelFilter,
    appenders: Vec<String>,
}

impl Root {
    /// Creates a new `RootBuilder` with no appenders and the specified level.
    pub fn builder(level: LogLevelFilter) -> RootBuilder {
        RootBuilder(Root {
            level: level,
            appenders: vec![],
        })
    }

    /// Returns the minimum level of log messages that the root logger will accept.
    pub fn level(&self) -> LogLevelFilter {
        self.level
    }

    /// Returns the list of names of appenders that will be attached to the root logger.
    pub fn appenders(&self) -> &[String] {
        &self.appenders
    }
}

/// A builder for `Root`s.
#[derive(Debug)]
pub struct RootBuilder(Root);

impl RootBuilder {
    /// Adds an appender.
    pub fn appender(mut self, appender: String) -> RootBuilder {
        self.0.appenders.push(appender);
        self
    }

    /// Adds appenders.
    pub fn appenders<I: IntoIterator<Item=String>>(mut self, appenders: I) -> RootBuilder {
        self.0.appenders.extend(appenders);
        self
    }

    /// Consumes the `RootBuilder`, returning the `Root`.
    pub fn build(self) -> Root {
        self.0
    }
}

/// Configuration for an appender.
pub struct Appender {
    name: String,
    /// The appender trait object.
    appender: Box<Append>,
}

impl fmt::Debug for Appender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Appender {{ name: {} }}", self.name)
    }
}

impl Appender {
    /// Creates a new `AppenderBuilder` with the specified name and `Append` trait object.
    pub fn builder(name: String, appender: Box<Append>) -> AppenderBuilder {
        AppenderBuilder(Appender {
            name: name,
            appender: appender,
        })
    }

    /// Returns the name of the appender.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Consumes the `Appender`, returning the `Append` trait object for the appender.
    pub fn appender(self) -> Box<Append> {
        self.appender
    }
}

/// A builder for `Appender`s.
#[derive(Debug)]
pub struct AppenderBuilder(Appender);

impl AppenderBuilder {
    /// Consumes the `AppenderBuilder`, returning the `Appender`.
    pub fn build(self) -> Appender {
        self.0
    }
}

/// Configuration for a logger.
#[derive(Debug)]
pub struct Logger {
    /// The name of the logger.
    name: String,
    /// The minimum level of log messages that the logger will accept.
    level: LogLevelFilter,
    /// The set of names of appenders that will be attached to the logger.
    appenders: Vec<String>,
    /// If `true`, appenders of parent loggers will also be attached to this logger.
    additive: bool,
}

impl Logger {
    /// Creates a new `LoggerBuilder` with the specified name and level.
    ///
    /// There are initially no appenders attached and `additive` is `true`.
    pub fn builder(name: String, level: LogLevelFilter) -> LoggerBuilder {
        LoggerBuilder(Logger {
            name: name,
            level: level,
            appenders: vec![],
            additive: true,
        })
    }

    /// Returns the name of the logger.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the minimum level of log messages that the logger will accept.
    pub fn level(&self) -> LogLevelFilter {
        self.level
    }

    /// Returns the list of names of appenders that will be attached to the logger.
    pub fn appenders(&self) -> &[String] {
        &self.appenders
    }

    /// Determines if appenders of parent loggers will also be attached to this logger.
    pub fn additive(&self) -> bool {
        self.additive
    }
}

/// A builder for `Logger`s.
#[derive(Debug)]
pub struct LoggerBuilder(Logger);

impl LoggerBuilder {
    /// Adds an appender.
    pub fn appender(mut self, appender: String) -> LoggerBuilder {
        self.0.appenders.push(appender);
        self
    }

    /// Adds appenders.
    pub fn appenders<I: IntoIterator<Item=String>>(mut self, appenders: I) -> LoggerBuilder {
        self.0.appenders.extend(appenders);
        self
    }

    /// Sets the additivity of the logger.
    pub fn additive(mut self, additive: bool) -> LoggerBuilder {
        self.0.additive = additive;
        self
    }

    /// Consumes the `LoggerBuilder`, returning the `Logger`.
    pub fn build(self) -> Logger {
        self.0
    }
}

/// A log4rs configuration.
#[derive(Debug)]
pub struct Config {
    appenders: Vec<Appender>,
    root: Root,
    loggers: Vec<Logger>,
}

impl Config {
    /// Creates a new `ConfigBuilder` with the specified `Root`.
    pub fn builder(root: Root) -> ConfigBuilder {
        ConfigBuilder(Config {
            appenders: vec![],
            root: root,
            loggers: vec![],
        })
    }

    /// Returns the `Appender`s associated with the `Config`.
    pub fn appenders(&self) -> &[Appender] {
        &self.appenders
    }

    /// Returns the `Root` associated with the `Config`.
    pub fn root(&self) -> &Root {
        &self.root
    }

    /// Returns the `Logger`s associated with the `Config`.
    pub fn loggers(&self) -> &[Logger] {
        &self.loggers
    }
}

/// A builder for `Config`s.
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    /// Adds an appender.
    pub fn appender(mut self, appender: Appender) -> ConfigBuilder {
        self.0.appenders.push(appender);
        self
    }

    /// Adds appenders.
    pub fn appenders<I: IntoIterator<Item=Appender>>(mut self, appenders: I) -> ConfigBuilder {
        self.0.appenders.extend(appenders);
        self
    }

    /// Adds a logger.
    pub fn logger(mut self, logger: Logger) -> ConfigBuilder {
        self.0.loggers.push(logger);
        self
    }

    /// Adds loggers.
    pub fn loggers<I: IntoIterator<Item=Logger>>(mut self, loggers: I) -> ConfigBuilder {
        self.0.loggers.extend(loggers);
        self
    }

    /// Consumes the `ConfigBuilder`, returning the `Config`.
    pub fn build(self) -> Result<Config, Error> {
        {
            let mut appender_names = HashSet::new();

            for appender in &self.0.appenders {
                if !appender_names.insert(&appender.name) {
                    return Err(Error::DuplicateAppenderName(appender.name.clone()));
                }
            }

            for appender in &self.0.root.appenders {
                if !appender_names.contains(&appender) {
                    return Err(Error::NonexistentAppender(appender.clone()));
                }
            }

            let mut logger_names = HashSet::new();
            for logger in &self.0.loggers {
                if !logger_names.insert(&logger.name) {
                    return Err(Error::DuplicateLoggerName(logger.name.clone()));
                }
                try!(check_logger_name(&logger.name));

                for appender in &logger.appenders {
                    if !appender_names.contains(&appender) {
                        return Err(Error::NonexistentAppender(appender.clone()));
                    }
                }
            }
        }

        Ok(self.0)
    }
}

fn check_logger_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(Error::InvalidLoggerName(name.to_string()));
    }

    let mut streak = 0;
    for ch in name.chars() {
        if ch != ':' {
            if streak > 0 && streak != 2 {
                return Err(Error::InvalidLoggerName(name.to_string()));
            }
            streak = 0;
        } else {
            streak += 1;
            if streak > 2 {
                return Err(Error::InvalidLoggerName(name.to_string()));
            }
        }
    }

    if streak > 0 {
        Err(Error::InvalidLoggerName(name.to_string()))
    } else {
        Ok(())
    }
}

impl ConfigPrivateExt for Config {
    fn unpack(self) -> (Vec<Appender>, Root, Vec<Logger>) {
        let Config { appenders, root, loggers } = self;
        (appenders, root, loggers)
    }
}

/// An error validating a log4rs `Config`.
#[derive(PartialEq, Debug)]
pub enum Error {
    /// Multiple appenders were registered with the same name.
    DuplicateAppenderName(String),
    /// A reference to a nonexistant appender.
    NonexistentAppender(String),
    /// Multiple loggers were registered with the same name.
    DuplicateLoggerName(String),
    /// A logger name was invalid.
    InvalidLoggerName(String),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::DuplicateAppenderName(ref n) => write!(fmt, "Duplicate appender name `{}`", n),
            Error::NonexistentAppender(ref n) => {
                write!(fmt, "Reference to nonexistent appender: `{}`", n)
            }
            Error::DuplicateLoggerName(ref n) => write!(fmt, "Duplicate logger name `{}`", n),
            Error::InvalidLoggerName(ref n) => write!(fmt, "Invalid logger name `{}`", n),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "An error constructing a Config"
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn check_logger_name() {
        let tests = [
            ("", false),
            ("asdf", true),
            ("asdf::jkl", true),
            ("::", false),
            ("asdf::jkl::", false),
            ("asdf:jkl", false),
            ("asdf:::jkl", false),
            ("asdf::jkl::", false),
        ];

        for &(ref name, expected) in &tests {
            assert!(expected == super::check_logger_name(name).is_ok(), "{}", name);
        }
    }
}
