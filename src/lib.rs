#![feature(std_misc, fs, io, path, core)]

extern crate log;
extern crate time;
extern crate "toml" as toml_parser;

use std::borrow::ToOwned;
use std::cmp;
use std::collections::HashMap;
use std::error;
use std::sync::{Mutex, Arc};
use log::{LogLevel, LogRecord, LogLevelFilter, SetLoggerError};

pub mod toml;
pub mod config;
pub mod appender;
pub mod pattern;

pub trait Append: Send + 'static{
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<error::Error>>;
}

struct ConfiguredLogger {
    level: LogLevelFilter,
    appenders: Vec<usize>,
    children: Vec<(String, Box<ConfiguredLogger>)>,
}

impl ConfiguredLogger {
    fn add(&mut self, path: &str, mut appenders: Vec<usize>, additive: bool, level: LogLevelFilter) {
        let (part, rest) = match path.find("::") {
            Some(idx) => (&path[..idx], &path[idx+2..]),
            None => (path, ""),
        };

        for &mut (ref child_part, ref mut child) in &mut self.children {
            if &child_part[..] == part {
                child.add(rest, appenders, additive, level);
                return;
            }
        }

        let child = if rest.is_empty() {
            if additive {
                appenders.extend(self.appenders.iter().cloned());
            }

            ConfiguredLogger {
                level: level,
                appenders: appenders,
                children: vec![],
            }
        } else {
            let mut child = ConfiguredLogger {
                level: self.level,
                appenders: self.appenders.clone(),
                children: vec![],
            };
            child.add(rest, appenders, additive, level);
            child
        };

        self.children.push((part.to_owned(), Box::new(child)));
    }

    fn max_log_level(&self) -> LogLevelFilter {
        let mut max = self.level;
        for &(_, ref child) in &self.children {
            max = cmp::max(max, child.max_log_level());
        }
        max
    }

    fn find(&self, path: &str) -> &ConfiguredLogger {
        let mut node = self;

        'parts: for part in path.split("::") {
            for &(ref child_part, ref child) in &node.children {
                if &child_part[..] == part {
                    node = child;
                    continue 'parts;
                }
            }

            break;
        }

        node
    }

    fn enabled(&self, level: LogLevel) -> bool {
        self.level >= level
    }

    fn log(&self, record: &log::LogRecord, appenders: &mut [Box<Append>]) {
        if self.enabled(record.level()) {
            for &idx in &self.appenders {
                let _ = appenders[idx].append(record);
            }
        }
    }
}

struct SharedLogger {
    root: ConfiguredLogger,
    appenders: Vec<Box<Append>>,
}

impl SharedLogger {
    fn new(config: config::Config) -> SharedLogger {
        let config::Config { appenders, root, loggers, .. } = config;

        let root = {
            let appender_map = appenders
                .iter()
                .enumerate()
                .map(|(i, appender)| {
                    (&appender.name, i)
                })
                .collect::<HashMap<_, _>>();

            let config::Root { level, appenders, .. } = root;
            let mut root = ConfiguredLogger {
                level: level,
                appenders: appenders
                    .into_iter()
                    .map(|appender| appender_map[appender].clone())
                    .collect(),
                children: vec![],
            };

            for logger in loggers {
                let appenders = logger.appenders
                    .into_iter()
                    .map(|appender| appender_map[appender])
                    .collect();
                root.add(&logger.name, appenders, logger.additive, logger.level);
            }

            root
        };

        let appenders = appenders.into_iter().map(|appender| appender.appender).collect();

        SharedLogger {
            root: root,
            appenders: appenders,
        }
    }
}

struct Logger {
    inner: Arc<Mutex<SharedLogger>>,
}

impl Logger {
    fn new(config: config::Config) -> Logger {
        Logger {
            inner: Arc::new(Mutex::new(SharedLogger::new(config)))
        }
    }

    fn max_log_level(&self) -> LogLevelFilter {
        self.inner.lock().unwrap().root.max_log_level()
    }
}

impl log::Log for Logger {
    fn enabled(&self, level: LogLevel, module: &str) -> bool {
        self.inner.lock().unwrap().root.find(module).enabled(level)
    }

    fn log(&self, record: &log::LogRecord) {
        let shared = &mut *self.inner.lock().unwrap();
        shared.root.find(record.location().module_path).log(record, &mut shared.appenders);
    }
}

pub fn init_config(config: config::Config) -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        let logger = Logger::new(config);
        max_log_level.set(logger.max_log_level());
        Box::new(logger)
    })
}

#[cfg(test)]
mod test {
    use log::{LogLevel, LogLevelFilter, Log};

    use super::*;

    #[test]
    fn enabled() {
        let appenders = vec![];
        let root = config::Root::new(LogLevelFilter::Debug);
        let loggers = vec![
            config::Logger::new("foo::bar".to_string(), LogLevelFilter::Trace),
            config::Logger::new("foo::bar::baz".to_string(), LogLevelFilter::Off),
            config::Logger::new("foo::baz::buz".to_string(), LogLevelFilter::Error),
        ];
        let config = config::Config::new(appenders, root, loggers).unwrap();

        let logger = super::Logger::new(config);

        assert!(logger.enabled(LogLevel::Warn, "bar"));
        assert!(!logger.enabled(LogLevel::Trace, "bar"));
        assert!(logger.enabled(LogLevel::Debug, "foo"));
        assert!(logger.enabled(LogLevel::Trace, "foo::bar"));
        assert!(!logger.enabled(LogLevel::Error, "foo::bar::baz"));
        assert!(logger.enabled(LogLevel::Debug, "foo::bar::bazbuz"));
        assert!(!logger.enabled(LogLevel::Error, "foo::bar::baz::buz"));
        assert!(!logger.enabled(LogLevel::Warn, "foo::baz::buz"));
        assert!(!logger.enabled(LogLevel::Warn, "foo::baz::buz::bar"));
        assert!(logger.enabled(LogLevel::Error, "foo::baz::buz::bar"));
    }
}
