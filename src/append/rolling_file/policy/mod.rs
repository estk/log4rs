//! Policies.
use std::error::Error;
use std::fmt;

use append::rolling_file::LogFile;
#[cfg(feature = "file")]
use file::Deserializable;

#[cfg(feature = "compound_policy")]
pub mod compound;

/// A trait implementing a rolling policy for a `RollingFileAppender`.
pub trait Policy: Sync + Send + 'static + fmt::Debug {
    /// Rolls the current log file, if necessary.
    ///
    /// This method is called after each log event. It is provided a reference
    /// to the current log file.
    fn process(&self, log: &mut LogFile) -> Result<(), Box<Error + Sync + Send>>;
}

#[cfg(feature = "file")]
impl Deserializable for Policy {
    fn name() -> &'static str {
        "policy"
    }
}
