//! The compound rolling policy.
//!
//! Requires the `compound_policy` feature.
#[cfg(feature = "config_parsing")]
use serde::de;
#[cfg(feature = "config_parsing")]
use serde_value::Value;
#[cfg(feature = "config_parsing")]
use std::collections::BTreeMap;

use crate::append::rolling_file::{
    policy::{compound::roll::Roll, Policy},
    LogFile,
};
#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

pub mod roll;
pub mod trigger;

/// Configuration for the compound policy.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompoundPolicyConfig {
    trigger: Triggers,
    roller: Roller,
}

#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(untagged)]
enum Triggers {
    Single(Trigger),
    Multiple(Vec<Trigger>),
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
    triggers: Vec<(Box<dyn trigger::Trigger>, Option<bool>)>,
    roller: Box<dyn Roll>,
}

impl CompoundPolicy {
    /// Creates a new `CompoundPolicy`.
    pub fn new(trigger: Box<dyn trigger::Trigger>, roller: Box<dyn Roll>) -> CompoundPolicy {
        Self::new_with_triggers(vec![trigger], roller)
    }

    /// Creates a policy which rolls when any trigger fires.
    ///
    /// For multiple triggers, assigns each to its pre- or post-write phase at
    /// construction. The policy evaluates every trigger in that phase before
    /// rolling once. An empty list never rolls.
    pub fn new_with_triggers(
        triggers: Vec<Box<dyn trigger::Trigger>>,
        roller: Box<dyn Roll>,
    ) -> CompoundPolicy {
        let multiple = triggers.len() > 1;
        let triggers = triggers
            .into_iter()
            .map(|trigger| {
                let phase = multiple.then(|| trigger.is_pre_process());
                (trigger, phase)
            })
            .collect();
        CompoundPolicy { triggers, roller }
    }

    fn process_triggers(&self, log: &mut LogFile<'_>, phase: Option<bool>) -> anyhow::Result<()> {
        let mut roll = false;
        for (trigger, assigned_phase) in &self.triggers {
            if phase
                .is_none_or(|pre| assigned_phase.unwrap_or_else(|| trigger.is_pre_process()) == pre)
            {
                roll |= trigger.trigger(log)?;
            }
        }
        if roll {
            log.roll();
            self.roller.roll(log.path())?;
        }
        Ok(())
    }
}

impl Policy for CompoundPolicy {
    fn process(&self, log: &mut LogFile<'_>) -> anyhow::Result<()> {
        self.process_triggers(log, None)
    }

    fn is_pre_process(&self) -> bool {
        self.triggers
            .iter()
            .any(|(trigger, phase)| phase.unwrap_or_else(|| trigger.is_pre_process()))
    }

    fn process_pre(&self, log: &mut LogFile<'_>) -> anyhow::Result<bool> {
        if self.triggers.len() == 1 {
            let pre = self.is_pre_process();
            if pre {
                self.process(log)?;
            }
            Ok(!pre)
        } else {
            self.process_triggers(log, Some(true))?;
            Ok(true)
        }
    }

    fn process_post(&self, log: &mut LogFile<'_>) -> anyhow::Result<()> {
        if self.triggers.len() == 1 {
            self.process(log)
        } else {
            self.process_triggers(log, Some(false))
        }
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
/// # To roll when any of several triggers fires, use a nonempty list instead:
/// # trigger:
/// #   - kind: onstartup
/// #   - kind: size
/// #     limit: 10 mb
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
        let triggers = match config.trigger {
            Triggers::Single(trigger) => vec![trigger],
            Triggers::Multiple(triggers) => triggers,
        };
        anyhow::ensure!(!triggers.is_empty(), "trigger list must not be empty");
        let triggers = triggers
            .into_iter()
            .map(|trigger| deserializers.deserialize(&trigger.kind, trigger.config))
            .collect::<anyhow::Result<_>>()?;
        let roller = deserializers.deserialize(&config.roller.kind, config.roller.config)?;
        Ok(Box::new(CompoundPolicy::new_with_triggers(
            triggers, roller,
        )))
    }
}
