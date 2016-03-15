//! The threshold filter.

use serde::de;
use log::{LogLevelFilter, LogRecord};
use std::error::Error;
use serde_value::Value;

use file::{Build, Builder};
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

/// A builder for the `ThresholdFilter`.
///
/// The `level` key is required and specifies the threshold for the filter.
pub struct ThresholdFilterBuilder;

impl Build for ThresholdFilterBuilder {
    type Trait = Filter;

    fn build(&self, config: Value, _: &Builder) -> Result<Box<Filter>, Box<Error>> {
        let config = try!(config.deserialize_into::<ThresholdFilterConfig>());
        Ok(Box::new(ThresholdFilter::new(config.level.0)))
    }
}

include!("threshold_serde.rs");
