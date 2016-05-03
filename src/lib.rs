//! log4rs is a highly configurable logging framework modeled after Java's
//! Logback and log4j libraries.
//!
//! # Architecture
//!
//! The basic units of configuration are *appenders*, *encoders*, *filters*, and
//! *loggers*.
//!
//! ## Appenders
//!
//! An appender takes a log record and logs it somewhere, for example, to a
//! file, the console, or the syslog.
//!
//! ## Encoders
//!
//! An encoder is responsible for taking a log record, transforming it into the
//! appropriate output format, and writing it out. An appender will normally
//! use an encoder internally.
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
//! The "root" logger is the ancestor of all other loggers. Since it has no
//! ancestors, its additivity cannot be configured.
//!
//! # Configuration
//!
//! log4rs can be configured programmatically by using the builders in the
//! `config` module to construct a log4rs `Config` object, which can be passed
//! to the `init_config` function.
//!
//! The more common configuration method, however, is via a separate config
//! file. The `init_file` function takes the path to a config file as
//! well as a `Deserializers` object which is responsible for instantiating the
//! various objects specified by the config file. The `file` module
//! documentation covers the exact configuration syntax, but an example in the
//! YAML format is provided below.
//!
//! # Examples
//!
//! ```yaml
//! # Scan this file for changes every 30 seconds
//! refresh_rate: 30
//!
//! appenders:
//!   # An appender named "stdout" that writes to stdout
//!   stdout:
//!     kind: console
//!
//!   # An appender named "requests" that writes to a file with a custom pattern encoder
//!   requests:
//!     kind: file
//!     path: "log/requests.log"
//!     encoder:
//!       pattern: "{d} - {m}{n}"
//!
//! # Set the default logging level to "warn" and attach the "stdout" appender to the root
//! root:
//!   level: warn
//!   appenders:
//!     - stdout
//!
//! loggers:
//!   # Raise the maximum log level for events sent to the "app::backend::db" logger to "info"
//!   app::backend::db:
//!     level: info
//!
//!   # Route log events sent to the "app::requests" logger to the "requests" appender,
//!   # and *not* the normal appenders installed at the root
//!   app::requests:
//!     level: info
//!     appenders:
//!       - requests
//!     additive: false
//! ```
#![doc(html_root_url="https://sfackler.github.io/log4rs/doc/v0.4.3")]
#![warn(missing_docs)]

extern crate antidote;
extern crate chrono;
extern crate crossbeam;
extern crate kernel32;
extern crate libc;
extern crate log;
extern crate serde;
extern crate serde_value;
extern crate typemap;
extern crate winapi;
#[cfg(feature = "yaml")]
extern crate serde_yaml;
#[cfg(feature = "json")]
extern crate serde_json;
#[cfg(feature = "toml")]
extern crate toml;

use crossbeam::sync::ArcCell;
use std::cmp;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use log::{LogLevel, LogMetadata, LogRecord, LogLevelFilter, SetLoggerError, MaxLogLevelFilter};

use append::Append;
use config::Config;
use filter::Filter;
use file::{Format, Deserializers};

pub mod append;
pub mod config;
pub mod filter;
pub mod file;
pub mod encode;
mod priv_serde;

struct ConfiguredLogger {
    level: LogLevelFilter,
    appenders: Vec<usize>,
    children: Vec<(String, ConfiguredLogger)>,
}

impl ConfiguredLogger {
    fn add(&mut self,
           path: &str,
           mut appenders: Vec<usize>,
           additive: bool,
           level: LogLevelFilter) {
        let (part, rest) = match path.find("::") {
            Some(idx) => (&path[..idx], &path[idx + 2..]),
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

        self.children.push((part.to_owned(), child));
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

    fn log(&self, record: &log::LogRecord, appenders: &[Appender]) {
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
    fn append(&self, record: &LogRecord) -> Result<(), Box<error::Error>> {
        for filter in &self.filters {
            match filter.filter(record) {
                filter::Response::Accept => break,
                filter::Response::Neutral => {}
                filter::Response::Reject => return Ok(()),
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
        let (appenders, root, mut loggers) = config.unpack();

        let root = {
            let appender_map = appenders.iter()
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

            // sort loggers by name length to ensure that we initialize them top to bottom
            loggers.sort_by_key(|l| l.name().len());
            for logger in loggers {
                let appenders = logger.appenders()
                                      .iter()
                                      .map(|appender| appender_map[&**appender])
                                      .collect();
                root.add(logger.name(), appenders, logger.additive(), logger.level());
            }

            root
        };

        let appenders = appenders.into_iter()
                                 .map(|appender| {
                                     let (_, appender, filters) = appender.unpack();
                                     Appender {
                                         appender: appender,
                                         filters: filters,
                                     }
                                 })
                                 .collect();

        SharedLogger {
            root: root,
            appenders: appenders,
        }
    }
}

struct Logger(Arc<ArcCell<SharedLogger>>);

impl Logger {
    fn new(config: config::Config) -> Logger {
        Logger(Arc::new(ArcCell::new(Arc::new(SharedLogger::new(config)))))
    }

    fn max_log_level(&self) -> LogLevelFilter {
        self.0.get().root.max_log_level()
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        self.enabled_inner(metadata.level(), metadata.target())
    }

    fn log(&self, record: &log::LogRecord) {
        let shared = self.0.get();
        shared.root.find(record.target()).log(record, &shared.appenders);
    }
}

impl Logger {
    fn enabled_inner(&self, level: LogLevel, target: &str) -> bool {
        self.0.get().root.find(target).enabled(level)
    }
}

fn handle_error<E: error::Error + ?Sized>(e: &E) {
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    let _ = writeln!(&mut stderr, "log4rs: {}", e);
}

/// Initializes the global logger as a log4rs logger with the provided config.
///
/// A `Handle` object is returned which can be used to adjust the logging
/// configuration.
pub fn init_config(config: config::Config) -> Result<Handle, SetLoggerError> {
    let mut handle = None;
    log::set_logger(|max_log_level| {
        let logger = Logger::new(config);
        max_log_level.set(logger.max_log_level());
        handle = Some(Handle {
            shared: logger.0.clone(),
            max_log_level: max_log_level,
        });
        Box::new(logger)
    }).map(|()| handle.unwrap())
}

/// A handle to the active logger.
pub struct Handle {
    shared: Arc<ArcCell<SharedLogger>>,
    max_log_level: MaxLogLevelFilter,
}

impl Handle {
    /// Sets the logging configuration.
    pub fn set_config(&self, config: Config) {
        let shared = SharedLogger::new(config);
        self.max_log_level.set(shared.root.max_log_level());
        self.shared.set(Arc::new(shared));
    }
}

/// Initializes the global logger as a log4rs logger configured via a file.
///
/// Configuration is read from a file located at the provided path on the
/// filesystem and appenders are created from the provided `Deserializers`.
///
/// Any nonfatal errors encountered when processing the configuration are
/// reported to stderr.
pub fn init_file<P: AsRef<Path>>(path: P, deserializers: Deserializers) -> Result<(), Error> {
    let path = path.as_ref().to_path_buf();
    let format = try!(get_format(&path));
    let source = try!(read_config(&path));
    let config = try!(parse_config(&source, format, &deserializers));

    let refresh_rate = config.refresh_rate();
    let config = config.into_config();

    match init_config(config) {
        Ok(handle) => {
            if let Some(refresh_rate) = refresh_rate {
                ConfigReloader::start(path,
                                      format,
                                      refresh_rate,
                                      source,
                                      deserializers,
                                      handle);
            }
            Ok(())
        }
        Err(e) => Err(e.into())
    }
}

/// An error initializing the logging framework from a file.
#[derive(Debug)]
pub enum Error {
    /// An error from the log crate
    Log(SetLoggerError),
    /// A fatal error initializing the log4rs config.
    Log4rs(Box<error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Log(ref e) => fmt::Display::fmt(e, fmt),
            Error::Log4rs(ref e) => fmt::Display::fmt(e, fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Log(ref e) => error::Error::description(e),
            Error::Log4rs(ref e) => error::Error::description(&**e),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Log(ref e) => Some(e),
            Error::Log4rs(ref e) => Some(&**e),
        }
    }
}

impl From<SetLoggerError> for Error {
    fn from(t: SetLoggerError) -> Error {
        Error::Log(t)
    }
}

impl From<Box<error::Error>> for Error {
    fn from(t: Box<error::Error>) -> Error {
        Error::Log4rs(t)
    }
}

fn get_format(path: &Path) -> Result<Format, Box<error::Error>> {
    match path.extension().and_then(|s| s.to_str()) {
        #[cfg(feature = "yaml")]
        Some("yaml") | Some("yml") => Ok(Format::Yaml),
        #[cfg(not(feature = "yaml"))]
        Some("yaml") | Some("yml") => Err("the `yaml` feature is required for YAML support".into()),
        #[cfg(feature = "json")]
        Some("json") => Ok(Format::Json),
        #[cfg(not(feature = "json"))]
        Some("json") => Err("the `json` feature is required for JSON support".into()),
        #[cfg(feature = "toml")]
        Some("toml") => Ok(Format::Toml),
        #[cfg(not(feature = "toml"))]
        Some("toml") => Err("the `toml` feature is required for TOML support".into()),
        Some(f) => Err(format!("unsupported file format `{}`", f).into()),
        None => Err("unable to determine the file format".into()),
    }
}

fn read_config(path: &Path) -> Result<String, Box<error::Error>> {
    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

fn parse_config(source: &str,
                format: Format,
                deserializers: &Deserializers)
                -> Result<file::Config, Box<error::Error>> {
    let config = try!(file::Config::parse(&source, format, deserializers));
    for error in config.errors() {
        handle_error(error);
    }
    Ok(config)
}

struct ConfigReloader {
    path: PathBuf,
    format: Format,
    rate: Duration,
    source: String,
    deserializers: Deserializers,
    handle: Handle,
}

impl ConfigReloader {
    fn start(path: PathBuf,
             format: Format,
             rate: Duration,
             source: String,
             deserializers: Deserializers,
             handle: Handle) {
        let mut reloader = ConfigReloader {
            path: path,
            format: format,
            rate: rate,
            source: source,
            deserializers: deserializers,
            handle: handle,
        };

        thread::Builder::new()
            .name("log4rs refresh".to_owned())
            .spawn(move || reloader.run())
            .unwrap();
    }

    fn run(&mut self) {
        loop {
            thread::sleep(self.rate);

            let source = match read_config(&self.path) {
                Ok(source) => source,
                Err(err) => {
                    handle_error(&*err);
                    continue;
                }
            };

            if source == self.source {
                continue;
            }

            self.source = source;

            let config = match parse_config(&self.source, self.format, &self.deserializers) {
                Ok(config) => config,
                Err(err) => {
                    handle_error(&*err);
                    continue;
                }
            };
            let refresh_rate = config.refresh_rate();
            let config = config.into_config();

            self.handle.set_config(config);

            match refresh_rate {
                Some(rate) => self.rate = rate,
                None => return,
            }
        }
    }
}

trait ErrorInternals {
    fn new(message: String) -> Self;
}

#[doc(hidden)]
trait ConfigPrivateExt {
    fn unpack(self) -> (Vec<config::Appender>, config::Root, Vec<config::Logger>);
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
        let root = config::Root::builder().build(LogLevelFilter::Debug);
        let mut config = config::Config::builder();
        let logger = config::Logger::builder().build("foo::bar".to_string(), LogLevelFilter::Trace);
        config = config.logger(logger);
        let logger = config::Logger::builder()
                         .build("foo::bar::baz".to_string(), LogLevelFilter::Off);
        config = config.logger(logger);
        let logger = config::Logger::builder()
                         .build("foo::baz::buz".to_string(), LogLevelFilter::Error);
        config = config.logger(logger);
        let config = config.build(root).unwrap();

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
