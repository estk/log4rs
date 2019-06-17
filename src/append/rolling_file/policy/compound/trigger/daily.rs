//! The daily trigger.
//!
//! Requires the `daily_trigger` feature.

#[cfg(feature = "file")]
use serde::de;
#[cfg(feature = "file")]
use std::ascii::AsciiExt;
use std::error::Error;
#[cfg(feature = "file")]
use std::fmt;

use chrono::{Datelike, Local};

use append::rolling_file::LogFile;
use append::rolling_file::policy::compound::trigger::Trigger;
#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};

static mut DAY: u32 = 0;

/// Configuration for the daily trigger.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DailyTriggerConfig {
}

/// A trigger which rolls the log once it has passed a certain size.
#[derive(Debug)]
pub struct DailyTrigger {
}

impl DailyTrigger {
    /// Returns a new trigger which rolls the log once it has passed the
    /// specified size in bytes.
    pub fn new() -> DailyTrigger {
        DailyTrigger {}
    }
}

impl Trigger for DailyTrigger {
    fn trigger(&self, file: &LogFile) -> Result<bool, Box<Error + Sync + Send>> {
        let mut roll = false;
        unsafe {
            let last_day = DAY;
            DAY = Local::today().day();
            roll = last_day != 0 && DAY != last_day;
        }
        Ok(roll)
    }
}

/// A deserializer for the `DailyTrigger`.
///
/// # Configuration
///
#[cfg(feature = "file")]
pub struct DailyTriggerDeserializer;

#[cfg(feature = "file")]
impl Deserialize for DailyTriggerDeserializer {
    type Trait = Trigger;

    type Config = DailyTriggerConfig;

    fn deserialize(
        &self,
        config: DailyTriggerConfig,
        _: &Deserializers,
    ) -> Result<Box<Trigger>, Box<Error + Sync + Send>> {
        Ok(Box::new(DailyTrigger::new()))
    }
}
