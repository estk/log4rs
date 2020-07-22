//! The threshold filter.
//!
//! Requires the `threshold_filter` feature.

use log::{LevelFilter, Record};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};
use crate::filter::{Filter, Response};

/// The threshold filter's configuration.
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
pub struct ThresholdFilterConfig {
    level: LevelFilter,
}

/// A filter that rejects all events at a level below a provided threshold.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ThresholdFilterDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for ThresholdFilterDeserializer {
    type Trait = dyn Filter;

    type Config = ThresholdFilterConfig;

    fn deserialize(
        &self,
        config: ThresholdFilterConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Filter>> {
        Ok(Box::new(ThresholdFilter::new(config.level)))
    }
}
