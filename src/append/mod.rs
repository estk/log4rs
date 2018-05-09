//! Appenders

use std::error::Error;
use std::fmt;
use log::{Record, Log};
#[cfg(feature = "file")]
use serde::{de, Deserialize, Deserializer};
#[cfg(feature = "file")]
use serde_value::Value;
#[cfg(feature = "file")]
use std::collections::BTreeMap;

#[cfg(feature = "file")]
use file::Deserializable;
#[cfg(feature = "file")]
use filter::FilterConfig;
use record::ExtendedRecord;

#[cfg(feature = "file_appender")]
pub mod file;
#[cfg(feature = "console_appender")]
pub mod console;
#[cfg(feature = "rolling_file_appender")]
pub mod rolling_file;

/// A trait implemented by log4rs appenders.
///
/// Appenders take a log record and processes them, for example, by writing it
/// to a file or the console.
pub trait Append: fmt::Debug + Send + Sync + 'static {
    /// Processes the provided `Record`.
    fn append(&self, record: &ExtendedRecord) -> Result<(), Box<Error + Sync + Send>>;

    /// Flushes all in-flight records.
    fn flush(&self);
}

#[cfg(feature = "file")]
impl Deserializable for Append {
    fn name() -> &'static str {
        "appender"
    }
}

impl<T: Log + fmt::Debug + 'static> Append for T {
    fn append(&self, record: &ExtendedRecord) -> Result<(), Box<Error + Sync + Send>> {
        self.log(record.record());
        Ok(())
    }

    fn flush(&self) {
        Log::flush(self)
    }
}

/// Configuration for an appender.
#[cfg(feature = "file")]
#[derive(PartialEq, Eq, Debug)]
pub struct AppenderConfig {
    /// The appender kind.
    pub kind: String,
    /// The filters attached to the appender.
    pub filters: Vec<FilterConfig>,
    /// The appender configuration.
    pub config: Value,
}

#[cfg(feature = "file")]
impl<'de> Deserialize<'de> for AppenderConfig {
    fn deserialize<D>(d: D) -> Result<AppenderConfig, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => kind.deserialize_into().map_err(|e| e.into_error())?,
            None => return Err(de::Error::missing_field("kind")),
        };

        let filters = match map.remove(&Value::String("filters".to_owned())) {
            Some(filters) => filters.deserialize_into().map_err(|e| e.into_error())?,
            None => vec![],
        };

        Ok(AppenderConfig {
            kind: kind,
            filters: filters,
            config: Value::Map(map),
        })
    }
}
