//! The threshold filter.
//!
//! Requires the `threshold_filter` feature.

use log::{LevelFilter, Record};

#[cfg(feature = "config_parsing")]
use crate::config_parsing::{Deserialize, Deserializers};
use crate::filter::{Filter, Response};

/// The threshold filter's configuration.
#[cfg(feature = "config_parsing")]
#[derive(serde::Deserialize)]
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
        ThresholdFilter { level }
    }
}

impl Filter for ThresholdFilter {
    fn filter(&self, record: &Record) -> Response {
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
#[cfg(feature = "config_parsing")]
pub struct ThresholdFilterDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for ThresholdFilterDeserializer {
    type Trait = dyn Filter;

    type Config = ThresholdFilterConfig;

    fn deserialize(
        &self,
        config: ThresholdFilterConfig,
        _: &Deserializers,
    ) -> Result<Box<dyn Filter>, failure::Error> {
        Ok(Box::new(ThresholdFilter::new(config.level)))
    }
}
