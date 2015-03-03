//! log4rs configuration

use std::collections::HashSet;
use std::fmt;
use std::error;
use log::LogLevelFilter;

use Append;

/// Configuration for the root logger.
#[derive(Debug)]
pub struct Root {
    /// The minimum level of log messages that the root logger will accept.
    pub level: LogLevelFilter,
    /// The set of names of appenders that will be attached to the root logger.
    pub appenders: Vec<String>,
    _p: (),
}

impl Root {
    /// Creates a new `Root` with no appenders and the specified level.
    pub fn new(level: LogLevelFilter) -> Root {
        Root {
            level: level,
            appenders: vec![],
            _p: (),
        }
    }
}

/// Configuration for an appender.
pub struct Appender {
    /// The name of the appender.
    pub name: String,
    /// The appender trait object.
    pub appender: Box<Append>,
    _p: (),
}

impl fmt::Debug for Appender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Appender {{ name: {} }}", self.name)
    }
}

impl Appender {
    /// Creates a new `Appender` with the specified name and `Append` trait object.
    pub fn new(name: String, appender: Box<Append>) -> Appender {
        Appender {
            name: name,
            appender: appender,
            _p: (),
        }
    }
}

/// Configuration for a logger.
#[derive(Debug)]
pub struct Logger {
    /// The name of the logger.
    pub name: String,
    /// The minimum level of log messages that the logger will accept.
    pub level: LogLevelFilter,
    /// The set of names of appenders that will be attached to the logger.
    pub appenders: Vec<String>,
    /// If `true`, appenders of parent loggers will also be attached to this logger.
    pub additive: bool,
    _p: (),
}

impl Logger {
    /// Creates a new `Logger` with the specified name and level.
    ///
    /// There are initially no appenders attached and `additive` is `true`.
    pub fn new(name: String, level: LogLevelFilter) -> Logger {
        Logger {
            name: name,
            level: level,
            appenders: vec![],
            additive: true,
            _p: (),
        }
    }
}

/// A log4rs configuration.
#[derive(Debug)]
pub struct Config {
    pub appenders: Vec<Appender>,
    pub root: Root,
    pub loggers: Vec<Logger>,
    _p: (),
}

impl Config {
    /// Creates a new `Config` from the provided appenders, root, and loggers.
    pub fn new(appenders: Vec<Appender>, root: Root, loggers: Vec<Logger>)
               -> Result<Config, Error> {
        {
            let mut appender_names = HashSet::new();

            for appender in &appenders {
                if !appender_names.insert(&appender.name) {
                    return Err(Error::DuplicateAppenderName(appender.name.clone()));
                }
            }

            for appender in &root.appenders {
                if !appender_names.contains(&appender) {
                    return Err(Error::NonexistentAppender(appender.clone()));
                }
            }

            let mut logger_names = HashSet::new();
            for logger in &loggers {
                if !logger_names.insert(&logger.name) {
                    return Err(Error::DuplicateLoggerName(logger.name.clone()));
                }
                try!(Config::check_logger_name(&logger.name));

                for appender in &logger.appenders {
                    if !appender_names.contains(&appender) {
                        return Err(Error::NonexistentAppender(appender.clone()));
                    }
                }
            }
        }

        Ok(Config {
            appenders: appenders,
            root: root,
            loggers: loggers,
            _p: (),
        })
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
}

#[derive(PartialEq, Debug)]
pub enum Error {
    DuplicateAppenderName(String),
    NonexistentAppender(String),
    DuplicateLoggerName(String),
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
    use super::*;

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

        for &(ref name, ref expected) in &tests {
            assert!(expected == &Config::check_logger_name(name).is_ok(), "{}", name);
        }
    }
}
