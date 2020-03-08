//! The time trigger.
//!
//! Requires the `time_trigger` feature.

use chrono::Local;
#[cfg(feature = "file")]
use serde::de;
#[cfg(feature = "file")]
use serde_derive::Deserialize;
use std::error::Error;
#[cfg(feature = "file")]
use std::fmt;
use std::sync::RwLock;

use crate::append::rolling_file::policy::compound::trigger::Trigger;
use crate::append::rolling_file::LogFile;

#[cfg(feature = "file")]
use crate::file::{Deserialize, Deserializers};

/// Configuration for the time trigger.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeTriggerConfig {
    #[serde(deserialize_with = "deserialize_fmt")]
    fmt: String,
}

#[cfg(feature = "file")]
fn deserialize_fmt<'de, D>(d: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct V;

    impl<'de2> de::Visitor<'de2> for V {
        type Value = String;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("a time format")
        }

        fn visit_str<E>(self, v: &str) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(v.to_owned())
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
            time_string: RwLock::new(Local::now().format(fmt).to_string()),
        }
    }
}

impl Trigger for TimeTrigger {
    fn trigger(&self, _file: &LogFile) -> Result<bool, Box<dyn Error + Sync + Send>> {
        let now_string = Local::now().format(&self.fmt).to_string().to_owned();
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
/// The valid chrono time format are supported.
/// fmt: yyyy-MM-dd
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
        Ok(Box::new(TimeTrigger::new(&config.fmt)))
    }
}
