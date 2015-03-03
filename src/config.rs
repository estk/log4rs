use std::collections::HashSet;
use std::fmt;
use log::LogLevelFilter;

use Append;

#[derive(Debug)]
pub struct Root {
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
    _p: (),
}

impl Root {
    pub fn new(level: LogLevelFilter) -> Root {
        Root {
            level: level,
            appenders: vec![],
            _p: (),
        }
    }
}

pub struct Appender {
    pub name: String,
    pub appender: Box<Append>,
    _p: (),
}

impl fmt::Debug for Appender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Appender {{ name: {} }}", self.name)
    }
}

impl Appender {
    pub fn new(name: String, appender: Box<Append>) -> Appender {
        Appender {
            name: name,
            appender: appender,
            _p: (),
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    pub name: String,
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
    pub additive: bool,
    _p: (),
}

impl Logger {
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

pub struct Config {
    pub appenders: Vec<Appender>,
    pub root: Root,
    pub loggers: Vec<Logger>,
    _p: (),
}

impl Config {
    pub fn new(appenders: Vec<Appender>, root: Root, loggers: Vec<Logger>)
               -> Result<Config, Error> {
        {
            let mut appender_names = HashSet::new();

            for appender in &appenders {
                if !appender_names.insert(&appender.name) {
                    return Err(Error);
                }
            }

            for appender in &root.appenders {
                if !appender_names.contains(&appender) {
                    return Err(Error);
                }
            }

            let mut logger_names = HashSet::new();
            for logger in &loggers {
                if !logger_names.insert(&logger.name) {
                    return Err(Error);
                }
                try!(Config::check_logger_name(&logger.name));

                for appender in &logger.appenders {
                    if !appender_names.contains(&appender) {
                        return Err(Error);
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
            return Err(Error);
        }

        let mut streak = 0;
        for ch in name.chars() {
            if ch != ':' {
                if streak > 0 && streak != 2 {
                    return Err(Error);
                }
                streak = 0;
            } else {
                streak += 1;
                if streak > 2 {
                    return Err(Error);
                }
            }
        }

        if streak > 0 {
            Err(Error)
        } else {
            Ok(())
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Error;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_logger_name() {
        let tests = [
            ("", Err(Error)),
            ("asdf", Ok(())),
            ("asdf::jkl", Ok(())),
            ("::", Err(Error)),
            ("asdf::jkl::", Err(Error)),
            ("asdf:jkl", Err(Error)),
            ("asdf:::jkl", Err(Error)),
            ("asdf::jkl::", Err(Error)),
        ];

        for &(ref name, ref expected) in &tests {
            assert!(expected == &Config::check_logger_name(name), "{}", name);
        }
    }
}
