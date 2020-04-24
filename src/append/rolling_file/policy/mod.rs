//! Policies.
use std::fmt;

use crate::append::rolling_file::LogFile;

#[cfg(feature = "config_parsing")]
use crate::config::Deserializable;

#[cfg(feature = "compound_policy")]
pub mod compound;

/// A trait implementing a rolling policy for a `RollingFileAppender`.
pub trait Policy: Sync + Send + 'static + fmt::Debug {
    /// Rolls the current log file, if necessary.
    ///
    /// This method is called after each log event. It is provided a reference
    /// to the current log file.
    fn process(&self, log: &mut LogFile) -> anyhow::Result<()>;
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Policy {
    fn name() -> &'static str {
        "policy"
    }
}
