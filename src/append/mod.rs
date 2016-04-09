//! Appenders

use std::error::Error;
use std::fmt;
use log::LogRecord;

pub mod file;
pub mod console;

/// A trait implemented by log4rs appenders.
pub trait Append: fmt::Debug + Send + Sync + 'static {
    /// Processes the provided `LogRecord`.
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>>;
}
