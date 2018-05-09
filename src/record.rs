use std::cell::Cell;

use chrono::{DateTime, Utc};
use log::Record;

/// Wrapper around log::Record.
///
/// Stores additional information not provided by `log::Record`.
#[derive(Debug)]
pub struct ExtendedRecord<'a> {
    record: &'a Record<'a>,
    lazy_timestamp: Cell<Option<DateTime<Utc>>>,
}

impl<'a> ExtendedRecord<'a> {
    /// Construct new `ExtendedRecord` based on a reference to the `log::Record`.
    pub fn new(record: &'a Record) -> Self {
        Self {
            record,
            lazy_timestamp: Cell::default(),
        }
    }

    /// Return reference to wrapped `log::Record`.
    pub fn record(&self) -> &Record {
        self.record
    }

    /// Return timestamp for this log record.
    ///
    /// Timestamp is not provided by the base `log::Record`, so on the first call this function creates new timestamp
    /// based on the current time, and returns the same value on all subsequent calls.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self.lazy_timestamp.get() {
            Some(timestamp) => timestamp,
            None => {
                let timestamp = Utc::now();
                self.lazy_timestamp.set(Some(timestamp));
                timestamp
            }
        }
    }
}
