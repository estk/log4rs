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
use humantime;
use log::LevelFilter;
use serde::de::{self, Deserialize as SerdeDeserialize, DeserializeOwned};
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
    type Config: DeserializeOwned;

    /// Create a new trait object based on the provided config.
    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &Deserializers,
    ) -> Result<Box<Self::Trait>, Box<error::Error + Sync + Send>>;
}

trait ErasedDeserialize: Send + Sync + 'static {
    type Trait: ?Sized;

    fn deserialize(
        &self,
        config: Value,
        deserializers: &Deserializers,
    ) -> Result<Box<Self::Trait>, Box<error::Error + Sync + Send>>;
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
    ) -> Result<Box<Self::Trait>, Box<error::Error + Sync + Send>> {
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

impl Default for Deserializers {
    fn default() -> Deserializers {
        let mut d = Deserializers::empty();

        #[cfg(feature = "console_appender")]
        d.insert("console", ::append::console::ConsoleAppenderDeserializer);

        #[cfg(feature = "file_appender")]
        d.insert("file", ::append::file::FileAppenderDeserializer);

        #[cfg(feature = "rolling_file_appender")]
        d.insert(
            "rolling_file",
            ::append::rolling_file::RollingFileAppenderDeserializer,
        );

        #[cfg(feature = "compound_policy")]
        d.insert(
            "compound",
            ::append::rolling_file::policy::compound::CompoundPolicyDeserializer,
        );

        #[cfg(feature = "delete_roller")]
        d.insert(
            "delete",
            ::append::rolling_file::policy::compound::roll::delete::DeleteRollerDeserializer,
        );

        #[cfg(feature = "fixed_window_roller")]
        d.insert(
            "fixed_window",
            ::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRollerDeserializer,
        );

        #[cfg(feature = "size_trigger")]
        d.insert(
            "size",
            ::append::rolling_file::policy::compound::trigger::size::SizeTriggerDeserializer,
        );

        #[cfg(feature = "json_encoder")]
        d.insert("json", ::encode::json::JsonEncoderDeserializer);

        #[cfg(feature = "pattern_encoder")]
        d.insert("pattern", ::encode::pattern::PatternEncoderDeserializer);

        #[cfg(feature = "threshold_filter")]
        d.insert(
            "threshold",
            ::filter::threshold::ThresholdFilterDeserializer,
        );

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
    pub fn deserialize<T: ?Sized>(
        &self,
        kind: &str,
        config: Value,
    ) -> Result<Box<T>, Box<error::Error + Sync + Send>>
    where
        T: Deserializable,
    {
        match self.0.get::<KeyAdaptor<T>>().and_then(|m| m.get(kind)) {
            Some(b) => b.deserialize(config, self),
            None => Err(format!(
                "no {} deserializer for kind `{}` registered",
                T::name(),
                kind
            ).into()),
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
            ErrorKind::Filter(ref name) => write!(
                fmt,
                "error deserializing filter attached to appender {}: {}",
                name,
                self.1
            ),
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

/// A raw deserializable log4rs configuration for xml.
#[cfg(feature = "xml_format")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfigXml {
    #[serde(deserialize_with = "de_duration", default)] refresh_rate: Option<Duration>,
    #[serde(default)] root: Root,
    #[serde(default)] appenders: HashMap<String, AppenderConfig>,
    #[serde(rename = "loggers", default)] loggers: LoggersXml,
}

/// Loggers section wrapper for xml configuration
#[cfg(feature = "xml_format")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggersXml {
    #[serde(rename = "logger", default)] loggers: Vec<LoggerXml>,
}

#[cfg(feature = "xml_format")]
impl Default for LoggersXml {
    fn default() -> Self {
        Self { loggers: vec![] }
    }
}

/// A raw deserializable log4rs configuration.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    #[serde(deserialize_with = "de_duration", default)] refresh_rate: Option<Duration>,
    #[serde(default)] root: Root,
    #[serde(default)] appenders: HashMap<String, AppenderConfig>,
    #[serde(default)] loggers: HashMap<String, Logger>,
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
            })
            .collect()
    }

    /// Returns the appenders.
    ///
    /// Any components which fail to be deserialized will be ignored.
    pub fn appenders_lossy(
        &self,
        deserializers: &Deserializers,
    ) -> (Vec<config::Appender>, Vec<Error>) {
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

#[cfg(feature = "xml_format")]
impl ::std::convert::From<RawConfigXml> for RawConfig {
    fn from(cfg: RawConfigXml) -> Self {
        Self {
            refresh_rate: cfg.refresh_rate,
            root: cfg.root,
            appenders: cfg.appenders,
            loggers: cfg.loggers
                .loggers
                .into_iter()
                .map(|l| (l.name.clone(), l.into()))
                .collect(),
        }
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
                    humantime::parse_duration(v)
                        .map(S)
                        .map_err(|e| E::custom(e))
                }
            }

            d.deserialize_any(V)
        }
    }

    Option::<S>::deserialize(d).map(|r| r.map(|s| s.0))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    #[serde(default = "root_level_default")]
    level: LevelFilter,
    #[serde(default)] appenders: Vec<String>,
}

impl Default for Root {
    fn default() -> Root {
        Root {
            level: root_level_default(),
            appenders: vec![],
        }
    }
}

fn root_level_default() -> LevelFilter {
    LevelFilter::Debug
}

/// logger struct for xml configuration
#[cfg(feature = "xml_format")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoggerXml {
    /// explicit field "name" for xml config
    name: String,

    level: LevelFilter,

    #[serde(default)] appenders: Vec<String>,

    #[serde(default = "logger_additive_default")] additive: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Logger {
    level: LevelFilter,
    #[serde(default)] appenders: Vec<String>,
    #[serde(default = "logger_additive_default")] additive: bool,
}

#[cfg(feature = "xml_format")]
impl ::std::convert::From<LoggerXml> for Logger {
    fn from(logger_xml: LoggerXml) -> Self {
        Logger {
            level: logger_xml.level,
            appenders: logger_xml.appenders,
            additive: logger_xml.additive,
        }
    }
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
        ::serde_yaml::from_str::<RawConfig>("{}").unwrap();
    }

    #[test]
    #[cfg(feature = "xml_format")]
    fn full_deserialize_xml() {
        let cfg = r#"
<?xml version="1.0" encoding="utf-8"?>
<configuration refresh_rate="30 seconds">
    <appenders>
        <stdout kind="console"/>
        <requests kind="file" path="log/requests.log">
            <encoder pattern="{d} - {m}{n}" />
        </requests>
    </appenders>
    <root level="warn">
        <appenders>stdout</appenders>
    </root>
    <loggers>
        <logger name="foo::bar::baz" level="trace" additive="false" >
            <appenders>requests</appenders>
        </logger>
    </loggers>
</configuration>
"#;
        let config: RawConfigXml = ::serde_xml_rs::deserialize(cfg.as_bytes()).unwrap();
        let config: RawConfig = config.into();
        let errors = config.appenders_lossy(&Deserializers::new()).1;
        println!("{:?}", errors);
        assert!(errors.is_empty());
        assert_eq!(config.refresh_rate, Some(Duration::from_secs(30)));

        let logger = config.loggers.get("foo::bar::baz").unwrap();
        assert_eq!(logger.appenders[0], "requests");
    }
}
