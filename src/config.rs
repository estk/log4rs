//! log4rs configuration

use std::collections::HashSet;
use std::fmt;
use std::iter::IntoIterator;
use std::error;
use log::LogLevelFilter;

use {Append, Filter, ConfigPrivateExt, PrivateConfigErrorsExt, PrivateConfigAppenderExt};

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
    appender: Box<Append>,
    filters: Vec<Box<Filter>>,
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
            filters: vec![],
        })
    }

    /// Returns the name of the appender.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the appender.
    pub fn appender(&self) -> &Append {
        &*self.appender
    }

    /// Returns the filters attached to the appender.
    pub fn filters(&self) -> &[Box<Filter>] {
        &self.filters
    }
}

impl PrivateConfigAppenderExt for Appender {
    fn unpack(self) -> (String, Box<Append>, Vec<Box<Filter>>) {
        let Appender { name, appender, filters } = self;
        (name, appender, filters)
    }
}

/// A builder for `Appender`s.
#[derive(Debug)]
pub struct AppenderBuilder(Appender);

impl AppenderBuilder {
    /// Adds a filter.
    pub fn filter(mut self, filter: Box<Filter>) -> AppenderBuilder {
        self.0.filters.push(filter);
        self
    }

    /// Adds filters.
    pub fn filters<I: IntoIterator<Item=Box<Filter>>>(mut self, filters: I) -> AppenderBuilder {
        self.0.filters.extend(filters);
        self
    }

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
    ///
    /// Unlike `build`, this method will always return a `Config` by stripping
    /// portions of the configuration that are incorrect.
    pub fn build_lossy(self) -> (Config, Result<(), Errors>) {
        let mut errors = vec![];

        let Config { appenders, mut root, loggers } = self.0;

        let mut ok_appenders = vec![];
        let mut appender_names = HashSet::new();
        for appender in appenders {
            if appender_names.insert(appender.name.clone()) {
                ok_appenders.push(appender);
            } else {
                errors.push(Error::DuplicateAppenderName(appender.name));
            }
        }

        let mut ok_root_appenders = vec![];
        for appender in root.appenders {
            if appender_names.contains(&appender) {
                ok_root_appenders.push(appender);
            } else {
                errors.push(Error::NonexistentAppender(appender));
            }
        }
        root.appenders = ok_root_appenders;

        let mut ok_loggers = vec![];
        let mut logger_names = HashSet::new();
        for mut logger in loggers {
            if !logger_names.insert(logger.name.clone()) {
                errors.push(Error::DuplicateLoggerName(logger.name));
                continue;
            }

            if let Err(err) = check_logger_name(&logger.name) {
                errors.push(err);
                continue;
            }

            let mut ok_logger_appenders = vec![];
            for appender in logger.appenders {
                if appender_names.contains(&appender) {
                    ok_logger_appenders.push(appender);
                } else {
                    errors.push(Error::NonexistentAppender(appender));
                }
            }
            logger.appenders = ok_logger_appenders;

            ok_loggers.push(logger);
        }

        let config = Config {
            appenders: ok_appenders,
            root: root,
            loggers: ok_loggers,
        };

        let errors = if errors.is_empty() {
            Ok(())
        } else {
            Err(Errors { errors: errors })
        };

        (config, errors)
    }

    /// Consumes the `ConfigBuilder`, returning the `Config`.
    pub fn build(self) -> Result<Config, Errors> {
        let (config, errors) = self.build_lossy();
        errors.map(|_| config)
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

/// Errors encountered when validating a log4rs `Config`.
#[derive(Debug)]
pub struct Errors {
    errors: Vec<Error>,
}

impl Errors {
    /// Returns a slice of `Error`s.
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
        "Errors encountered when validating a log4rs `Config`"
    }
}

impl PrivateConfigErrorsExt for Errors {
    fn unpack(self) -> Vec<Error> {
        self.errors
    }
}

/// An error validating a log4rs `Config`.
#[derive(Debug)]
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
        "An error constructing a log4rs `Config`"
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
