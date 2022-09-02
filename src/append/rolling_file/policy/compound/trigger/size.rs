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
