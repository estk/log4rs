//! The compound rolling policy.
//!
//! Requires the `compound_policy` feature.
#[cfg(feature = "config_parsing")]
use serde;

#[cfg(feature = "config_parsing")]
use self::{
    roll::{delete::DeleteRollerConfig, fixed_window::FixedWindowRollerConfig, IntoRoller},
    trigger::{size::SizeTriggerConfig, IntoTrigger},
};
#[cfg(feature = "config_parsing")]
use super::IntoPolicy;
use crate::append::rolling_file::{
    policy::{compound::roll::Roll, Policy},
    LogFile,
};

pub mod roll;
pub mod trigger;

/// Configuration for the compound policy.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompoundPolicyConfig {
    trigger: TriggerConfig,
    roller: RollerConfig,
}

#[cfg(feature = "config_parsing")]
impl IntoPolicy for CompoundPolicyConfig {
    fn into_policy(self) -> anyhow::Result<Box<dyn Policy>> {
        let trigger = self.trigger.into_trigger();
        let roller = self.roller.into_roller()?;
        Ok(Box::new(CompoundPolicy::new(trigger, roller)))
    }
}

#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(tag = "kind")]
enum TriggerConfig {
    #[cfg(feature = "size_trigger")]
    #[serde(rename = "size")]
    SizeTrigger(SizeTriggerConfig),
}

#[cfg(feature = "config_parsing")]
impl IntoTrigger for TriggerConfig {
    fn into_trigger(self) -> Box<dyn trigger::Trigger> {
        match self {
            TriggerConfig::SizeTrigger(s) => s.into_trigger(),
        }
    }
}

// #[cfg(feature = "config_parsing")]
// impl<'de> serde::Deserialize<'de> for Trigger {
//     fn deserialize<D>(d: D) -> Result<Trigger, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

//         let kind = match map.remove(&Value::String("kind".to_owned())) {
//             Some(kind) => kind.deserialize_into().map_err(|e| e.to_error())?,
//             None => return Err(de::Error::missing_field("kind")),
//         };

//         Ok(Trigger {
//             kind,
//             config: Value::Map(map),
//         })
//     }
// }

#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(tag = "kind")]
enum RollerConfig {
    #[cfg(feature = "delete_roller")]
    #[serde(rename = "delete")]
    DeleteRoller(DeleteRollerConfig),

    #[cfg(feature = "fixed_window_roller")]
    #[serde(rename = "fixed_window")]
    FixedWindowRoller(FixedWindowRollerConfig),
}

#[cfg(feature = "config_parsing")]
impl IntoRoller for RollerConfig {
    fn into_roller(self) -> anyhow::Result<Box<dyn Roll>> {
        match self {
            RollerConfig::DeleteRoller(d) => d.into_roller(),
            RollerConfig::FixedWindowRoller(f) => f.into_roller(),
        }
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
