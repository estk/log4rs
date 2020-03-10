//! The time trigger.
//!
//! Requires the `time_trigger` feature.

#[cfg(feature = "file")]
use serde::de;
#[cfg(feature = "file")]
use serde_derive::Deserialize;
use std::error::Error;
#[cfg(feature = "file")]
use std::fmt;
use std::sync::RwLock;

use crate::append::rolling_file::policy::compound::now_string;
use crate::append::rolling_file::policy::compound::trigger::Trigger;
use crate::append::rolling_file::LogFile;

#[cfg(feature = "file")]
use crate::file::{Deserialize, Deserializers};

/// Configuration for the time trigger.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeTriggerConfig {
    #[serde(deserialize_with = "deserialize_unit")]
    unit: String,
}

#[cfg(feature = "file")]
fn deserialize_unit<'de, D>(d: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct V;

    impl<'de2> de::Visitor<'de2> for V {
        type Value = String;

        fn expecting(&self, unit: &mut fmt::Formatter) -> fmt::Result {
            unit.write_str("a time unit")
        }

        fn visit_str<E>(self, v: &str) -> Result<String, E>
        where
            E: de::Error,
        {
            match v {
                "year" => Ok("%Y".to_owned()),
                "month" => Ok("%Y%m".to_owned()),
                "day" => Ok("%Y%m%d".to_owned()),
                "hour" => Ok("%Y%m%d%H".to_owned()),
                "minute" => Ok("%Y%m%d%H%M".to_owned()),
                "second" => Ok("%Y%m%d%H%M%S".to_owned()),
                _ => Err(E::invalid_value(
                    de::Unexpected::Str(v),
                    &"invalid unit, should be one of year, month, day, hour, minute, second",
                )),
            }
        }
    }

    d.deserialize_any(V)
}

/// A trigger which rolls the log once it has passed a certain time.
#[derive(Debug)]
pub struct TimeTrigger {
    fmt: String,
    time_string: RwLock<String>,
}

impl TimeTrigger {
    /// Returns a new trigger which rolls the log once it has passed the
    /// specified time format.
    pub fn new(fmt: &str) -> TimeTrigger {
        TimeTrigger {
            fmt: fmt.to_owned(),
            time_string: RwLock::new(now_string(fmt)),
        }
    }
}

impl Trigger for TimeTrigger {
    fn trigger(&self, _file: Option<&LogFile>) -> Result<bool, Box<dyn Error + Sync + Send>> {
        let now_string = now_string(&self.fmt);
        let mut time_string = self.time_string.write().unwrap();
        let is_trigger = *time_string != now_string;
        *time_string = now_string;
        Ok(is_trigger)
    }
}

/// A deserializer for the `TimeTrigger`.
///
/// # Configuration
///
/// ```yaml
/// kind: time
///
/// The valid time unit are supported.
/// unit: year, month, day, hour, minute
/// ```
#[cfg(feature = "file")]
pub struct TimeTriggerDeserializer;

#[cfg(feature = "file")]
impl Deserialize for TimeTriggerDeserializer {
    type Trait = dyn Trigger;

    type Config = TimeTriggerConfig;

    fn deserialize(
        &self,
        config: TimeTriggerConfig,
        _: &Deserializers,
    ) -> Result<Box<dyn Trigger>, Box<dyn Error + Sync + Send>> {
        Ok(Box::new(TimeTrigger::new(&config.unit)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::append::rolling_file::policy::compound::set_mock_time;

    #[test]
    fn trigger() {
        set_mock_time("2020-03-07");
        let trigger = TimeTrigger::new("%Y%m%d");
        assert_eq!(false, trigger.trigger(Option::None).unwrap());
        set_mock_time("2020-03-08");
        assert_eq!(true, trigger.trigger(Option::None).unwrap());
    }
}
