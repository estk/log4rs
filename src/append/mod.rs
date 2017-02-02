//! Appenders

use std::error::Error;
use std::fmt;
use log::LogRecord;

#[cfg(feature = "file")]
use file::Deserializable;

#[cfg(feature = "file_appender")]
pub mod file;
#[cfg(feature = "console_appender")]
pub mod console;
#[cfg(feature = "rolling_file_appender")]
pub mod rolling_file;

/// A trait implemented by log4rs appenders.
///
/// Appenders take a log record and processes them, for example, by writing it
/// to a file or the console.
pub trait Append: fmt::Debug + Send + Sync + 'static {
    /// Processes the provided `LogRecord`.
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error + Sync + Send>>;
}

#[cfg(feature = "file")]
impl Deserializable for Append {
    fn name() -> &'static str {
        "appender"
    }
}
