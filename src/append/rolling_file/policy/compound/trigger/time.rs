//! The time trigger.
//!
//! Requires the `time_trigger` feature.

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};
#[cfg(feature = "config_parsing")]
use serde::de;
#[cfg(feature = "config_parsing")]
use std::fmt;
use std::sync::RwLock;

use crate::append::rolling_file::{policy::compound::trigger::Trigger, LogFile};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

/// Configuration for the time trigger.
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TimeTriggerConfig {
    #[serde(deserialize_with = "deserialize_limit")]
    limit: TimeTriggerLimit,
}

#[cfg(feature = "config_parsing")]
fn deserialize_limit<'de, D>(d: D) -> Result<TimeTriggerLimit, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct V;

    impl<'de2> de::Visitor<'de2> for V {
        type Value = TimeTriggerLimit;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("a time")
        }

        fn visit_u64<E>(self, v: u64) -> Result<TimeTriggerLimit, E>
        where
            E: de::Error,
        {
            Ok(TimeTriggerLimit::Second(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<TimeTriggerLimit, E>
        where
            E: de::Error,
        {
            if v < 0 {
                return Err(E::invalid_value(
                    de::Unexpected::Signed(v),
                    &"a non-negative number",
                ));
            }

            Ok(TimeTriggerLimit::Second(v as u64))
        }

        fn visit_str<E>(self, v: &str) -> Result<TimeTriggerLimit, E>
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
                None => return Ok(TimeTriggerLimit::Second(number)),
            };

            let result = if unit.eq_ignore_ascii_case("second")
                || unit.eq_ignore_ascii_case("seconds")
            {
                Some(TimeTriggerLimit::Second(number))
            } else if unit.eq_ignore_ascii_case("minute") || unit.eq_ignore_ascii_case("minutes") {
                Some(TimeTriggerLimit::Minute(number))
            } else if unit.eq_ignore_ascii_case("hour") || unit.eq_ignore_ascii_case("hours") {
                Some(TimeTriggerLimit::Hour(number))
            } else if unit.eq_ignore_ascii_case("day") || unit.eq_ignore_ascii_case("days") {
                Some(TimeTriggerLimit::Day(number))
            } else if unit.eq_ignore_ascii_case("week") || unit.eq_ignore_ascii_case("weeks") {
                Some(TimeTriggerLimit::Week(number))
            } else if unit.eq_ignore_ascii_case("month") || unit.eq_ignore_ascii_case("months") {
                Some(TimeTriggerLimit::Month(number))
            } else if unit.eq_ignore_ascii_case("year") || unit.eq_ignore_ascii_case("years") {
                Some(TimeTriggerLimit::Year(number))
            } else {
                return Err(E::invalid_value(de::Unexpected::Str(unit), &"a valid unit"));
            };

            match result {
                Some(n) => Ok(n),
                None => Err(E::invalid_value(de::Unexpected::Str(v), &"a time")),
            }
        }
    }

    d.deserialize_any(V)
}

/// A trigger which rolls the log once it has passed a certain time.
#[derive(Debug)]
pub struct TimeTrigger {
    limit: TimeTriggerLimit,
    time_start: RwLock<DateTime<Local>>,
}

/// The TimeTriger have the following units are supported (case insensitive):
/// "second", "seconds", "minute", "minutes", "hour", "hours", "day", "days", "week", "weeks", "month", "months", "year", "years". The unit defaults to
/// week if not specified.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TimeTriggerLimit {
    /// TimeTriger in second(s).
    Second(u64),
    /// TimeTriger in minute(s).
    Minute(u64),
    /// TimeTriger in hour(s).
    Hour(u64),
    /// TimeTriger in day(s).
    Day(u64),
    /// TimeTriger in week(s).
    Week(u64),
    /// TimeTriger in month(s).
    Month(u64),
    /// TimeTriger in year(s).
    Year(u64),
}

impl Default for TimeTriggerLimit {
    fn default() -> Self {
        TimeTriggerLimit::Week(1)
    }
}

impl TimeTrigger {
    /// Returns a new trigger which rolls the log once it has passed the
    /// specified time.
    pub fn new(limit: TimeTriggerLimit) -> TimeTrigger {
        let time = Local::now();
        let year = time.year();
        let month = time.month();
        let day = time.day();
        let weekday = time.weekday();
        let hour = time.hour();
        let min = time.minute();

        let time_new = match limit {
            TimeTriggerLimit::Second(_) => time,
            TimeTriggerLimit::Minute(_) => Local
                .with_ymd_and_hms(year, month, day, hour, min, 0)
                .unwrap(),
            TimeTriggerLimit::Hour(_) => Local
                .with_ymd_and_hms(year, month, day, hour, 0, 0)
                .unwrap(),
            TimeTriggerLimit::Day(_) => Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap(),
            TimeTriggerLimit::Week(_) => {
                Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
                    - Duration::days(weekday.num_days_from_monday() as i64)
            }
            TimeTriggerLimit::Month(_) => Local.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap(),
            TimeTriggerLimit::Year(_) => Local.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap(),
        };

        TimeTrigger {
            limit,
            time_start: RwLock::new(time_new),
        }
    }
}

impl Trigger for TimeTrigger {
    fn trigger(&self, _file: &LogFile) -> anyhow::Result<bool> {
        let time_now = Local::now();
        let mut time_start = self.time_start.write().unwrap();
        let duration = time_now.signed_duration_since(*time_start);
        let is_triger = match self.limit {
            TimeTriggerLimit::Second(num) => duration.num_seconds() as u64 >= num,
            TimeTriggerLimit::Minute(num) => duration.num_minutes() as u64 >= num,
            TimeTriggerLimit::Hour(num) => duration.num_hours() as u64 >= num,
            TimeTriggerLimit::Day(num) => duration.num_days() as u64 >= num,
            TimeTriggerLimit::Week(num) => duration.num_weeks() as u64 >= num,
            TimeTriggerLimit::Month(num) => {
                let num_years_start = time_start.year() as u64;
                let num_months_start = num_years_start * 12 + time_start.month() as u64;
                let num_years_now = time_now.year() as u64;
                let num_months_now = num_years_now * 12 + time_now.month() as u64;

                num_months_now - num_months_start >= num
            }
            TimeTriggerLimit::Year(num) => {
                let num_years_start = time_start.year() as u64;
                let num_years_now = time_now.year() as u64;

                num_years_now - num_years_start >= num
            }
        };
        if is_triger {
            let tmp = TimeTrigger::new(self.limit);
            let time_new = tmp.time_start.read().unwrap();
            *time_start = *time_new;
        }
        Ok(is_triger)
    }

    fn is_pre_process(&self) -> bool {
        true
    }
}

/// A deserializer for the `TimeTrigger`.
///
/// # Configuration
///
/// ```yaml
/// kind: time
///
/// # The time limit in second. The following units are supported (case insensitive):
/// # "second", "seconds", "minute", "minutes", "hour", "hours", "day", "days", "week", "weeks", "month", "months", "year", "years". The unit defaults to
/// # second if not specified. Required.
/// limit: 7 day
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub(crate) struct TimeTriggerDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for TimeTriggerDeserializer {
    type Trait = dyn Trigger;

    type Config = TimeTriggerConfig;

    fn deserialize(
        &self,
        config: TimeTriggerConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Trigger>> {
        Ok(Box::new(TimeTrigger::new(config.limit)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trigger() {
        let file = tempfile::tempdir().unwrap();
        let logfile = LogFile {
            writer: &mut None,
            path: file.path(),
            len: 0,
        };
        let trigger = TimeTrigger::new(TimeTriggerLimit::Second(10));
        let result = trigger.trigger(&logfile).unwrap();
        assert_eq!(false, result);
        std::thread::sleep(std::time::Duration::from_secs(12));
        let result = trigger.trigger(&logfile).unwrap();
        assert_eq!(true, result);
        let result = trigger.trigger(&logfile).unwrap();
        assert_eq!(false, result);
        std::thread::sleep(std::time::Duration::from_secs(12));
        let result = trigger.trigger(&logfile).unwrap();
        assert_eq!(true, result);
    }
}
