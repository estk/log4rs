//! Rollers

use std::error::Error;
use std::fmt;
use std::path::Path;

#[cfg(feature = "file")]
use file::Deserializable;

#[cfg(feature = "delete_roller")]
pub mod delete;
#[cfg(feature = "fixed_window_roller")]
pub mod fixed_window;

/// A trait which processes log files after they have been rolled over.
pub trait Roll: fmt::Debug + Send + Sync + 'static {
    /// Processes the log file.
    ///
    /// At the time that this method has been called, the log file has already
    /// been closed.
    ///
    /// If this method returns successfully, there *must* no longer be a file
    /// at the specified location.
    fn roll(&self, file: &Path) -> Result<(), Box<Error + Sync + Send>>;
}

#[cfg(feature = "file")]
impl Deserializable for Roll {
    fn name() -> &'static str {
        "roller"
    }
}
