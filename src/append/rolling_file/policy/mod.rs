//! Policies.
use std::fmt;
use std::path::Path;

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

    /// The function that gets called when the policy is built aka. when
    /// the logger gets initialized.
    ///
    /// `path` is the path to the current logfile.
    /// This log file is opened by the builder and therefore not writable.
    ///
    /// Default implementation here for backwards compatibility reasons
    fn startup(&self, _path: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Policy {
    fn name() -> &'static str {
        "policy"
    }
}
