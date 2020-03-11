//! Triggers

use std::fmt;

use failure::Error;

use crate::append::rolling_file::LogFile;
#[cfg(feature = "file")]
use crate::file::Deserializable;

#[cfg(feature = "size_trigger")]
pub mod size;

/// A trait which identifies if the active log file should be rolled over.
pub trait Trigger: fmt::Debug + Send + Sync + 'static {
    /// Determines if the active log file should be rolled over.
    fn trigger(&self, file: &LogFile) -> Result<bool, Error>;
}

#[cfg(feature = "file")]
impl Deserializable for dyn Trigger {
    fn name() -> &'static str {
        "trigger"
    }
}
