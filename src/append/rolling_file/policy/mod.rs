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
    /// The default phase methods call this before or after each log event,
    /// according to `is_pre_process`.
    fn process(&self, log: &mut LogFile<'_>) -> anyhow::Result<()>;
    /// Returns whether the policy processes log files before writing events.
    ///
    /// Policies supporting both phases should override `process_pre` and `process_post`.
    fn is_pre_process(&self) -> bool;

    /// Processes the log before writing an event.
    ///
    /// By default, selects the phase once using `is_pre_process` and calls
    /// `process` here if it returns true.
    ///
    /// Returns whether `process_post` should run after writing the event.
    fn process_pre(&self, log: &mut LogFile<'_>) -> anyhow::Result<bool> {
        let pre = self.is_pre_process();
        if pre {
            self.process(log)?;
        }
        Ok(!pre)
    }

    /// Processes the log after writing an event.
    ///
    /// Called only when `process_pre` returns true. Delegates to `process` by default.
    fn process_post(&self, log: &mut LogFile<'_>) -> anyhow::Result<()> {
        self.process(log)
    }
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Policy {
    fn name() -> &'static str {
        "policy"
    }
}
