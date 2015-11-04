//! log4rs is a highly configurable logging framework modeled after Java's
//! Logback and log4j libraries.
//!
//! # Architecture
//!
//! The basic units of configuration are *appenders*, *filters*, and *loggers*.
//!
//! ## Appenders
//!
//! An appender takes a log record and logs it somewhere, for example, to a
//! file, the console, or the syslog.
//!
//! ## Filters
//!
//! Filters are associated with appenders and, like the name would suggest,
//! filter log events coming into that appender.
//!
//! ## Loggers
//!
//! A log event is targeted at a specific logger, which are identified by
//! string names. The logging macros built in to the `log` crate set the logger
//! of a log event to the one identified by the module containing the
//! invocation location.
//!
//! Loggers form a heirarchy: logger names are divided into components by "::".
//! One logger is the ancestor of another if the first logger's component list
//! is a prefix of the second logger's component list.
//!
//! Loggers are associated with a maximum log level. Log events for that logger
//! with a level above the maximum will be ignored. The maximum log level for
//! any logger can be configured manually; if it is not, the level will be
//! inherited from the logger's parent.
//!
//! Loggers are also associated with a set of appenders. Appenders can be
//! associated directly with a logger. In addition, the appenders of the
//! logger's parent will be associated with the logger unless the logger has
//! its *additivity* set to `false`. Log events sent to the logger that are not
//! filtered out by the logger's maximum log level will be sent to all
//! associated appenders.
//!
//! The "root" logger is the ancestor of all other logger. Since it has no
//! ancestors, its additivity cannot be configured.
//!
//! # Configuration
//!
//! log4rs can be configured either programmatically by using the builders in
//! the `config` module to construct a log4rs `Config` object, which can be
//! passed to the `init_config` function.
//!
//! The more common configuration method, however, is via a separate TOML
//! config file. The `init_file` function takes the path to a config file as
//! well as a `Creator` object which is responsible for instantiating the
//! various objects specified by the config file. The `toml` module
//! documentation covers the exact configuration syntax, but an example is
//! provided below.
//!
//! # Examples
//!
//! ```toml
//! # Scan this file for changes every 30 seconds
//! refresh_rate = 30
//!
//! # An appender named "stdout" that writes to stdout
//! [appender.stdout]
//! kind = "console"
//!
//! # An appender named "requests" that writes to a file with a custom pattern
//! [appender.requests]
//! kind = "file"
//! path = "log/requests.log"
//! pattern = "%d - %m"
//!
//! # Set the default logging level to "warn" and attach the "stdout" appender to the root
//! [root]
//! level = "warn"
//! appenders = ["stdout"]
//!
//! # Raise the maximum log level for events sent to the "app::backend::db" logger to "info"
//! [[logger]]
//! name = "app::backend::db"
//! level = "info"
//!
//! # Route log events sent to the "app::requests" logger to the "requests" appender,
//! # and *not* the normal appenders installed at the root
//! [[logger]]
//! name = "app::requests"
//! level = "info"
//! appenders = ["requests"]
//! additive = false
//! ```
#![doc(html_root_url="https://sfackler.github.io/log4rs/doc/v0.3.3")]
#![warn(missing_docs)]

extern crate log;
extern crate time;
extern crate toml as toml_parser;
extern crate term;

use std::borrow::ToOwned;
use std::convert::AsRef;
use std::cmp;
use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Arc};
use std::thread;
use time::Duration;
use log::{LogLevel, LogMetadata, LogRecord, LogLevelFilter, SetLoggerError, MaxLogLevelFilter};

use toml::Creator;

pub mod appender;
pub mod config;
pub mod filter;
pub mod pattern;
pub mod toml;

/// A trait implemented by log4rs appenders.
pub trait Append: Send + 'static {
    /// Processes the provided `LogRecord`.
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<error::Error>>;
}

/// The response returned by a filter.
pub enum FilterResponse {
    /// Accept the log event.
    ///
    /// It will be immediately passed to the appender, bypassing any remaining
    /// filters.
    Accept,

    /// Take no action on the log event.
    ///
    /// It will continue on to remaining filters or pass on to the appender if
    /// there are none remaining.
    Neutral,

    /// Reject the log event.
    Reject,
}

/// The trait implemented by log4rs filters.
pub trait Filter: Send + 'static {
    /// Filters a log event.
    fn filter(&mut self, record: &LogRecord) -> FilterResponse;
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

    fn log(&self, record: &log::LogRecord, appenders: &mut [Appender]) {
        if self.enabled(record.level()) {
            for &idx in &self.appenders {
                if let Err(err) = appenders[idx].append(record) {
                    handle_error(&*err);
                }
            }
        }
    }
}

struct Appender {
    appender: Box<Append>,
    filters: Vec<Box<Filter>>,
}

impl Appender {
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<error::Error>> {
        for filter in &mut self.filters {
            match filter.filter(record) {
                FilterResponse::Accept => break,
                FilterResponse::Neutral => {}
                FilterResponse::Reject => return Ok(()),
            }
        }

        self.appender.append(record)
    }
}

struct SharedLogger {
    root: ConfiguredLogger,
    appenders: Vec<Appender>,
}

impl SharedLogger {
    fn new(config: config::Config) -> SharedLogger {
        let (appenders, root, loggers) = config.unpack();

        let root = {
            let appender_map = appenders
                .iter()
                .enumerate()
                .map(|(i, appender)| (appender.name(), i))
                .collect::<HashMap<_, _>>();

            let mut root = ConfiguredLogger {
                level: root.level(),
                appenders: root.appenders()
                    .iter()
                    .map(|appender| appender_map[&**appender])
                    .collect(),
                children: vec![],
            };

            for logger in loggers {
                let appenders = logger.appenders()
                    .iter()
                    .map(|appender| appender_map[&**appender])
                    .collect();
                root.add(logger.name(), appenders, logger.additive(), logger.level());
            }

            root
        };

        let appenders = appenders.into_iter().map(|appender| {
            let (_, appender, filters) = appender.unpack();
            Appender {
                appender: appender,
                filters: filters,
            }
        }).collect();

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
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        self.enabled_inner(metadata.level(), metadata.target())
    }

    fn log(&self, record: &log::LogRecord) {
        let shared = &mut *self.inner.lock().unwrap();
        shared.root.find(record.target()).log(record, &mut shared.appenders);
    }
}

impl Logger {
    fn enabled_inner(&self, level: LogLevel, target: &str) -> bool {
        self.inner.lock().unwrap().root.find(target).enabled(level)
    }
}

fn handle_error<E: error::Error+?Sized>(e: &E) {
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    let _ = writeln!(&mut stderr, "{}", e);
}

/// Initializes the global logger with a log4rs logger configured by `config`.
pub fn init_config(config: config::Config) -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        let logger = Logger::new(config);
        max_log_level.set(logger.max_log_level());
        Box::new(logger)
    })
}

/// Initializes the global logger with a log4rs logger.
///
/// Configuration is read from a TOML file located at the provided path on the
/// filesystem and appenders are created from the provided `Creator`.
///
/// Any errors encountered when processing the configuration are reported to
/// stderr.
pub fn init_file<P: AsRef<Path>>(path: P, creator: Creator) -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        let path = path.as_ref().to_path_buf();
        let (source, refresh_rate, config) = match read_config(&path) {
            Ok(source) => {
                match parse_config(&source, &creator) {
                    Ok(config) => {
                        let (refresh_rate, config) = config.unpack();
                        (source, refresh_rate, config)
                    }
                    Err(err) => {
                        handle_error(&*err);
                        ("".to_string(), None, config::Config::builder(
                                config::Root::builder(LogLevelFilter::Off).build()).build().unwrap())
                    }
                }
            },
            Err(err) => {
                handle_error(&err);
                ("".to_string(), None, config::Config::builder(
                        config::Root::builder(LogLevelFilter::Off).build()).build().unwrap())
            }
        };
        let logger = Logger::new(config);
        max_log_level.set(logger.max_log_level());
        if let Some(refresh_rate) = refresh_rate {
            ConfigReloader::start(path, refresh_rate, source, creator, &logger, max_log_level);
        }
        Box::new(logger)
    })
}

fn read_config(path: &Path) -> Result<String, io::Error> {
    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

fn parse_config(source: &str, creator: &Creator) -> Result<toml::Config, Box<error::Error>> {
    let (config, errors) = try!(toml::Config::parse(&source, creator));
    if let Err(errors) = errors {
        for error in errors.errors() {
            handle_error(error);
        }
    }
    Ok(config)
}

struct ConfigReloader {
    path: PathBuf,
    rate: Duration,
    source: String,
    creator: Creator,
    shared: Arc<Mutex<SharedLogger>>,
    max_log_level: MaxLogLevelFilter,
}

impl ConfigReloader {
    fn start(path: PathBuf, rate: Duration, source: String, creator: Creator, logger: &Logger,
             max_log_level: MaxLogLevelFilter) {
        let mut reloader = ConfigReloader {
            path: path,
            rate: rate,
            source: source,
            creator: creator,
            shared: logger.inner.clone(),
            max_log_level: max_log_level,
        };

        thread::Builder::new()
            .name("log4rs config refresh thread".to_string())
            .spawn(move || reloader.run())
            .unwrap();
    }

    fn run(&mut self) {
        loop {
            thread::sleep_ms(self.rate.num_milliseconds() as u32);

            let source = match read_config(&self.path) {
                Ok(source) => source,
                Err(err) => {
                    handle_error(&err);
                    continue;
                }
            };

            if source == self.source {
                continue;
            }

            self.source = source;

            let config = match parse_config(&self.source, &self.creator) {
                Ok(config) => config,
                Err(err) => {
                    handle_error(&*err);
                    continue;
                }
            };
            let (refresh_rate, config) = config.unpack();

            let shared = SharedLogger::new(config);
            self.max_log_level.set(shared.root.max_log_level());
            *self.shared.lock().unwrap() = shared;

            match refresh_rate {
                Some(rate) => self.rate = rate,
                None => return,
            }
        }
    }
}

#[doc(hidden)]
trait ConfigPrivateExt {
    fn unpack(self) -> (Vec<config::Appender>, config::Root, Vec<config::Logger>);
}

#[doc(hidden)]
trait PrivateTomlConfigExt {
    fn unpack(self) -> (Option<Duration>, config::Config);
}

#[doc(hidden)]
trait PrivateConfigErrorsExt {
    fn unpack(self) -> Vec<config::Error>;
}

#[doc(hidden)]
trait PrivateConfigAppenderExt {
    fn unpack(self) -> (String, Box<Append>, Vec<Box<Filter>>);
}

#[cfg(test)]
mod test {
    use log::{LogLevel, LogLevelFilter};

    use super::*;

    #[test]
    fn enabled() {
        let root = config::Root::builder(LogLevelFilter::Debug).build();
        let mut config = config::Config::builder(root);
        let logger = config::Logger::builder("foo::bar".to_string(), LogLevelFilter::Trace).build();
        config = config.logger(logger);
        let logger = config::Logger::builder("foo::bar::baz".to_string(), LogLevelFilter::Off).build();
        config = config.logger(logger);
        let logger = config::Logger::builder("foo::baz::buz".to_string(), LogLevelFilter::Error).build();
        config = config.logger(logger);
        let config = config.build().unwrap();

        let logger = super::Logger::new(config);

        assert!(logger.enabled_inner(LogLevel::Warn, "bar"));
        assert!(!logger.enabled_inner(LogLevel::Trace, "bar"));
        assert!(logger.enabled_inner(LogLevel::Debug, "foo"));
        assert!(logger.enabled_inner(LogLevel::Trace, "foo::bar"));
        assert!(!logger.enabled_inner(LogLevel::Error, "foo::bar::baz"));
        assert!(logger.enabled_inner(LogLevel::Debug, "foo::bar::bazbuz"));
        assert!(!logger.enabled_inner(LogLevel::Error, "foo::bar::baz::buz"));
        assert!(!logger.enabled_inner(LogLevel::Warn, "foo::baz::buz"));
        assert!(!logger.enabled_inner(LogLevel::Warn, "foo::baz::buz::bar"));
        assert!(logger.enabled_inner(LogLevel::Error, "foo::baz::buz::bar"));
    }
}
