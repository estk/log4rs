//! Filters
use log::{LogRecord, LogLevelFilter};

/// The trait implemented by log4rs filters.
pub trait Filter: Send + 'static {
    /// Filters a log event.
    fn filter(&mut self, record: &LogRecord) -> Response;
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

/// A filter that rejects all events at a level below a provided threshold.
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
    fn filter(&mut self, record: &LogRecord) -> Response {
        if record.level() > self.level {
            Response::Reject
        } else {
            Response::Neutral
        }
    }
}
