//! The threshold filter.
//!
//! Requires the `threshold_filter` feature.

use log::{LevelFilter, Record};
#[cfg(feature = "file")]
use std::error::Error;

#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};
use filter::{Filter, Response};
use record::ExtendedRecord;

/// The threshold filter's configuration.
#[cfg(feature = "file")]
#[derive(Deserialize)]
pub struct ThresholdFilterConfig {
    level: LevelFilter,
}

/// A filter that rejects all events at a level below a provided threshold.
#[derive(Debug)]
pub struct ThresholdFilter {
    level: LevelFilter,
}

impl ThresholdFilter {
    /// Creates a new `ThresholdFilter` with the specified threshold.
    pub fn new(level: LevelFilter) -> ThresholdFilter {
        ThresholdFilter { level: level }
    }
}

impl Filter for ThresholdFilter {
    fn filter(&self, record: &ExtendedRecord) -> Response {
        if record.record().level() > self.level {
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
#[cfg(feature = "file")]
pub struct ThresholdFilterDeserializer;

#[cfg(feature = "file")]
impl Deserialize for ThresholdFilterDeserializer {
    type Trait = Filter;

    type Config = ThresholdFilterConfig;

    fn deserialize(
        &self,
        config: ThresholdFilterConfig,
        _: &Deserializers,
    ) -> Result<Box<Filter>, Box<Error + Sync + Send>> {
        Ok(Box::new(ThresholdFilter::new(config.level)))
    }
}
