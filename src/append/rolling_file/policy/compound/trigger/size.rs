//! The size trigger.
//!
//! Requires the `size_trigger` feature.

#[cfg(feature = "config_parsing")]
use serde::de;
#[cfg(feature = "config_parsing")]
use std::fmt;

use crate::append::rolling_file::{policy::compound::trigger::Trigger, LogFile};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

/// Configuration for the size trigger.
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizeTriggerConfig {
    #[serde(deserialize_with = "deserialize_limit")]
    limit: u64,
}

#[cfg(feature = "config_parsing")]
fn deserialize_limit<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct V;

    impl<'de2> de::Visitor<'de2> for V {
        type Value = u64;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("a size")
        }

        fn visit_u64<E>(self, v: u64) -> Result<u64, E>
        where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> Result<u64, E>
        where
            E: de::Error,
        {
            if v < 0 {
                return Err(E::invalid_value(
                    de::Unexpected::Signed(v),
                    &"a non-negative number",
                ));
            }

            Ok(v as u64)
        }

        fn visit_str<E>(self, v: &str) -> Result<u64, E>
        where
            E: de::Error,
        {
            let (number, unit) = match v.find(|c: char| !c.is_ascii_digit()) {
                Some(n) => (v[..n].trim(), Some(v[n..].trim())),
                None => (v.trim(), None),
            };

            let number = match number.parse::<u64>() {
                Ok(n) => n,
                Err(_) => return Err(E::invalid_value(de::Unexpected::Str(number), &"a number")),
            };

            let unit = match unit {
                Some(u) => u,
                None => return Ok(number),
            };

            let number = if unit.eq_ignore_ascii_case("b") {
                Some(number)
            } else if unit.eq_ignore_ascii_case("kb") || unit.eq_ignore_ascii_case("kib") {
                number.checked_mul(1024)
            } else if unit.eq_ignore_ascii_case("mb") || unit.eq_ignore_ascii_case("mib") {
                number.checked_mul(1024 * 1024)
            } else if unit.eq_ignore_ascii_case("gb") || unit.eq_ignore_ascii_case("gib") {
                number.checked_mul(1024 * 1024 * 1024)
            } else if unit.eq_ignore_ascii_case("tb") || unit.eq_ignore_ascii_case("tib") {
                number.checked_mul(1024 * 1024 * 1024 * 1024)
            } else {
                return Err(E::invalid_value(de::Unexpected::Str(unit), &"a valid unit"));
            };

            match number {
                Some(n) => Ok(n),
                None => Err(E::invalid_value(de::Unexpected::Str(v), &"a byte size")),
            }
        }
    }

    d.deserialize_any(V)
}

/// A trigger which rolls the log once it has passed a certain size.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct SizeTrigger {
    limit: u64,
}

impl SizeTrigger {
    /// Returns a new trigger which rolls the log once it has passed the
    /// specified size in bytes.
    pub fn new(limit: u64) -> SizeTrigger {
        SizeTrigger { limit }
    }
}

impl Trigger for SizeTrigger {
    fn trigger(&self, file: &LogFile) -> anyhow::Result<bool> {
        Ok(file.len_estimate() > self.limit)
    }

    fn is_pre_process(&self) -> bool {
        false
    }
}

/// A deserializer for the `SizeTrigger`.
///
/// # Configuration
///
/// ```yaml
/// kind: size
///
/// # The size limit in bytes. The following units are supported (case insensitive):
/// # "b", "kb", "kib", "mb", "mib", "gb", "gib", "tb", "tib". The unit defaults to
/// # bytes if not specified. Required.
/// limit: 10 mb
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct SizeTriggerDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for SizeTriggerDeserializer {
    type Trait = dyn Trigger;

    type Config = SizeTriggerConfig;

    fn deserialize(
        &self,
        config: SizeTriggerConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Trigger>> {
        Ok(Box::new(SizeTrigger::new(config.limit)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "config_parsing")]
    use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

    #[cfg(feature = "config_parsing")]
    static BYTE_MULTIPLIER: u64 = 1024;

    #[test]
    fn test_pre_process() {
        let trigger = SizeTrigger::new(2048);
        assert!(!trigger.is_pre_process());
    }

    #[test]
    fn test_trigger() {
        let file = tempfile::tempdir().unwrap();
        let mut logfile = LogFile {
            writer: &mut None,
            path: file.path(),
            len: 0,
        };

        let trigger_bytes = 5;
        let trigger = SizeTrigger::new(trigger_bytes);

        // Logfile size is < trigger size, should never trigger
        for size in 0..trigger_bytes {
            logfile.len = size;
            assert!(!trigger.trigger(&logfile).unwrap());
        }

        // Logfile size is == trigger size, should not trigger
        logfile.len = trigger_bytes;
        assert!(!trigger.trigger(&logfile).unwrap());

        // Logfile size is >= trigger size, should trigger
        logfile.len = trigger_bytes + 1;
        assert!(trigger.trigger(&logfile).unwrap());
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_u64_deserialize() {
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER,
        };
        assert_de_tokens(
            &trigger,
            &[
                Token::Struct {
                    name: "SizeTriggerConfig",
                    len: 1,
                },
                Token::Str("limit"),
                Token::U64(1024),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_i64_deserialize() {
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER,
        };

        let mut cfg = vec![
            Token::Struct {
                name: "SizeTriggerConfig",
                len: 1,
            },
            Token::Str("limit"),
            Token::I64(1024),
            Token::StructEnd,
        ];

        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::I64(-1024);
        assert_de_tokens_error::<SizeTriggerConfig>(
            &cfg,
            "invalid value: integer `-1024`, expected a non-negative number",
        );
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_float_deserialize() {
        assert_de_tokens_error::<SizeTriggerConfig>(
            &[
                Token::Struct {
                    name: "SizeTriggerConfig",
                    len: 1,
                },
                Token::Str("limit"),
                Token::F32(2.0),
                Token::StructEnd,
            ],
            "invalid type: floating point `2.0`, expected a size",
        );
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_str_deserialize() {
        // Test no unit (aka value in Bytes)
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER,
        };

        let mut cfg = vec![
            Token::Struct {
                name: "SizeTriggerConfig",
                len: 1,
            },
            Token::Str("limit"),
            Token::Str("1024"),
            Token::StructEnd,
        ];

        assert_de_tokens(&trigger, &cfg);

        // Test not an unsigned number
        cfg[2] = Token::Str("-1024");
        assert_de_tokens_error::<SizeTriggerConfig>(
            &cfg,
            "invalid value: string \"\", expected a number",
        );

        // Test not a valid unit
        cfg[2] = Token::Str("1024 pb");
        assert_de_tokens_error::<SizeTriggerConfig>(
            &cfg,
            "invalid value: string \"pb\", expected a valid unit",
        );

        // u64::MAX which will overflow when converted to bytes
        cfg[2] = Token::Str("18446744073709551615 kb");
        assert_de_tokens_error::<SizeTriggerConfig>(
            &cfg,
            "invalid value: string \"18446744073709551615 kb\", expected a byte size",
        );
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_byte_deserialize() {
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER,
        };

        let mut cfg = vec![
            Token::Struct {
                name: "SizeTriggerConfig",
                len: 1,
            },
            Token::Str("limit"),
            Token::Str("1024b"),
            Token::StructEnd,
        ];

        // Test spacing & b vs B
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1024 B");
        assert_de_tokens(&trigger, &cfg);
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_kilobyte_deserialize() {
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER,
        };

        let mut cfg = vec![
            Token::Struct {
                name: "SizeTriggerConfig",
                len: 1,
            },
            Token::Str("limit"),
            Token::Str("1 kb"),
            Token::StructEnd,
        ];

        // Test kb unit
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 KB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 kB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 Kb");
        assert_de_tokens(&trigger, &cfg);
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_megabyte_deserialize() {
        // Test mb unit
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER.pow(2),
        };

        let mut cfg = vec![
            Token::Struct {
                name: "SizeTriggerConfig",
                len: 1,
            },
            Token::Str("limit"),
            Token::Str("1 mb"),
            Token::StructEnd,
        ];

        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 MB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 mB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 Mb");
        assert_de_tokens(&trigger, &cfg);
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_gigabyte_deserialize() {
        // Test gb unit
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER.pow(3),
        };

        let mut cfg = vec![
            Token::Struct {
                name: "SizeTriggerConfig",
                len: 1,
            },
            Token::Str("limit"),
            Token::Str("1 gb"),
            Token::StructEnd,
        ];

        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 GB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 gB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 Gb");
        assert_de_tokens(&trigger, &cfg);
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_terabyte_deserialize() {
        // Test tb unit
        let trigger = SizeTriggerConfig {
            limit: BYTE_MULTIPLIER.pow(4),
        };

        let mut cfg = vec![
            Token::Struct {
                name: "SizeTriggerConfig",
                len: 1,
            },
            Token::Str("limit"),
            Token::Str("1 tb"),
            Token::StructEnd,
        ];

        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 TB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 tB");
        assert_de_tokens(&trigger, &cfg);

        cfg[2] = Token::Str("1 Tb");
        assert_de_tokens(&trigger, &cfg);
    }
}
