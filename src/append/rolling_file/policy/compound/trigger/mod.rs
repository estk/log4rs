//! Triggers

use std::fmt;

use crate::append::rolling_file::LogFile;

#[cfg(feature = "size_trigger")]
pub mod size;

/// A trait which identifies if the active log file should be rolled over.
pub trait Trigger: fmt::Debug + Send + Sync + 'static {
    /// Determines if the active log file should be rolled over.
    fn trigger(&self, file: &LogFile) -> anyhow::Result<bool>;
}

///
pub trait IntoTrigger {
    ///
    fn into_trigger(self) -> Box<dyn Trigger>;
}
