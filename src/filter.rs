//! A set of common filters.
use log::{LogRecord, LogLevelFilter};

use {Filter, FilterResponse};

/// A filter that rejects all events at a level below a provided threshold.
pub struct ThresholdFilter {
    level: LogLevelFilter,
}

impl ThresholdFilter {
    /// Creates a new `ThresholdFilter` with the specified threshold.
    pub fn new(level: LogLevelFilter) -> ThresholdFilter {
        ThresholdFilter {
            level: level
        }
    }
}

impl Filter for ThresholdFilter {
    fn filter(&mut self, record: &LogRecord) -> FilterResponse {
        if record.level() > self.level {
            FilterResponse::Reject
        } else {
            FilterResponse::Neutral
        }
    }
}
