//! Rollers

use std::{fmt, path::Path};

#[cfg(feature = "config_parsing")]
use crate::config::Deserializable;

#[cfg(feature = "delete_roller")]
pub mod delete;
#[cfg(feature = "fixed_window_roller")]
pub mod fixed_window;
#[cfg(feature = "fixed_window_roller")]
mod pattern;

/// A trait which processes log files after they have been rolled over.
pub trait Roll: fmt::Debug + Send + Sync + 'static {
    /// Processes the log file.
    ///
    /// At the time that this method has been called, the log file has already
    /// been closed.
    ///
    /// If this method returns successfully, there *must* no longer be a file
    /// at the specified location.
    fn roll(&self, file: &Path) -> anyhow::Result<()>;
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Roll {
    fn name() -> &'static str {
        "roller"
    }
}
