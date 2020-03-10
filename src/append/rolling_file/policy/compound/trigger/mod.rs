//! Triggers

use std::error::Error;
use std::option::Option;
use std::fmt;

use crate::append::rolling_file::LogFile;
#[cfg(feature = "file")]
use crate::file::Deserializable;

#[cfg(feature = "size_trigger")]
pub mod size;
#[cfg(feature = "time_trigger")]
pub mod time;

/// A trait which identifies if the active log file should be rolled over.
pub trait Trigger: fmt::Debug + Send + Sync + 'static {
    /// Determines if the active log file should be rolled over.
    fn trigger(&self, file: Option<&LogFile>) -> Result<bool, Box<dyn Error + Sync + Send>>;
}

#[cfg(feature = "file")]
impl Deserializable for dyn Trigger {
    fn name() -> &'static str {
        "trigger"
    }
}
