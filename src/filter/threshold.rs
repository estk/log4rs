//! The threshold filter.

use log::{LogLevelFilter, LogRecord};
use std::error::Error;
use serde_value::Value;

use file::{Deserialize, Deserializers};
use filter::{Filter, Response};
use priv_serde::DeLogLevelFilter;

/// A filter that rejects all events at a level below a provided threshold.
#[derive(Debug)]
pub struct ThresholdFilter {
    level: LogLevelFilter,
}

impl ThresholdFilter {
    /// Creates a new `ThresholdFilter` with the specified threshold.
    pub fn new(level: LogLevelFilter) -> ThresholdFilter {
        ThresholdFilter { level: level }
    }
}

impl Filter for ThresholdFilter {
    fn filter(&self, record: &LogRecord) -> Response {
        if record.level() > self.level {
            Response::Reject
        } else {
            Response::Neutral
        }
    }
}

/// A deserializer for the `ThresholdFilter`.
///
/// The `level` key is required and specifies the threshold for the filter.
pub struct ThresholdFilterDeserializer;

impl Deserialize for ThresholdFilterDeserializer {
    type Trait = Filter;

    fn deserialize(&self,
                   config: Value,
                   _: &Deserializers)
                   -> Result<Box<Filter>, Box<Error>> {
        let config = try!(config.deserialize_into::<ThresholdFilterConfig>());
        Ok(Box::new(ThresholdFilter::new(config.level.0)))
    }
}

include!("threshold_serde.rs");
