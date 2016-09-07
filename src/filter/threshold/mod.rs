//! The threshold filter.
//!
//! Requires the `threshold_filter` feature.

use log::{LogLevelFilter, LogRecord};
use std::error::Error;

use file::{Deserialize, Deserializers};
use filter::{Filter, Response};

include!("serde.rs");

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
/// # Configuration
///
/// ```yaml
/// kind: threshold
///
/// # The threshold log level to filter at. Required
/// level: warn
/// ```
pub struct ThresholdFilterDeserializer;

impl Deserialize for ThresholdFilterDeserializer {
    type Trait = Filter;

    type Config = ThresholdFilterConfig;

    fn deserialize(&self,
                   config: ThresholdFilterConfig,
                   _: &Deserializers)
                   -> Result<Box<Filter>, Box<Error>> {
        Ok(Box::new(ThresholdFilter::new(config.level)))
    }
}
