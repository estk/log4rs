//! Triggers

use std::error::Error;
use std::fmt;

use append::rolling_file::LogFile;
#[cfg(feature = "file")]
use file::Deserializable;

#[cfg(feature = "size_trigger")]
pub mod size;

/// A trait which identifies if the active log file should be rolled over.
pub trait Trigger: fmt::Debug + Send + Sync + 'static {
    /// Determines if the active log file should be rolled over.
    fn trigger(&self, file: &LogFile) -> Result<bool, Box<Error + Sync + Send>>;
}

#[cfg(feature = "file")]
impl Deserializable for Trigger {
    fn name() -> &'static str {
        "trigger"
    }
}
