//! Triggers

use std::fmt;

use crate::append::rolling_file::LogFile;
#[cfg(feature = "config_parsing")]
use crate::config::Deserializable;

#[cfg(feature = "size_trigger")]
pub mod size;

/// A trait which identifies if the active log file should be rolled over.
pub trait Trigger: fmt::Debug + Send + Sync + 'static {
    /// Determines if the active log file should be rolled over.
    fn trigger(&self, file: &LogFile) -> anyhow::Result<bool>;
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Trigger {
    fn name() -> &'static str {
        "trigger"
    }
}
