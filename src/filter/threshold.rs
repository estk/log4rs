//! The threshold filter.
//!
//! Requires the `threshold_filter` feature.

use log::{LevelFilter, Record};
#[cfg(feature = "serde_derive")]
use serde_derive::Deserialize;
#[cfg(feature = "file")]
use std::error::Error;

#[cfg(feature = "file")]
use crate::file::{Deserialize, Deserializers};
use crate::filter::{Filter, Response};

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
#[cfg(feature = "file")]
pub struct ThresholdFilterDeserializer;

#[cfg(feature = "file")]
impl Deserialize for ThresholdFilterDeserializer {
    type Trait = dyn Filter;

    type Config = ThresholdFilterConfig;

    fn deserialize(
        &self,
        config: ThresholdFilterConfig,
        _: &Deserializers,
    ) -> Result<Box<dyn Filter>, Box<dyn Error + Sync + Send>> {
        Ok(Box::new(ThresholdFilter::new(config.level)))
    }
}
