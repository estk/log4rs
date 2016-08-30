//! Appenders

use std::error::Error;
use std::fmt;
use log::LogRecord;

use file::Deserializable;

pub mod file;
pub mod console;
pub mod rolling_file;

/// A trait implemented by log4rs appenders.
///
/// Appenders take a log record and processes them, for example, by writing it
/// to a file or the console.
pub trait Append: fmt::Debug + Send + Sync + 'static {
    /// Processes the provided `LogRecord`.
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>>;
}

impl Deserializable for Append {
    fn name() -> &'static str {
        "appender"
    }
}
