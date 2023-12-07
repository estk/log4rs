//! Triggers

use std::fmt;

use crate::append::rolling_file::LogFile;
#[cfg(feature = "config_parsing")]
use crate::config::Deserializable;

#[cfg(feature = "size_trigger")]
pub mod size;

#[cfg(feature = "time_trigger")]
pub mod time;

/// A trait which identifies if the active log file should be rolled over.
pub trait Trigger: fmt::Debug + Send + Sync + 'static {
    /// Determines if the active log file should be rolled over.
    fn trigger(&self, file: &LogFile) -> anyhow::Result<bool>;

    /// Sets the is_pre_process flag for log files.
    ///
    /// Defaults to true for time triggers and false for size triggers
    fn is_pre_process(&self) -> bool;
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Trigger {
    fn name() -> &'static str {
        "trigger"
    }
}
