//! Support for log4rs configuration from files.
//!
//! Multiple file formats are supported, each requiring a Cargo feature to be
//! enabled. YAML support requires the `yaml` feature, and JSON support requires
//! the `JSON` feature.
//!
//! # Syntax
//!
//! All file formats currently share the same structure. The example below is
//! of the YAML format.
//!
//! ```yaml
//! # If set, log4rs will scan the file at the specified rate for changes and
//! # automatically reconfigure the logger. The input string is parsed by the
//! # humantime crate.
//! refresh_rate: 30 seconds
//!
//! # The "appenders" map contains the set of appenders, indexed by their names.
//! appenders:
//!
//!   foo:
//!
//!     # All appenders must specify a "kind", which will be used to look up the
//!     # logic to construct the appender in the `Deserializers` passed to the
//!     # deserialization function.
//!     kind: console
//!
//!     # Filters attached to an appender are specified inside the "filters"
//!     # array.
//!     filters:
//!
//!       -
//!         # Like appenders, filters are identified by their "kind".
//!         kind: threshold
//!
//!         # The remainder of the configuration is passed along to the
//!         # filter's builder, and will vary based on the kind of filter.
//!         level: error
//!
//!     # The remainder of the configuration is passed along to the appender's
//!     # builder, and will vary based on the kind of appender.
//!     # Appenders will commonly be associated with an encoder.
//!     encoder:
//!
//!       # Like appenders, encoders are identified by their "kind". If no kind
//!       # is specified, it will default to "pattern".
//!       kind: pattern
//!
//!       # The remainder of the configuration is passed along to the
//!       # encoder's builder, and will vary based on the kind of encoder.
//!       pattern: "{d} [{t}] {m}{n}"
//!
//! # The root logger is configured by the "root" map. Defaults to a level of
//! # "debug" and no appenders if not provided.
//! root:
//!
//!   # The maximum log level for the root logger.
//!   level: warn
//!
//!   # The list of appenders attached to the root logger. Defaults to an empty
//!   # list if not specified.
//!   appenders:
//!     - foo
//!
//! # The "loggers" map contains the set of configured loggers, indexed by their
//! # names.
//! loggers:
//!
//!   foo::bar::baz:
//!
//!     # The maximum log level. Defaults to the level of the logger's parent if
//!     # not specified.
//!     level: trace
//!
//!     # The list of appenders attached to the logger. Defaults to an empty
//!     # list if not specified.
//!     appenders:
//!       - foo
//!
//!     # The additivity of the logger. If true, appenders attached to the
//!     # logger's parent will also be attached to this logger. Defaults to true
//!     # if not specified.
//!     additive: false
//! ```
use humantime;
use log::LogLevelFilter;
use serde;
use serde::de::{self, Deserialize as SerdeDeserialize};
use serde_value::Value;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use typemap::{Key, ShareCloneMap};

use config;
use append::AppenderConfig;

/// A trait implemented by traits which are deserializable.
pub trait Deserializable: 'static {
    /// Returns a name for objects implementing the trait suitable for display in error messages.
    ///
    /// For example, the `Deserializable` implementation for the `Append` trait returns "appender".
    fn name() -> &'static str;
}

/// A trait for objects that can deserialize log4rs components out of a config.
pub trait Deserialize: Send + Sync + 'static {
    /// The trait that this deserializer will create.
    type Trait: ?Sized + Deserializable;

    /// This deserializer's configuration.
    type Config: serde::Deserialize;

    /// Create a new trait object based on the provided config.
    fn deserialize(&self,
                   config: Self::Config,
                   deserializers: &Deserializers)
                   -> Result<Box<Self::Trait>, Box<error::Error + Sync + Send>>;
}

trait ErasedDeserialize: Send + Sync + 'static {
    type Trait: ?Sized;

    fn deserialize(&self,
                   config: Value,
                   deserializers: &Deserializers)
                   -> Result<Box<Self::Trait>, Box<error::Error + Sync + Send>>;
}

struct DeserializeEraser<T>(T);

impl<T> ErasedDeserialize for DeserializeEraser<T>
    where T: Deserialize
{
    type Trait = T::Trait;

    fn deserialize(&self,
                   config: Value,
                   deserializers: &Deserializers)
                   -> Result<Box<Self::Trait>, Box<error::Error + Sync + Send>> {
        let config = config.deserialize_into()?;
        self.0.deserialize(config, deserializers)
    }
}

struct KeyAdaptor<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized + 'static> Key for KeyAdaptor<T> {
    type Value = HashMap<String, Arc<ErasedDeserialize<Trait = T>>>;
}

/// A container of `Deserialize`rs.
#[derive(Clone)]
pub struct Deserializers(ShareCloneMap);

/// Creates a `Deserializers` with the following mappings:
///
/// * Appenders
///     * "console" -> `ConsoleAppenderDeserializer`
///         * Requires the `console_appender` feature (enabled by default).
///     * "file" -> `FileAppenderDeserializer`
///         * Requires the `file_appender` feature (enabled by default).
///     * "rolling_file" -> `RollingFileAppenderDeserializer`
///         * Requires the `rolling_file_appender` feature.
/// * Encoders
///     * "pattern" -> `PatternEncoderDeserializer`
///         * Requires the `pattern_encoder` feature (enabled by default).
///     * "json" -> `JsonEncoderDeserializer`
///         * Requires the `json_encoder` feature.
/// * Filters
///     * "threshold" -> `ThresholdFilterDeserializer`
///         * Requires the `threshold_filter` feature.
/// * Policies
///     *  "compound" -> `CompoundPolicyDeserializer`
///         * Requires the `compound_policy` feature.
/// * Rollers
///     * "delete" -> `DeleteRollerDeserializer`
///         * Requires the `delete_roller` feature.
///     * "fixed_window" -> `FixedWindowRollerDeserializer`
///         * Requires the `fixed_window_roller` feature.
/// * Triggers
///     * "size" -> `SizeTriggerDeserializer`
///         * Requires the `size_trigger` feature.
impl Default for Deserializers {
    fn default() -> Deserializers {
        let mut d = Deserializers::empty();

        #[cfg(feature = "console_appender")]
        d.insert("console", ::append::console::ConsoleAppenderDeserializer);

        #[cfg(feature = "file_appender")]
        d.insert("file", ::append::file::FileAppenderDeserializer);

        #[cfg(feature = "rolling_file_appender")]
        d.insert("rolling_file", ::append::rolling_file::RollingFileAppenderDeserializer);

        #[cfg(feature = "compound_policy")]
        d.insert("compound", ::append::rolling_file::policy::compound::CompoundPolicyDeserializer);

        #[cfg(feature = "delete_roller")]
        d.insert("delete", ::append::rolling_file::policy::compound::roll::delete::DeleteRollerDeserializer);

        #[cfg(feature = "fixed_window_roller")]
        d.insert("fixed_window", ::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRollerDeserializer);

        #[cfg(feature = "size_trigger")]
        d.insert("size", ::append::rolling_file::policy::compound::trigger::size::SizeTriggerDeserializer);

        #[cfg(feature = "json_encoder")]
        d.insert("json", ::encode::json::JsonEncoderDeserializer);

        #[cfg(feature = "pattern_encoder")]
        d.insert("pattern", ::encode::pattern::PatternEncoderDeserializer);

        #[cfg(feature = "threshold_filter")]
        d.insert("threshold", ::filter::threshold::ThresholdFilterDeserializer);

        d
    }
}

impl Deserializers {
    /// Creates a new `Deserializers` with default mappings.
    pub fn new() -> Deserializers {
        Deserializers::default()
    }

    /// Creates a new `Deserializers` with no mappings.
    pub fn empty() -> Deserializers {
        Deserializers(ShareCloneMap::custom())
    }

    /// Adds a mapping from the specified `kind` to a deserializer.
    pub fn insert<T>(&mut self, kind: &str, deserializer: T)
        where T: Deserialize
    {
        self.0
            .entry::<KeyAdaptor<T::Trait>>()
            .or_insert_with(HashMap::new)
            .insert(kind.to_owned(), Arc::new(DeserializeEraser(deserializer)));
    }

    /// Deserializes a value of a specific type and kind.
    pub fn deserialize<T: ?Sized>(&self,
                                  kind: &str,
                                  config: Value)
                                  -> Result<Box<T>, Box<error::Error + Sync + Send>>
        where T: Deserializable
    {
        match self.0.get::<KeyAdaptor<T>>().and_then(|m| m.get(kind)) {
            Some(b) => b.deserialize(config, self),
            None => {
                Err(format!("no {} deserializer for kind `{}` registered",
                            T::name(),
                            kind)
                    .into())
            }
        }
    }
}

/// An error deserializing a configuration into a log4rs `Config`.
#[derive(Debug)]
pub struct Error(ErrorKind, Box<error::Error + Sync + Send>);

#[derive(Debug)]
enum ErrorKind {
    Appender(String),
    Filter(String),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ErrorKind::Appender(ref name) => {
                write!(fmt, "error deserializing appender {}: {}", name, self.1)
            }
            ErrorKind::Filter(ref name) => {
                write!(fmt, "error deserializing filter attached to appender {}: {}", name, self.1)
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "error deserializing a log4rs `Config`"
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(&*self.1)
    }
}

/// A raw deserializable log4rs configuration.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    #[serde(deserialize_with = "de_duration", default)]
    refresh_rate: Option<Duration>,
    #[serde(default)]
    root: Root,
    #[serde(default)]
    appenders: HashMap<String, AppenderConfig>,
    #[serde(default)]
    loggers: HashMap<String, Logger>,
}

impl RawConfig {
    /// Returns the root.
    pub fn root(&self) -> config::Root {
        config::Root::builder()
            .appenders(self.root.appenders.clone())
            .build(self.root.level)
    }

    /// Returns the loggers.
    pub fn loggers(&self) -> Vec<config::Logger> {
        self.loggers
            .iter()
            .map(|(name, logger)| {
                config::Logger::builder()
                    .appenders(logger.appenders.clone())
                    .additive(logger.additive)
                    .build(name.clone(), logger.level)
            }).collect()
    }

    /// Returns the appenders.
    ///
    /// Any components which fail to be deserialized will be ignored.
    pub fn appenders_lossy(&self,
                           deserializers: &Deserializers)
                           -> (Vec<config::Appender>, Vec<Error>) {
        let mut appenders = vec![];
        let mut errors = vec![];

        for (name, appender) in &self.appenders {
            let mut builder = config::Appender::builder();
            for filter in &appender.filters {
                match deserializers.deserialize(&filter.kind, filter.config.clone()) {
                    Ok(filter) => builder = builder.filter(filter),
                    Err(e) => errors.push(Error(ErrorKind::Filter(name.clone()), e)),
                }
            }
            match deserializers.deserialize(&appender.kind, appender.config.clone()) {
                Ok(appender) => appenders.push(builder.build(name.clone(), appender)),
                Err(e) => errors.push(Error(ErrorKind::Appender(name.clone()), e)),
            }
        }

        (appenders, errors)
    }

    /// Returns the requested refresh rate.
    pub fn refresh_rate(&self) -> Option<Duration> {
        self.refresh_rate
    }
}

fn de_duration<D>(d: D) -> Result<Option<Duration>, D::Error>
    where D: de::Deserializer
{
    struct S(Duration);

    impl de::Deserialize for S {
        fn deserialize<D>(d: D) -> Result<S, D::Error>
            where D: de::Deserializer
        {
            struct V;

            impl de::Visitor for V {
                type Value = S;

                fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                    fmt.write_str("a duration")
                }

                fn visit_str<E>(self, v: &str) -> Result<S, E>
                    where E: de::Error
                {
                    humantime::parse_duration(v)
                        .map(S)
                        .map_err(|e| E::custom(e))
                }
            }

            d.deserialize(V)
        }
    }

    Option::<S>::deserialize(d).map(|r| r.map(|s| s.0))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    #[serde(deserialize_with = "::priv_serde::de_filter", default = "root_level_default")]
    level: LogLevelFilter,
    #[serde(default)]
    appenders: Vec<String>,
}

impl Default for Root {
    fn default() -> Root {
        Root {
            level: root_level_default(),
            appenders: vec![]
        }
    }
}

fn root_level_default() -> LogLevelFilter {
    LogLevelFilter::Debug
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Logger {
    #[serde(deserialize_with = "::priv_serde::de_filter")]
    level: LogLevelFilter,
    #[serde(default)]
    appenders: Vec<String>,
    #[serde(default = "logger_additive_default")]
    additive: bool,
}

fn logger_additive_default() -> bool { true }

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    #[cfg(all(feature = "yaml_format", feature = "threshold_filter"))]
    fn full_deserialize() {
        let cfg = r#"
refresh_rate: 60 seconds

appenders:
  console:
    kind: console
    filters:
      - kind: threshold
        level: debug
  baz:
    kind: file
    path: /tmp/baz.log
    encoder:
      pattern: "%m"

root:
  appenders:
    - console
  level: info

loggers:
  foo::bar::baz:
    level: warn
    appenders:
      - baz
    additive: false
"#;
        let config = ::serde_yaml::from_str::<RawConfig>(cfg).unwrap();
        let errors = config.appenders_lossy(&Deserializers::new()).1;
        println!("{:?}", errors);
        assert!(errors.is_empty());
    }

    #[test]
    #[cfg(feature = "yaml_format")]
    fn empty() {
        let config = ::serde_yaml::from_str::<RawConfig>("{}").unwrap();
    }
}
