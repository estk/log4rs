//! The OnStartUp trigger.
//!
//! Requires the `onstartup_trigger` feature.

use std::sync::Once;

use crate::append::rolling_file::{policy::compound::trigger::Trigger, LogFile};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

/// Configuration for the onstartup trigger.
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnStartUpTriggerConfig {
    #[serde(default = "default_min_size")]
    min_size: u64,
}

#[cfg(feature = "config_parsing")]
fn default_min_size() -> u64 {
    1
}

/// A trigger which rolls the log on startup.
#[derive(Debug)]
pub struct OnStartUpTrigger {
    min_size: u64,
    initial: Once,
}

impl OnStartUpTrigger {
    /// Returns a new trigger which rolls the log on startup.
    pub fn new(min_size: u64) -> OnStartUpTrigger {
        OnStartUpTrigger {
            min_size,
            initial: Once::new(),
        }
    }
}

impl Trigger for OnStartUpTrigger {
    fn trigger(&self, file: &LogFile) -> anyhow::Result<bool> {
        let mut result = false;
        self.initial.call_once(|| {
            if file.len_estimate() >= self.min_size {
                result = true;
            }
        });
        Ok(result)
    }

    fn is_pre_process(&self) -> bool {
        true
    }
}

/// A deserializer for the `OnStartUpTrigger`.
///
/// # Configuration
///
/// ```yaml
/// kind: onstartup
///
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct OnStartUpTriggerDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for OnStartUpTriggerDeserializer {
    type Trait = dyn Trigger;

    type Config = OnStartUpTriggerConfig;

    fn deserialize(
        &self,
        config: OnStartUpTriggerConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Trigger>> {
        Ok(Box::new(OnStartUpTrigger::new(config.min_size)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pre_process() {
        let trigger = OnStartUpTrigger::new(0);
        assert!(trigger.is_pre_process());
    }

    fn trigger_with_file_size(file_size: u64) -> (bool, bool) {
        let file = tempfile::tempdir().unwrap();
        let logfile = LogFile {
            writer: &mut None,
            path: file.path(),
            len: file_size,
        };

        let trigger = OnStartUpTrigger::new(1); // default
        let result1 = trigger.trigger(&logfile).unwrap();
        let result2 = trigger.trigger(&logfile).unwrap();
        (result1, result2)
    }

    #[test]
    fn trigger() {
        // When the file size < min_size, the trigger should return false.
        assert_eq!(trigger_with_file_size(0), (false, false));
        // When the file size == min_size, the trigger should return true for the first time.
        assert_eq!(trigger_with_file_size(1), (true, false));
        // When the file size >= min_size, the trigger should return true for the first time.
        assert_eq!(trigger_with_file_size(2), (true, false));
    }
}
