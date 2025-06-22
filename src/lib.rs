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
//! Implementations:
//!   - [console](append/console/struct.ConsoleAppenderDeserializer.html#configuration): requires the `console_appender` feature.
//!   - [file](append/file/struct.FileAppenderDeserializer.html#configuration): requires the `file_appender` feature.
//!   - [rolling_file](append/rolling_file/struct.RollingFileAppenderDeserializer.html#configuration): requires the `rolling_file_appender` feature and can be configured with the `compound_policy`.
//!     - [compound](append/rolling_file/policy/compound/struct.CompoundPolicyDeserializer.html#configuration): requires the `compound_policy` feature
//!       - Rollers
//!         - [delete](append/rolling_file/policy/compound/roll/delete/struct.DeleteRollerDeserializer.html#configuration): requires the `delete_roller` feature
//!         - [fixed_window](append/rolling_file/policy/compound/roll/fixed_window/struct.FixedWindowRollerDeserializer.html#configuration): requires the `fixed_window_roller` feature
//!       - Triggers
//!         - [size](append/rolling_file/policy/compound/trigger/size/struct.SizeTriggerDeserializer.html#configuration): requires the `size_trigger` feature
//!         - [time](append/rolling_file/policy/compound/trigger/tine/struct.TimeTriggerDeserializer.html#configuration): requires the `time_trigger` feature
//!         - [onstartup](append/rolling_file/policy/compound/trigger/tine/struct.OnStartUpTriggerDeserializer.html#configuration): requires the `onstartup_trigger` feature
//!
//! ## Encoders
//!
//! An encoder is responsible for taking a log record, transforming it into the
//! appropriate output format, and writing it out. An appender will normally
//! use an encoder internally.
//!
//! Implementations:
//!   - [pattern](encode/pattern/struct.PatternEncoderDeserializer.html#configuration): requires the `pattern_encoder` feature
//!   - [json](encode/json/struct.JsonEncoderDeserializer.html#configuration): requires the `json_encoder` feature
//!
//! ## Filters
//!
//! Filters are associated with appenders and, like the name would suggest,
//! filter log events coming into that appender.
//!
//! Implementations:
//!   - [threshold](filter/threshold/struct.ThresholdFilterDeserializer.html#configuration): requires the `threshold_filter` feature
//!
//! ## Loggers
//!
//! A log event is targeted at a specific logger, which are identified by
//! string names. The logging macros built in to the `log` crate set the logger
//! of a log event to the one identified by the module containing the
//! invocation location.
//!
//! Loggers form a hierarchy: logger names are divided into components by "::".
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
//! its *additive* set to `false`. Log events sent to the logger that are not
//! filtered out by the logger's maximum log level will be sent to all
//! associated appenders.
//!
//! The "root" logger is the ancestor of all other loggers. Since it has no
//! ancestors, its additivity cannot be configured.
//!
//! # Configuration
//!
//! For a detailed breakdown on configuration, refer to the
//! [config module](config/index.html#configuration).
//!
//! log4rs makes heavy use of Cargo features to enable consumers to pick the
//! functionality they wish to use. File-based configuration requires the `file`
//! feature, and each file format requires its own feature as well. In addition,
//! each component has its own feature. For example, YAML support requires the
//! `yaml_format` feature and the console appender requires the
//! `console_appender` feature.
//!
//! By default, the `all_components`, `gzip`, `file`, and `yaml_format` features
//! are enabled.
//!
//! As a convenience, the `all_components` feature activates all logger components.
//!
//! # Examples
//!
//! ## Configuration via a YAML file
//!
//! ```yaml
//! # Scan this file for changes every 30 seconds
//! refresh_rate: 30 seconds
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
//!
//! Add the following in your application initialization.
//!
//! ```no_run
//! # #[cfg(feature = "config_parsing")]
//! # fn f() {
//! log4rs::init_file("log4rs.yml", Default::default()).unwrap();
//! # }
//! ```
//!
//! ## Programmatically constructing a configuration:
//!
//! ```no_run
//! # #[cfg(all(feature = "console_appender",
//! #           feature = "file_appender",
//! #           feature = "pattern_encoder"))]
//! # fn f() {
//! use log::LevelFilter;
//! use log4rs::append::console::ConsoleAppender;
//! use log4rs::append::file::FileAppender;
//! use log4rs::encode::pattern::PatternEncoder;
//! use log4rs::config::{Appender, Config, Logger, Root};
//!
//! fn main() {
//!     let stdout = ConsoleAppender::builder().build();
//!
//!     let requests = FileAppender::builder()
//!         .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
//!         .build("log/requests.log")
//!         .unwrap();
//!
//!     let config = Config::builder()
//!         .appender(Appender::builder().build("stdout", Box::new(stdout)))
//!         .appender(Appender::builder().build("requests", Box::new(requests)))
//!         .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
//!         .logger(Logger::builder()
//!             .appender("requests")
//!             .additive(false)
//!             .build("app::requests", LevelFilter::Info))
//!         .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
//!         .unwrap();
//!
//!     let handle = log4rs::init_config(config).unwrap();
//!
//!     // use handle to change logger configuration at runtime
//! }
//! # }
//! # fn main() {}
//! ```
//!
//! ## Custom implementations of logging components
//!
//! You can impl some trait for your struct and use it with log4rs. For example:
//! - Impl [log4rs::append::Append](append/trait.Append.html) for your custom appender.
//! - Impl [log4rs::encode::Encode](encode/trait.Encode.html) for your custom encoder.
//! - Impl [log4rs::filter::Filter](filter/trait.Filter.html) for your custom filter.
//!
//! Here is a very simple example to create a custom appender,
//! for more examples about custom, see [examples/custom.rs](https://github.com/estk/log4rs/tree/main/examples/custom.rs):
//! ```no_run
//! # fn f() {
//! use log4rs::append::Append;
//! use log4rs::config::{Appender, Root};
//!
//! #[derive(Debug)]
//! struct MyAppender(usize);
//!
//! // impl your process record logic here
//! impl Append for MyAppender {
//!     fn append(&self, record: &log::Record) -> anyhow::Result<()> {
//!         println!("appender({}): {record:?}", self.0);
//!         Ok(())
//!     }
//!     fn flush(&self) {}
//! }
//!
//! fn main() {
//!     let appender = MyAppender(100);
//!     let log_config = log4rs::config::Config::builder()
//!         .appender(Appender::builder().build("my_appender", Box::new(appender)))
//!         .build(
//!             Root::builder()
//!                 .appender("my_appender")
//!                 .build(log::LevelFilter::Info),
//!         )
//!         .unwrap();
//!     log4rs::init_config(log_config).unwrap();
//!     log::trace!("This is a trace message");
//!     log::info!("This is an info message");
//!     log::warn!("This is a warning message");
//! }
//! # }
//! # fn main() {}
//! ```
//!
//! To configure log4rs with a file, you should implement [log4rs::config::Deserialize](config/trait.Deserialize.html) for your config and be sure to register it with [log4rs::config::Deserializers](config/struct.Deserializers.html) as shown in the example below.
//!
//! Here is a very simple example to use a custom appender with custom config.
//! For more examples about custom config file, see [examples/custom_config.rs](https://github.com/estk/log4rs/tree/main/examples/custom_config.rs):
//! ```yaml
//! # custom_config.yml
//! appenders:
//!   my_appender:
//!     kind: custom_appender
//!     appender_data: 42
//!
//! root:
//!   level: INFO
//!   appenders:
//!     - my_appender
//! ```
//!
//! ```no_run
//! # #[cfg(feature = "config_parsing")]
//! # fn f() {
//! use log4rs::append::Append;
//! use log4rs::config::{Deserialize, Deserializers};
//!
//! #[derive(Debug)]
//! struct MyAppender(usize);
//!
//! // impl your process record logic here
//! impl Append for MyAppender {
//!     fn append(&self, record: &log::Record) -> anyhow::Result<()> {
//!         println!("appender({}): {record:?}", self.0);
//!         Ok(())
//!     }
//!     fn flush(&self) {}
//! }
//! // Define config struct for custom appender
//! #[derive(serde::Deserialize)]
//! pub struct MyAppenderConfig {
//!     pub appender_data: Option<usize>,
//! }
//!
//! #[derive(Default)]
//! pub struct MyAppenderDeserializer;
//!
//! // impl Deserialize for custom appender config
//! impl Deserialize for MyAppenderDeserializer {
//!     type Trait = dyn Append;
//!
//!     type Config = MyAppenderConfig;
//!
//!     fn deserialize(
//!         &self,
//!         config: MyAppenderConfig,
//!         _: &Deserializers,
//!     ) -> anyhow::Result<Box<Self::Trait>> {
//!         let appender_data = config.appender_data.unwrap_or(0);
//!         let appender = MyAppender(appender_data);
//!         Ok(Box::new(appender))
//!     }
//! }
//!
//! fn main() {
//!     let log_file = "custom_config.yml";
//!     // Access the default deserializers map
//!     let mut deserializers = Deserializers::default();
//!     // Register the "custom_appender" deserializer into the default deserializers map
//!     deserializers.insert("custom_appender", MyAppenderDeserializer);
//!     // Initialize log4rs with the custom deserializers from the file
//!     log4rs::init_file(log_file, deserializers).unwrap();
//!
//!     log::trace!("This is a trace message");
//!     log::info!("This is an info message");
//!     log::warn!("This is a warning message");
//! }
//! # }
//! # fn main() {}
//! ```
//!
//! For more examples see the [examples](https://github.com/estk/log4rs/tree/main/examples).
//!

#![allow(clippy::manual_non_exhaustive)]
#![warn(missing_docs)]

use std::{
    cmp, collections::HashMap, fmt, hash::BuildHasherDefault, io, io::prelude::*, sync::Arc,
};

use arc_swap::ArcSwap;
use fnv::FnvHasher;
use log::{Level, LevelFilter, Metadata, Record};

pub mod append;
pub mod config;
pub mod encode;
pub mod filter;
#[cfg(feature = "console_writer")]
mod priv_io;

pub use config::{init_config, Config};

#[cfg(feature = "config_parsing")]
pub use config::{init_file, init_raw_config};

use self::{append::Append, filter::Filter};

type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

#[derive(Debug)]
struct ConfiguredLogger {
    level: LevelFilter,
    appenders: Vec<usize>,
    children: FnvHashMap<String, ConfiguredLogger>,
}

impl ConfiguredLogger {
    fn add(&mut self, path: &str, mut appenders: Vec<usize>, additive: bool, level: LevelFilter) {
        let (part, rest) = match path.find("::") {
            Some(idx) => (&path[..idx], &path[idx + 2..]),
            None => (path, ""),
        };

        if let Some(child) = self.children.get_mut(part) {
            child.add(rest, appenders, additive, level);
            return;
        }

        let child = if rest.is_empty() {
            if additive {
                appenders.extend(self.appenders.iter().cloned());
            }

            ConfiguredLogger {
                level,
                appenders,
                children: FnvHashMap::default(),
            }
        } else {
            let mut child = ConfiguredLogger {
                level: self.level,
                appenders: self.appenders.clone(),
                children: FnvHashMap::default(),
            };
            child.add(rest, appenders, additive, level);
            child
        };

        self.children.insert(part.to_owned(), child);
    }

    fn max_log_level(&self) -> LevelFilter {
        let mut max = self.level;
        for child in self.children.values() {
            max = cmp::max(max, child.max_log_level());
        }
        max
    }

    fn find(&self, path: &str) -> &ConfiguredLogger {
        let mut node = self;

        for part in path.split("::") {
            match node.children.get(part) {
                Some(child) => node = child,
                None => break,
            }
        }

        node
    }

    fn enabled(&self, level: Level) -> bool {
        self.level >= level
    }

    fn log(&self, record: &log::Record, appenders: &[Appender]) -> Result<(), Vec<anyhow::Error>> {
        let mut errors = vec![];
        if self.enabled(record.level()) {
            for &idx in &self.appenders {
                if let Err(err) = appenders[idx].append(record) {
                    errors.push(err);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug)]
struct Appender {
    appender: Box<dyn Append>,
    filters: Vec<Box<dyn Filter>>,
}

impl Appender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        for filter in &self.filters {
            match filter.filter(record) {
                filter::Response::Accept => break,
                filter::Response::Neutral => {}
                filter::Response::Reject => return Ok(()),
            }
        }

        self.appender.append(record)
    }

    fn flush(&self) {
        self.appender.flush();
    }
}

struct SharedLogger {
    root: ConfiguredLogger,
    appenders: Vec<Appender>,
    err_handler: Box<dyn Send + Sync + Fn(&anyhow::Error)>,
}

impl fmt::Debug for SharedLogger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedLogger")
            .field("root", &self.root)
            .field("appenders", &self.appenders)
            .finish()
    }
}

impl SharedLogger {
    fn new(config: config::Config) -> SharedLogger {
        Self::new_with_err_handler(
            config,
            Box::new(|e: &anyhow::Error| {
                let _ = writeln!(io::stderr(), "log4rs: {}", e);
            }),
        )
    }
    fn new_with_err_handler(
        config: config::Config,
        err_handler: Box<dyn Send + Sync + Fn(&anyhow::Error)>,
    ) -> SharedLogger {
        let (appenders, root, mut loggers) = config.unpack();

        let root = {
            let appender_map = appenders
                .iter()
                .enumerate()
                .map(|(i, appender)| (appender.name(), i))
                .collect::<HashMap<_, _>>();

            let mut root = ConfiguredLogger {
                level: root.level(),
                appenders: root
                    .appenders()
                    .iter()
                    .map(|appender| appender_map[&**appender])
                    .collect(),
                children: FnvHashMap::default(),
            };

            // sort loggers by name length to ensure that we initialize them top to bottom
            loggers.sort_by_key(|l| l.name().len());
            for logger in loggers {
                let appenders = logger
                    .appenders()
                    .iter()
                    .map(|appender| appender_map[&**appender])
                    .collect();
                root.add(logger.name(), appenders, logger.additive(), logger.level());
            }

            root
        };

        let appenders = appenders
            .into_iter()
            .map(|appender| {
                let (_, appender, filters) = appender.unpack();
                Appender { appender, filters }
            })
            .collect();

        SharedLogger {
            root,
            appenders,
            err_handler,
        }
    }
}

/// The fully configured log4rs Logger which is appropriate
/// to use with the `log::set_boxed_logger` function.
#[derive(Debug)]
pub struct Logger(Arc<ArcSwap<SharedLogger>>);

impl Logger {
    /// Create a new `Logger` given a configuration.
    pub fn new(config: config::Config) -> Logger {
        Logger(Arc::new(ArcSwap::new(Arc::new(SharedLogger::new(config)))))
    }
    /// Create a new `Logger` given a configuration and err handler.
    pub fn new_with_err_handler(
        config: config::Config,
        err_handler: Box<dyn Send + Sync + Fn(&anyhow::Error)>,
    ) -> Logger {
        Logger(Arc::new(ArcSwap::new(Arc::new(
            SharedLogger::new_with_err_handler(config, err_handler),
        ))))
    }

    /// Set the max log level above which everything will be filtered.
    pub fn max_log_level(&self) -> LevelFilter {
        self.0.load().root.max_log_level()
    }
    /// Get a `Handler` instance to reconfigure logger while running
    pub fn handle(&self) -> Handle {
        Handle {
            shared: self.0.clone(),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0
            .load()
            .root
            .find(metadata.target())
            .enabled(metadata.level())
    }

    fn log(&self, record: &log::Record) {
        let shared = self.0.load();
        if let Err(errs) = shared
            .root
            .find(record.target())
            .log(record, &shared.appenders)
        {
            for e in errs {
                (shared.err_handler)(&e)
            }
        }
    }

    fn flush(&self) {
        for appender in &self.0.load().appenders {
            appender.flush();
        }
    }
}

pub(crate) fn handle_error(e: &anyhow::Error) {
    let _ = writeln!(io::stderr(), "log4rs: {}", e);
}

/// A handle to the active logger.
#[derive(Clone, Debug)]
pub struct Handle {
    shared: Arc<ArcSwap<SharedLogger>>,
}

impl Handle {
    /// Sets the logging configuration.
    pub fn set_config(&self, config: Config) {
        let shared = SharedLogger::new(config);
        log::set_max_level(shared.root.max_log_level());
        self.shared.store(Arc::new(shared));
    }

    /// Get the maximum log level according to the current configuration
    pub fn max_log_level(&self) -> LevelFilter {
        self.shared.load().root.max_log_level()
    }
}

#[cfg(test)]
mod test {
    use log::{Level, LevelFilter, Log};

    use super::*;

    #[test]
    #[cfg(all(feature = "config_parsing", feature = "json_format"))]
    fn init_from_raw_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("append.log");

        let cfg = serde_json::json!({
            "refresh_rate": "60 seconds",
            "root" : {
                "appenders": ["baz"],
                "level": "info",
            },
            "appenders": {
                "baz": {
                    "kind": "file",
                    "path": path,
                    "encoder": {
                        "pattern": "{m}"
                    }
                }
            },
        });
        let config = serde_json::from_str::<config::RawConfig>(&cfg.to_string()).unwrap();
        if let Err(e) = init_raw_config(config) {
            panic!("{}", e);
        }
        assert!(path.exists());
        log::info!("init_from_raw_config");

        let mut contents = String::new();
        std::fs::File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert_eq!(contents, "init_from_raw_config");
    }

    #[test]
    fn enabled() {
        let root = config::Root::builder().build(LevelFilter::Debug);
        let mut config = config::Config::builder();
        let logger = config::Logger::builder().build("foo::bar", LevelFilter::Trace);
        config = config.logger(logger);
        let logger = config::Logger::builder().build("foo::bar::baz", LevelFilter::Off);
        config = config.logger(logger);
        let logger = config::Logger::builder().build("foo::baz::buz", LevelFilter::Error);
        config = config.logger(logger);
        let config = config.build(root).unwrap();

        let logger = super::Logger::new(config);

        assert!(logger.enabled(&Metadata::builder().level(Level::Warn).target("bar").build()));
        assert!(!logger.enabled(
            &Metadata::builder()
                .level(Level::Trace)
                .target("bar")
                .build()
        ));
        assert!(logger.enabled(
            &Metadata::builder()
                .level(Level::Debug)
                .target("foo")
                .build()
        ));
        assert!(logger.enabled(
            &Metadata::builder()
                .level(Level::Trace)
                .target("foo::bar")
                .build()
        ));
        assert!(!logger.enabled(
            &Metadata::builder()
                .level(Level::Error)
                .target("foo::bar::baz")
                .build()
        ));
        assert!(logger.enabled(
            &Metadata::builder()
                .level(Level::Debug)
                .target("foo::bar::bazbuz")
                .build()
        ));
        assert!(!logger.enabled(
            &Metadata::builder()
                .level(Level::Error)
                .target("foo::bar::baz::buz")
                .build()
        ));
        assert!(!logger.enabled(
            &Metadata::builder()
                .level(Level::Warn)
                .target("foo::baz::buz")
                .build()
        ));
        assert!(!logger.enabled(
            &Metadata::builder()
                .level(Level::Warn)
                .target("foo::baz::buz::bar")
                .build()
        ));
        assert!(logger.enabled(
            &Metadata::builder()
                .level(Level::Error)
                .target("foo::baz::buz::bar")
                .build()
        ));
    }
}
