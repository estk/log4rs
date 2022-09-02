//! Support for log4rs configuration from files.
//!
//! Multiple file formats are supported, each requiring a Cargo feature to be
//! enabled. YAML support requires the `yaml_format` feature, JSON support requires
//! the `json_format` feature, and TOML support requires the `toml_format` feature.
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
//!       # Like appenders, encoders are identified by their "kind".
//!       #
//!       # Default: pattern
//!       kind: pattern
//!
//!       # The remainder of the configuration is passed along to the
//!       # encoder's builder, and will vary based on the kind of encoder.
//!       pattern: "{d} [{t}] {m}{n}"
//!
//! # The root logger is configured by the "root" map.
//! root:
//!
//!   # The maximum log level for the root logger.
//!   #
//!   # Default: warn
//!   level: warn
//!
//!   # The list of appenders attached to the root logger.
//!   #
//!   # Default: empty list
//!   appenders:
//!     - foo
//!
//! # The "loggers" map contains the set of configured loggers, indexed by their
//! # names.
//! loggers:
//!
//!   foo::bar::baz:
//!
//!     # The maximum log level.
//!     #
//!     # Default: parent logger's level
//!     level: trace
//!
//!     # The list of appenders attached to the logger.
//!     #
//!     # Default: empty list
//!     appenders:
//!       - foo
//!
//!     # The additivity of the logger. If true, appenders attached to the logger's
//!     # parent will also be attached to this logger.
//!     #
//!     Default: true
//!     additive: false
//! ```
#![allow(deprecated)]

use std::{
    borrow::ToOwned, collections::HashMap, fmt, marker::PhantomData, sync::Arc, time::Duration,
};

use anyhow::anyhow;
use derivative::Derivative;
use log::LevelFilter;
use serde::de::{self, Deserialize as SerdeDeserialize, DeserializeOwned};
use serde_value::Value;
use thiserror::Error;
use typemap::{Key, ShareCloneMap};

use crate::{config, append::AppenderConfig, filter::IntoFilter};

#[allow(unused_imports)]
use crate::append;



use super::runtime::IntoAppender;

/// A trait implemented by traits which are deserializable.
pub trait Deserializable: 'static {
    /// Returns a name for objects implementing the trait suitable for display in error messages.
    ///
    /// For example, the `Deserializable` implementation for the `Append` trait returns "appender".
    fn name() -> &'static str;
}

/// A trait for objects that can deserialize log4rs components out of a config.
#[deprecated]
pub trait Deserialize: Send + Sync + 'static {
    /// The trait that this deserializer will create.
    type Trait: ?Sized + Deserializable;

    /// This deserializer's configuration.
    type Config: DeserializeOwned;

    /// Create a new trait object based on the provided config.
    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>>;
}

trait ErasedDeserialize: Send + Sync + 'static {
    type Trait: ?Sized;

    fn deserialize(
        &self,
        config: Value,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>>;
}

struct DeserializeEraser<T>(T);

impl<T> ErasedDeserialize for DeserializeEraser<T>
where
    T: Deserialize,
{
    type Trait = T::Trait;

    fn deserialize(
        &self,
        config: Value,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let config = config.deserialize_into()?;
        self.0.deserialize(config, deserializers)
    }
}

struct KeyAdaptor<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized + 'static> Key for KeyAdaptor<T> {
    type Value = HashMap<String, Arc<dyn ErasedDeserialize<Trait = T>>>;
}

/// A container of `Deserialize`rs.
#[derive(Clone)]
#[deprecated]
pub struct Deserializers(ShareCloneMap);

impl Default for Deserializers {
    fn default() -> Deserializers {
        #[allow(unused_mut)]
        let mut d = Deserializers::empty();
        d
    }
}

impl Deserializers {
    /// Creates a `Deserializers` with default mappings.
    ///
    /// All are enabled by default.
    ///
    /// * Appenders
    ///     * "console" -> `ConsoleAppenderDeserializer`
    ///         * Requires the `console_appender` feature.
    ///     * "file" -> `FileAppenderDeserializer`
    ///         * Requires the `file_appender` feature.
    ///     * "rolling_file" -> `RollingFileAppenderDeserializer`
    ///         * Requires the `rolling_file_appender` feature.
    /// * Encoders
    ///     * "pattern" -> `PatternEncoderDeserializer`
    ///         * Requires the `pattern_encoder` feature.
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
    pub fn new() -> Deserializers {
        Deserializers::default()
    }

    /// Creates a new `Deserializers` with no mappings.
    pub fn empty() -> Deserializers {
        Deserializers(ShareCloneMap::custom())
    }

    /// Adds a mapping from the specified `kind` to a deserializer.
    pub fn insert<T>(&mut self, kind: &str, deserializer: T)
    where
        T: Deserialize,
    {
        self.0
            .entry::<KeyAdaptor<T::Trait>>()
            .or_insert_with(HashMap::new)
            .insert(kind.to_owned(), Arc::new(DeserializeEraser(deserializer)));
    }

    /// Deserializes a value of a specific type and kind.
    pub fn deserialize<T: ?Sized>(&self, kind: &str, config: Value) -> anyhow::Result<Box<T>>
    where
        T: Deserializable,
    {
        match self.0.get::<KeyAdaptor<T>>().and_then(|m| m.get(kind)) {
            Some(b) => b.deserialize(config, self),
            None => Err(anyhow!(
                "no {} deserializer for kind `{}` registered",
                T::name(),
                kind
            )),
        }
    }
}

#[derive(Debug, Error)]
pub enum DeserializingConfigError {
    #[error("error deserializing appender {0}: {1}")]
    Appender(String, anyhow::Error),
    #[error("error deserializing filter attached to appender {0}: {1}")]
    Filter(String, anyhow::Error),
}

/// A raw deserializable log4rs configuration.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig<A, F> 
where A: Clone + IntoAppender,F: Clone+IntoFilter
{
    #[serde(deserialize_with = "de_duration", default)]
    refresh_rate: Option<Duration>,

    #[serde(default)]
    root: Root,

    #[serde(default)]
    appenders: HashMap<String, AppenderConfig<A,F>>,

    #[serde(default)]
    loggers: HashMap<String, Logger>,
}

#[derive(Debug, Error)]
#[error("errors deserializing appenders {0:#?}")]
pub struct AppenderErrors(Vec<DeserializingConfigError>);

impl AppenderErrors {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn handle(&mut self) {
        for error in self.0.drain(..) {
            crate::handle_error(&error.into());
        }
    }
}

impl<A,F> RawConfig<A,F> 
where A: Clone + IntoAppender,F: Clone+IntoFilter
{
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
            })
            .collect()
    }

    /// Returns the appenders.
    ///
    /// Any components which fail to be deserialized will be ignored.
    pub fn appenders_lossy(
        &self,
        _: &Deserializers,
    ) -> (Vec<config::Appender>, AppenderErrors) {
        let mut appenders:Vec<config::Appender> = vec![];
        let mut errors = vec![];

        self.appenders.iter().for_each(|(k,v)|{
            let build = config::Appender::builder();
            match v.clone().into_appender(build, k.clone()) {
                Ok(ok) => {appenders.push(ok)},
                Err(err) => {errors.push(err)},
            }
        });

        (appenders, AppenderErrors(errors))
    }

    /// Returns the requested refresh rate.
    pub fn refresh_rate(&self) -> Option<Duration> {
        self.refresh_rate
    }
}

fn de_duration<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct S(Duration);

    impl<'de2> de::Deserialize<'de2> for S {
        fn deserialize<D>(d: D) -> Result<S, D::Error>
        where
            D: de::Deserializer<'de2>,
        {
            struct V;

            impl<'de3> de::Visitor<'de3> for V {
                type Value = S;

                fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                    fmt.write_str("a duration")
                }

                fn visit_str<E>(self, v: &str) -> Result<S, E>
                where
                    E: de::Error,
                {
                    humantime::parse_duration(v).map(S).map_err(E::custom)
                }
            }

            d.deserialize_any(V)
        }
    }

    Option::<S>::deserialize(d).map(|r| r.map(|s| s.0))
}

#[derive(Clone, Debug, Derivative, serde::Deserialize)]
#[derivative(Default)]
#[serde(deny_unknown_fields)]
struct Root {
    #[serde(default = "root_level_default")]
    #[derivative(Default(value = "root_level_default()"))]
    level: LevelFilter,
    #[serde(default)]
    appenders: Vec<String>,
}

fn root_level_default() -> LevelFilter {
    LevelFilter::Debug
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct Logger {
    level: LevelFilter,
    #[serde(default)]
    appenders: Vec<String>,
    #[serde(default = "logger_additive_default")]
    additive: bool,
}

fn logger_additive_default() -> bool {
    true
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    #[cfg(all(feature = "yaml_format", feature = "threshold_filter"))]
    fn full_deserialize() {
        use crate::{append::{LocalAppender, file::{FileAppender, FileAppenderConfig}}, filter::LocalFilter};

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
        let config = ::serde_yaml::from_str::<RawConfig<LocalAppender, LocalFilter>>(cfg).unwrap();
        let errors = config.appenders_lossy(&Deserializers::new()).1;
        println!("{:?}", errors);
        assert!(errors.is_empty());
    }

    #[test]
    #[cfg(feature = "yaml_format")]
    fn empty() {
        use crate::{filter::LocalFilter, append::LocalAppender};

        ::serde_yaml::from_str::<RawConfig<LocalAppender, LocalFilter>>("{}").unwrap();
    }
}
