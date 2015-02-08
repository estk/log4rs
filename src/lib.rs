//! log4rs is a highly configurable logging framework for Rust integrating with
//! the `log` logging facade.
//!
//! The log4rs framework consists of a number of loggers as well as directies
//! which configure which log messages are directed to whic loggers.
//! Configuration is handled via a TOML file:
//!
//! ```toml
//! # log4rs will automatically check the configuration file for updates at the
//! # specified frequency in seconds if this is set
//! refresh_rate = 30
//!
//! # A minimal configuration of a logger called "stderr" of the "console" kind
//! [logger.stderr]
//! kind = "console"
//!
//! # A logger called "errors" with a more advanced configuration.
//! [logger.errors]
//! kind = "file"
//! path = "logs/errors.log"
//! pattern = "%d [%l] %T - %m"
//!
//! # A directive that routes all error level log messages to the errors logger
//! [[directive]]
//! level = "error"
//! logger = "errors"
//!
//! # A directive that routes all error, warning, and info level messages from
//! # the foo::bar::baz module to the stderr logger.
//! [[directive]]
//! level = "info"
//! logger = "stderr"
//! path = "foo::bar::baz"
//! ```
//!
//! The `init` method should be called early in the runtime of a program to
//! initialize the logging system. It takes a path to the config file as well
//! as a map of available loggers. The `default_loggers` function returns a map
//! of loggers that are provided with log4rs, including the `console` and `file`
//! kinds mentioned above:
//!
//! ```rust
//! # #![allow(unstable)]
//! extern crate log4rs;
//!
//! fn main() {
//!     log4rs::init(Path::new("log.toml"), log4rs::default_loggers()).unwrap();
//!
//!     // ...
//! }
//! ```
#![doc(html_root_url="https://sfackler.github.io/log4rs/doc")]
#![warn(missing_docs)]
#![feature(std_misc, core, io, path)]
#![allow(missing_copy_implementations)]

extern crate log;
extern crate toml;
extern crate time;

use std::borrow::ToOwned;
use std::collections::{HashMap, BTreeMap};
use std::default::Default;
use std::i64;
use std::old_io::fs::{self, File};
use std::old_io::timer;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

use log::{MaxLogLevelFilter, LogLevel, LogLevelFilter, LogRecord, LogLocation, SetLoggerError};
use log::Log as log_Log;

use config::RawConfig;

mod config;
pub mod loggers;
pub mod pattern;

/// A trait specifying the interface for log4rs loggers.
pub trait Log: Send {
    /// Logs the provided log record.
    fn log(&mut self, record: &LogRecord);
}

/// A trait specifying the interface for creation of log4rs loggers.
pub trait MakeLogger: Send+Sync {
    /// Creates a new log4rs logger, given the TOML table from the configuration
    /// file.
    fn make_logger(&self, config: &toml::Table) -> Result<Box<Log>, String>;
}

struct LogDirective {
    path: String,
    level: LogLevelFilter,
}

struct ConfiguredLogger {
    logger: Box<Log>,
    directives: Vec<LogDirective>,
    max_log_level: LogLevelFilter,
}

impl ConfiguredLogger {
    fn enabled(&self, level: LogLevel, module: &str) -> bool {
        if level > self.max_log_level {
            return false;
        }

        for directive in self.directives.iter().rev() {
            if module.starts_with(&*directive.path) {
                return level <= directive.level;
            }
        }

        false
    }
}

struct Config {
    refresh_rate: Option<Duration>,
    loggers: Vec<ConfiguredLogger>,
}

impl Default for Config {
    fn default() -> Config {
        let directive = LogDirective {
            path: "".to_owned(),
            level: LogLevelFilter::Warn,
        };

        let logger = ConfiguredLogger {
            logger: loggers::ConsoleLoggerMaker.make_logger(&BTreeMap::new()).unwrap(),
            directives: vec![directive],
            max_log_level: LogLevelFilter::Warn,
        };

        Config {
            refresh_rate: Some(Duration::minutes(1)),
            loggers: vec![logger],
        }
    }
}

impl Config {
    fn from_raw(raw: RawConfig, make_loggers: &HashMap<String, Box<MakeLogger>>)
                -> Result<Config, Vec<String>> {
        let mut errors = vec![];

        let RawConfig {
            refresh_rate,
            loggers: raw_loggers,
            directives: raw_directives,
        } = raw;

        let mut loggers = raw_loggers.into_iter().filter_map(|(name, spec)| {
            let logger = make_loggers.get(&spec.kind)
                .ok_or(format!("Unknown logger kind `{}`", name))
                .and_then(|maker| maker.make_logger(&spec.config))
                .map(|logger| {
                    ConfiguredLogger {
                        logger: logger,
                        directives: vec![],
                        max_log_level: LogLevelFilter::Off,
                    }
                });
            match logger {
                Ok(logger) => Some((name, logger)),
                Err(err) => {
                    errors.push(err);
                    None
                }
            }
        }).collect::<HashMap<String, ConfiguredLogger>>();

        for directive in raw_directives.iter() {
            for logger in directive.loggers.iter() {
                let logger = if let Some(logger) = loggers.get_mut(&*logger) {
                    logger
                } else {
                    errors.push(format!("Unknown logger `{}`", logger));
                    continue;
                };

                logger.directives.push(LogDirective {
                    path: directive.path.clone(),
                    level: directive.level,
                });
            }
        }

        let mut loggers = loggers.into_iter().map(|e| e.1).collect::<Vec<_>>();
        for logger in loggers.iter_mut() {
            logger.directives.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
            logger.max_log_level = logger.directives.iter()
                .map(|d| d.level)
                .max()
                .unwrap_or(LogLevelFilter::Off);
        }

        if errors.is_empty() {
            Ok(Config {
                refresh_rate: refresh_rate,
                loggers: loggers,
            })
        } else {
            Err(errors)
        }
    }

    fn max_log_level(&self) -> LogLevelFilter {
        self.loggers.iter().map(|logger| logger.max_log_level).max().unwrap_or(LogLevelFilter::Off)
    }

    fn enabled(&self, level: LogLevel, module: &str) -> bool {
        self.loggers.iter().any(|logger| logger.enabled(level, module))
    }

    fn log(&mut self, record: &LogRecord) {
        for logger in self.loggers.iter_mut() {
            if logger.enabled(record.level(), record.location().module_path) {
                logger.logger.log(record)
            }
        }
    }
}

struct InnerLogger {
    config: Config,
}

struct Logger {
    max_log_level: MaxLogLevelFilter,
    config_file: Path,
    loggers: HashMap<String, Box<MakeLogger>>,
    inner: Mutex<InnerLogger>,
}

impl log::Log for Arc<Logger> {
    fn enabled(&self, level: LogLevel, module: &str) -> bool {
        self.inner.lock().unwrap().config.enabled(level, module)
    }

    fn log(&self, record: &log::LogRecord) {
        self.inner.lock().unwrap().config.log(record)
    }
}

impl Logger {
    fn reload_config(&self) {
        let config = File::open(&self.config_file)
            .and_then(|mut f| f.read_to_string())
            .map_err(|e| vec![e.to_string()])
            .and_then(|c| config::parse_config(&*c))
            .and_then(|c| Config::from_raw(c, &self.loggers));

        match config {
            Ok(config) => {
                let max_log_level = config.max_log_level();
                self.inner.lock().unwrap().config = config;
                self.max_log_level.set(max_log_level);
            }
            Err(errs) => self.log_errors(&*errs)
        }
    }

    fn log_errors(&self, errors: &[String]) {
        let mut inner = self.inner.lock().unwrap();

        for err in errors.iter() {
            // we can't use log! since this may happen before the logger's registered
            static LOC: LogLocation = LogLocation {
                line: line!(),
                file: file!(),
                module_path: module_path!(),
            };
            inner.config.log(&LogRecord::new(
                    LogLevel::Error,
                    &LOC,
                    format_args!("Error loading config \"{}\": {}",
                                   self.config_file.display(), err)))
        }
    }
}

fn load_config(logger: &Arc<Logger>) {
    logger.reload_config();

    if logger.inner.lock().unwrap().config.refresh_rate.is_none() {
        return;
    }

    let logger: Arc<_> = logger.clone();
    thread::Builder::new().name("log4rs config refresh thread".to_owned()).spawn(move || {
        let mut last_check = time::precise_time_ns();

        let mut last_mtime = match fs::stat(&logger.config_file) {
            Ok(stat) => Some(stat.modified),
            Err(err) => {
                logger.log_errors(&[err.to_string()]);
                None
            }
        };

        loop {
            let refresh_rate = match logger.inner.lock().unwrap().config.refresh_rate {
                Some(rate) => rate,
                None => break,
            };

            let stop = last_check + refresh_rate.num_nanoseconds().unwrap_or(i64::MAX) as u64;
            loop {
                let now = time::precise_time_ns();
                let delta = Duration::nanoseconds((stop - now) as i64);
                if delta <= Duration::zero() {
                    break;
                }
                timer::sleep(delta);
            }

            last_check = time::precise_time_ns();
            let mtime = match fs::stat(&logger.config_file) {
                Ok(stat) => stat.modified,
                Err(err) => {
                    logger.log_errors(&[err.to_string()]);
                    continue;
                }
            };

            match (last_mtime, mtime) {
                (None, _) => logger.reload_config(),
                (Some(old), new) if new != old => logger.reload_config(),
                _ => {}
            }

            last_mtime = Some(mtime);
        }
    });
}

/// Returns a set of built-in log4rs loggers.
///
/// This includes `console` and `file` loggers.
pub fn default_loggers() -> HashMap<String, Box<MakeLogger>> {
    let mut loggers = HashMap::new();
    loggers.insert("console".to_owned(), Box::new(loggers::ConsoleLoggerMaker) as Box<MakeLogger>);
    loggers.insert("file".to_owned(), Box::new(loggers::FileLoggerMaker) as Box<MakeLogger>);
    loggers
}

/// Initializes the global logger with a log4rs logger.
///
/// Configuration is read from a TOML file located at the provided path on the
/// filesystem, and log4rs loggers are created from the provided set of logger
/// makers.
///
/// Any errors encountered when processing the configuration are reported to
/// stderr.
pub fn init(config_file: Path, loggers: HashMap<String, Box<MakeLogger>>)
        -> Result<(), SetLoggerError> {
    log::set_logger(move |max_log_level| {
        let inner = InnerLogger {
            config: Default::default(),
        };
        max_log_level.set(inner.config.max_log_level());

        let logger = Arc::new(Logger {
            max_log_level: max_log_level,
            config_file: config_file,
            loggers: loggers,
            inner: Mutex::new(inner),
        });

        load_config(&logger);

        Box::new(logger)
    })
}

#[cfg(test)]
mod test {
    use super::{config, Config, default_loggers};
    use log::LogLevel;

    #[test]
    fn test_level_overrides() {
        let cfg = r#"
[logger.console]
kind = "console"

[[directive]]
path = "foo::bar::baz"
level = "warn"
logger = "console"

[[directive]]
path = "foo::bar"
level = "debug"
logger = "console"
"#;
        let cfg = Config::from_raw(config::parse_config(cfg).ok().unwrap(),
                                   &default_loggers()).ok().unwrap();

        assert!(!cfg.enabled(LogLevel::Debug, "foo"));
        assert!(cfg.enabled(LogLevel::Debug, "foo::bar"));
        assert!(cfg.enabled(LogLevel::Debug, "foo::bar::thing"));
        assert!(!cfg.enabled(LogLevel::Debug, "foo::bar::baz"));
        assert!(!cfg.enabled(LogLevel::Debug, "foo::bar::baz::buz"));
    }
}
