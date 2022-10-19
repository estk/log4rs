//! The compound rolling policy.
//!
//! Requires the `compound_policy` feature.
#[cfg(feature = "config_parsing")]
use serde::{self, de};
#[cfg(feature = "config_parsing")]
use serde_value::Value;
#[cfg(feature = "config_parsing")]
use std::collections::BTreeMap;
use std::path::Path;

use crate::append::rolling_file::{policy::Policy, LogFile};
#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

pub mod roll;
pub mod trigger;

pub use roll::Roll;

/// Configuration for the compound policy.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompoundPolicyConfig {
    trigger: Trigger,
    roller: Roller,
}

#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct Trigger {
    kind: String,
    config: Value,
}

#[cfg(feature = "config_parsing")]
impl<'de> serde::Deserialize<'de> for Trigger {
    fn deserialize<D>(d: D) -> Result<Trigger, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => kind.deserialize_into().map_err(|e| e.to_error())?,
            None => return Err(de::Error::missing_field("kind")),
        };

        Ok(Trigger {
            kind,
            config: Value::Map(map),
        })
    }
}

#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct Roller {
    kind: String,
    config: Value,
}

#[cfg(feature = "config_parsing")]
impl<'de> serde::Deserialize<'de> for Roller {
    fn deserialize<D>(d: D) -> Result<Roller, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => kind.deserialize_into().map_err(|e| e.to_error())?,
            None => return Err(de::Error::missing_field("kind")),
        };

        Ok(Roller {
            kind,
            config: Value::Map(map),
        })
    }
}

/// A rolling policy which delegates to a "trigger" and "roller".
///
/// The trigger determines if the log file should roll, for example, by checking
/// the size of the file. The roller processes the old log file, for example,
/// by compressing it and moving it to a different location.
#[derive(Debug)]
pub struct CompoundPolicy {
    trigger: Box<dyn trigger::Trigger>,
    roller: Box<dyn Roll>,
}

impl CompoundPolicy {
    /// Creates a new `CompoundPolicy`.
    pub fn new(trigger: Box<dyn trigger::Trigger>, roller: Box<dyn Roll>) -> CompoundPolicy {
        CompoundPolicy { trigger, roller }
    }
}

impl Policy for CompoundPolicy {
    fn process(&self, log: &mut LogFile) -> anyhow::Result<()> {
        if self.trigger.trigger(log)? {
            log.roll();
            self.roller.roll(log.path())?;
        }
        Ok(())
    }

    fn startup(&self, path: &Path) -> anyhow::Result<()> {
        self.roller.roll(path)?;
        Ok(())
    }
}

/// A deserializer for the `CompoundPolicyDeserializer`.
///
/// # Configuration
///
/// ```yaml
/// kind: compound
///
/// # The trigger, which determines when the log will roll over. Required.
/// trigger:
///
///   # Identifies which trigger is to be used. Required.
///   kind: size
///
///   # The remainder of the configuration is passed to the trigger's
///   # deserializer, and will vary based on the kind of trigger.
///   limit: 10 mb
///
/// # The roller, which processes the old log file. Required.
/// roller:
///
///   # Identifies which roller is to be used. Required.
///   kind: delete
///
///   # The remainder of the configuration is passed to the roller's
///   # deserializer, and will vary based on the kind of roller.
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct CompoundPolicyDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for CompoundPolicyDeserializer {
    type Trait = dyn Policy;

    type Config = CompoundPolicyConfig;

    fn deserialize(
        &self,
        config: CompoundPolicyConfig,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<dyn Policy>> {
        let trigger = deserializers.deserialize(&config.trigger.kind, config.trigger.config)?;
        let roller = deserializers.deserialize(&config.roller.kind, config.roller.config)?;
        Ok(Box::new(CompoundPolicy::new(trigger, roller)))
    }
}
