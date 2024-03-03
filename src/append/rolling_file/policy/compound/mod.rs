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

    fn is_pre_process(&self) -> bool {
        self.trigger.is_pre_process()
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

#[cfg(test)]
mod test {
    use self::{roll::delete::DeleteRoller, trigger::size::SizeTrigger};

    use super::*;
    use tempfile::NamedTempFile;

    #[cfg(feature = "config_parsing")]
    use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

    fn create_policy() -> CompoundPolicy {
        let trigger = SizeTrigger::new(1024);
        let roller = DeleteRoller::new();
        CompoundPolicy::new(Box::new(trigger), Box::new(roller))
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_trigger_deser() {
        let mut cfg = vec![
            Token::Struct {
                name: "Trigger",
                len: 2,
            },
            Token::Str("kind"),
            Token::Str("size"),
            Token::Str("limit"),
            Token::U64(1024),
            Token::StructEnd,
        ];

        assert_de_tokens(
            &Trigger {
                kind: "size".to_owned(),
                config: Value::Map({
                    let mut map = BTreeMap::new();
                    map.insert(Value::String("limit".to_owned()), Value::U64(1024));
                    map
                }),
            },
            &cfg,
        );

        // Intentionally break the config
        cfg[1] = Token::Str("knd");
        assert_de_tokens_error::<Trigger>(&cfg, "missing field `kind`");
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_roller_deser() {
        let mut cfg = vec![
            Token::Struct {
                name: "Roller",
                len: 1,
            },
            Token::Str("kind"),
            Token::Str("delete"),
            Token::StructEnd,
        ];

        assert_de_tokens(
            &Roller {
                kind: "delete".to_owned(),
                config: Value::Map(BTreeMap::new()),
            },
            &cfg,
        );

        // Intentionally break the config
        cfg[1] = Token::Str("knd");
        assert_de_tokens_error::<Roller>(&cfg, "missing field `kind`");
    }

    #[test]
    fn test_pre_process() {
        let policy = create_policy();
        assert!(!policy.is_pre_process());
    }

    #[test]
    fn test_process() {
        let policy = create_policy();
        // Don't roll then roll
        let file_sizes = vec![0, 2048];
        let tmp_file = NamedTempFile::new().unwrap();

        for file_size in file_sizes {
            let mut logfile = LogFile {
                writer: &mut None,
                path: tmp_file.as_ref(),
                len: file_size,
            };
            assert!(policy.process(&mut logfile).is_ok());
        }
    }
}
