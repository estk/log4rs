//! Filters

use std::fmt;
use log::LogRecord;
#[cfg(feature = "file")]
use serde_value::Value;
#[cfg(feature = "file")]
use serde::de;
#[cfg(feature = "file")]
use std::collections::BTreeMap;

#[cfg(feature = "file")]
use file::Deserializable;

#[cfg(feature = "threshold_filter")]
pub mod threshold;

/// The trait implemented by log4rs filters.
///
/// Filters are associated with appenders and limit the log events that will be
/// sent to that appender.
pub trait Filter: fmt::Debug + Send + Sync + 'static {
    /// Filters a log event.
    fn filter(&self, record: &LogRecord) -> Response;
}

#[cfg(feature = "file")]
impl Deserializable for Filter {
    fn name() -> &'static str {
        "filter"
    }
}

/// The response returned by a filter.
pub enum Response {
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

/// Configuration for a filter.
#[derive(PartialEq, Eq, Debug)]
#[cfg(feature = "file")]
pub struct FilterConfig {
    /// The filter kind.
    pub kind: String,
    /// The filter configuration.
    pub config: Value,
}

#[cfg(feature = "file")]
impl de::Deserialize for FilterConfig {
    fn deserialize<D>(d: D) -> Result<FilterConfig, D::Error>
        where D: de::Deserializer
    {
        let mut map = try!(BTreeMap::<Value, Value>::deserialize(d));

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => try!(kind.deserialize_into().map_err(|e| e.to_error())),
            None => return Err(de::Error::missing_field("kind")),
        };

        Ok(FilterConfig {
            kind: kind,
            config: Value::Map(map),
        })
    }
}
