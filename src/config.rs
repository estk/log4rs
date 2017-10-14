//! log4rs configuration

use std::collections::HashSet;
use std::fmt;
use std::iter::IntoIterator;
use std::error;
use log::LevelFilter;

use append::Append;
use filter::Filter;
use {ConfigPrivateExt, PrivateConfigAppenderExt};

/// Configuration for the root logger.
#[derive(Debug)]
pub struct Root {
    level: LevelFilter,
    appenders: Vec<String>,
}

impl Root {
    /// Creates a new `RootBuilder` with no appenders.
    pub fn builder() -> RootBuilder {
        RootBuilder { appenders: vec![] }
    }

    /// Returns the minimum level of log messages that the root logger will accept.
    pub fn level(&self) -> LevelFilter {
        self.level
    }

    /// Returns the list of names of appenders that will be attached to the root logger.
    pub fn appenders(&self) -> &[String] {
        &self.appenders
    }
}

/// A builder for `Root`s.
#[derive(Debug)]
pub struct RootBuilder {
    appenders: Vec<String>,
}

impl RootBuilder {
    /// Adds an appender.
    pub fn appender<T>(mut self, appender: T) -> RootBuilder
        where T: Into<String>
    {
        self.appenders.push(appender.into());
        self
    }

    /// Adds appenders.
    pub fn appenders<I>(mut self, appenders: I) -> RootBuilder
        where I: IntoIterator,
              I::Item: Into<String>,
    {
        self.appenders.extend(appenders.into_iter().map(Into::into));
        self
    }

    /// Consumes the `RootBuilder`, returning the `Root`.
    pub fn build(self, level: LevelFilter) -> Root {
        Root {
            level: level,
            appenders: self.appenders,
        }
    }
}

/// Configuration for an appender.
#[derive(Debug)]
pub struct Appender {
    name: String,
    appender: Box<Append>,
    filters: Vec<Box<Filter>>,
}

impl Appender {
    /// Creates a new `AppenderBuilder` with the specified name and `Append` trait object.
    pub fn builder() -> AppenderBuilder {
        AppenderBuilder { filters: vec![] }
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
pub struct AppenderBuilder {
    filters: Vec<Box<Filter>>,
}

impl AppenderBuilder {
    /// Adds a filter.
    pub fn filter(mut self, filter: Box<Filter>) -> AppenderBuilder {
        self.filters.push(filter);
        self
    }

    /// Adds filters.
    pub fn filters<I>(mut self, filters: I) -> AppenderBuilder
        where I: IntoIterator<Item = Box<Filter>>
    {
        self.filters.extend(filters);
        self
    }

    /// Consumes the `AppenderBuilder`, returning the `Appender`.
    pub fn build<T>(self, name: T, appender: Box<Append>) -> Appender
        where T: Into<String>
    {
        Appender {
            name: name.into(),
            appender: appender,
            filters: self.filters,
        }
    }
}

/// Configuration for a logger.
#[derive(Debug)]
pub struct Logger {
    name: String,
    level: LevelFilter,
    appenders: Vec<String>,
    additive: bool,
}

impl Logger {
    /// Creates a new `LoggerBuilder` with the specified name and level.
    ///
    /// There are initially no appenders attached and `additive` is `true`.
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder {
            appenders: vec![],
            additive: true,
        }
    }

    /// Returns the name of the logger.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the minimum level of log messages that the logger will accept.
    pub fn level(&self) -> LevelFilter {
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
pub struct LoggerBuilder {
    appenders: Vec<String>,
    additive: bool,
}

impl LoggerBuilder {
    /// Adds an appender.
    pub fn appender<T>(mut self, appender: T) -> LoggerBuilder
        where T: Into<String>
    {
        self.appenders.push(appender.into());
        self
    }

    /// Adds appenders.
    pub fn appenders<I>(mut self, appenders: I) -> LoggerBuilder
        where I: IntoIterator,
              I::Item: Into<String>
    {
        self.appenders.extend(appenders.into_iter().map(Into::into));
        self
    }

    /// Sets the additivity of the logger.
    pub fn additive(mut self, additive: bool) -> LoggerBuilder {
        self.additive = additive;
        self
    }

    /// Consumes the `LoggerBuilder`, returning the `Logger`.
    pub fn build<T>(self, name: T, level: LevelFilter) -> Logger
        where T: Into<String>
    {
        Logger {
            name: name.into(),
            level: level,
            appenders: self.appenders,
            additive: self.additive,
        }
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
    /// Creates a new `ConfigBuilder`.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder {
            appenders: vec![],
            loggers: vec![],
        }
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
pub struct ConfigBuilder {
    appenders: Vec<Appender>,
    loggers: Vec<Logger>,
}

impl ConfigBuilder {
    /// Adds an appender.
    pub fn appender(mut self, appender: Appender) -> ConfigBuilder {
        self.appenders.push(appender);
        self
    }

    /// Adds appenders.
    pub fn appenders<I>(mut self, appenders: I) -> ConfigBuilder
        where I: IntoIterator<Item = Appender>
    {
        self.appenders.extend(appenders);
        self
    }

    /// Adds a logger.
    pub fn logger(mut self, logger: Logger) -> ConfigBuilder {
        self.loggers.push(logger);
        self
    }

    /// Adds loggers.
    pub fn loggers<I>(mut self, loggers: I) -> ConfigBuilder
        where I: IntoIterator<Item = Logger>
    {
        self.loggers.extend(loggers);
        self
    }

    /// Consumes the `ConfigBuilder`, returning the `Config`.
    ///
    /// Unlike `build`, this method will always return a `Config` by stripping
    /// portions of the configuration that are incorrect.
    pub fn build_lossy(self, mut root: Root) -> (Config, Vec<Error>) {
        let mut errors = vec![];

        let ConfigBuilder { appenders, loggers } = self;

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

        (config, errors)
    }

    /// Consumes the `ConfigBuilder`, returning the `Config`.
    pub fn build(self, root: Root) -> Result<Config, Errors> {
        let (config, errors) = self.build_lossy(root);
        if errors.is_empty() {
            Ok(config)
        } else {
            Err(Errors {
                errors: errors,
            })
        }
    }
}

fn check_logger_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(Error::InvalidLoggerName(name.to_owned()));
    }

    let mut streak = 0;
    for ch in name.chars() {
        if ch == ':' {
            streak += 1;
            if streak > 2 {
                return Err(Error::InvalidLoggerName(name.to_owned()));
            }
        } else {
            if streak > 0 && streak != 2 {
                return Err(Error::InvalidLoggerName(name.to_owned()));
            }
            streak = 0;
        }
    }

    if streak > 0 {
        Err(Error::InvalidLoggerName(name.to_owned()))
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
            writeln!(fmt, "{}", error)?;
        }
        Ok(())
    }
}

impl error::Error for Errors {
    fn description(&self) -> &str {
        "Errors encountered when validating a log4rs `Config`"
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
    #[doc(hidden)]
    __Extensible,
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
            Error::__Extensible => unreachable!(),
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
        let tests = [("", false),
                     ("asdf", true),
                     ("asdf::jkl", true),
                     ("::", false),
                     ("asdf::jkl::", false),
                     ("asdf:jkl", false),
                     ("asdf:::jkl", false),
                     ("asdf::jkl::", false)];

        for &(ref name, expected) in &tests {
            assert!(expected == super::check_logger_name(name).is_ok(),
                    "{}",
                    name);
        }
    }
}
