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

use std::{collections::HashMap, fmt, time::Duration};

use derivative::Derivative;
use log::LevelFilter;
use serde::de::{self, Deserialize as SerdeDeserialize};
use thiserror::Error;

use crate::{append::AppenderConfig, config, filter::IntoFilter};

#[allow(unused_imports)]
use crate::append;

use super::runtime::IntoAppender;

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
where
    A: Clone + IntoAppender,
    F: Clone + IntoFilter,
{
    #[serde(deserialize_with = "de_duration", default)]
    refresh_rate: Option<Duration>,

    #[serde(default)]
    root: Root,

    #[serde(default)]
    appenders: HashMap<String, AppenderConfig<A, F>>,

    #[serde(default)]
    loggers: HashMap<String, Logger>,
}

impl<A, F> RawConfig<A, F>
where
    A: Clone + IntoAppender,
    F: Clone + IntoFilter,
    Self: for<'de> serde::Deserialize<'de>,
{
    /// Build RawConfig from the serde::Deserializer trait
    pub fn from_serde<'de, D>(
        deserializer: D,
    ) -> Result<RawConfig<A, F>, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::deserialize(deserializer)
    }
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

impl<A, F> RawConfig<A, F>
where
    A: Clone + IntoAppender,
    F: Clone + IntoFilter,
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
    pub fn appenders_lossy(&self) -> (Vec<config::Appender>, AppenderErrors) {
        let mut appenders: Vec<config::Appender> = vec![];
        let mut errors = vec![];

        self.appenders.iter().for_each(|(k, v)| {
            let build = config::Appender::builder();
            match v.clone().into_appender(build, k.clone()) {
                Ok(ok) => appenders.push(ok),
                Err(err) => errors.push(err),
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
        use crate::{
            append::{
                file::{FileAppender, FileAppenderConfig},
                LocalAppender,
            },
            filter::LocalFilter,
        };

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
        kind: pattern
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
        let errors = config.appenders_lossy().1;
        println!("{:?}", errors);
        assert!(errors.is_empty());
    }

    #[test]
    #[cfg(feature = "yaml_format")]
    fn empty() {
        use crate::{append::LocalAppender, filter::LocalFilter};

        ::serde_yaml::from_str::<RawConfig<LocalAppender, LocalFilter>>("{}").unwrap();
    }
}
