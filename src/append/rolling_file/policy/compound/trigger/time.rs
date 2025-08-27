//! The time trigger.
//!
//! Requires the `time_trigger` feature.

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};

use rand::Rng;
#[cfg(feature = "config_parsing")]
use serde::de;
#[cfg(feature = "config_parsing")]
use std::fmt;
use std::sync::{Once, RwLock};

use crate::append::rolling_file::{policy::compound::trigger::Trigger, LogFile};
#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

#[cfg(feature = "config_parsing")]
/// Configuration for the time trigger.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeTriggerConfig {
    /// The date/time interval between log file rolls.
    pub interval: TimeTriggerInterval,
    /// Whether to modulate the interval.
    #[serde(default)]
    pub modulate: bool,
    /// The maximum random delay in seconds.
    #[serde(default)]
    pub max_random_delay: u64,
}

#[cfg(not(feature = "config_parsing"))]
/// Configuration for the time trigger.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct TimeTriggerConfig {
    /// The date/time interval between log file rolls.Q
    pub interval: TimeTriggerInterval,
    /// Whether to modulate the interval.
    pub modulate: bool,
    /// The maximum random delay in seconds.
    pub max_random_delay: u64,
}

/// A trigger which rolls the log once it has passed a certain time.
#[derive(Debug)]
pub struct TimeTrigger {
    config: TimeTriggerConfig,
    next_roll_time: RwLock<DateTime<Local>>,
    initial: Once,
}

/// The TimeTrigger supports the following units (case insensitive):
/// "second", "seconds", "minute", "minutes", "hour", "hours", "day", "days", "week", "weeks", "month", "months", "year", "years". The unit defaults to
/// second if not specified.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TimeTriggerInterval {
    /// TimeTriger in second(s).
    Second(i64),
    /// TimeTriger in minute(s).
    Minute(i64),
    /// TimeTriger in hour(s).
    Hour(i64),
    /// TimeTriger in day(s).
    Day(i64),
    /// TimeTriger in week(s).
    Week(i64),
    /// TimeTriger in month(s).
    Month(i64),
    /// TimeTriger in year(s).
    Year(i64),
}

impl Default for TimeTriggerInterval {
    fn default() -> Self {
        TimeTriggerInterval::Second(1)
    }
}

#[cfg(mock_time)]
fn get_current_time() -> DateTime<Local> {
    use mock_instant::thread_local::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch");
    DateTime::from_timestamp(now.as_secs() as i64, now.subsec_nanos())
        .unwrap()
        .naive_local()
        .and_local_timezone(Local)
        .unwrap()
}

#[cfg(not(mock_time))]
fn get_current_time() -> DateTime<Local> {
    Local::now()
}

#[cfg(feature = "config_parsing")]
impl<'de> serde::Deserialize<'de> for TimeTriggerInterval {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct V;

        impl<'de2> de::Visitor<'de2> for V {
            type Value = TimeTriggerInterval;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a time")
            }

            fn visit_u64<E>(self, v: u64) -> Result<TimeTriggerInterval, E>
            where
                E: de::Error,
            {
                Ok(TimeTriggerInterval::Second(v as i64))
            }

            fn visit_i64<E>(self, v: i64) -> Result<TimeTriggerInterval, E>
            where
                E: de::Error,
            {
                if v < 0 {
                    return Err(E::invalid_value(
                        de::Unexpected::Signed(v),
                        &"a non-negative number",
                    ));
                }

                Ok(TimeTriggerInterval::Second(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<TimeTriggerInterval, E>
            where
                E: de::Error,
            {
                let (number, unit) = match v.find(|c: char| !c.is_ascii_digit()) {
                    Some(n) => (v[..n].trim(), Some(v[n..].trim())),
                    None => (v.trim(), None),
                };

                let number = match number.parse::<i64>() {
                    Ok(n) => {
                        if n < 0 {
                            return Err(E::invalid_value(
                                de::Unexpected::Signed(n),
                                &"a non-negative number",
                            ));
                        }
                        n
                    }
                    Err(_) => {
                        return Err(E::invalid_value(de::Unexpected::Str(number), &"a number"))
                    }
                };

                let unit = match unit {
                    Some(u) => u,
                    None => return Ok(TimeTriggerInterval::Second(number)),
                };

                let result = if unit.eq_ignore_ascii_case("second")
                    || unit.eq_ignore_ascii_case("seconds")
                {
                    Some(TimeTriggerInterval::Second(number))
                } else if unit.eq_ignore_ascii_case("minute")
                    || unit.eq_ignore_ascii_case("minutes")
                {
                    Some(TimeTriggerInterval::Minute(number))
                } else if unit.eq_ignore_ascii_case("hour") || unit.eq_ignore_ascii_case("hours") {
                    Some(TimeTriggerInterval::Hour(number))
                } else if unit.eq_ignore_ascii_case("day") || unit.eq_ignore_ascii_case("days") {
                    Some(TimeTriggerInterval::Day(number))
                } else if unit.eq_ignore_ascii_case("week") || unit.eq_ignore_ascii_case("weeks") {
                    Some(TimeTriggerInterval::Week(number))
                } else if unit.eq_ignore_ascii_case("month") || unit.eq_ignore_ascii_case("months")
                {
                    Some(TimeTriggerInterval::Month(number))
                } else if unit.eq_ignore_ascii_case("year") || unit.eq_ignore_ascii_case("years") {
                    Some(TimeTriggerInterval::Year(number))
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
}

impl TimeTrigger {
    /// Returns a new trigger which rolls the log once it has passed the
    /// specified time.
    pub fn new(config: TimeTriggerConfig) -> TimeTrigger {
        TimeTrigger {
            config,
            next_roll_time: RwLock::default(),
            initial: Once::new(),
        }
    }

    fn get_next_time(&self, current: DateTime<Local>) -> DateTime<Local> {
        let interval = self.config.interval;
        let modulate = self.config.modulate;

        let year = current.year();
        if let TimeTriggerInterval::Year(n) = interval {
            let n = n as i32;
            let increment = if modulate { n - year % n } else { n };
            let year_new = year + increment;
            let result = Local.with_ymd_and_hms(year_new, 1, 1, 0, 0, 0).unwrap();
            return result;
        }

        if let TimeTriggerInterval::Month(n) = interval {
            let month0 = current.month0();
            let n = n as u32;
            let increment = if modulate { n - month0 % n } else { n };
            let num_months = (year as u32) * 12 + month0;
            let num_months_new = num_months + increment;
            let year_new = (num_months_new / 12) as i32;
            let month_new = (num_months_new) % 12 + 1;
            let result = Local
                .with_ymd_and_hms(year_new, month_new, 1, 0, 0, 0)
                .unwrap();
            return result;
        }

        let month = current.month();
        let day = current.day();
        if let TimeTriggerInterval::Week(n) = interval {
            let week0 = current.iso_week().week0() as i64;
            let weekday = current.weekday().num_days_from_monday() as i64; // Monday is the first day of the week
            let time = Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
            let increment = if modulate { n - week0 % n } else { n };
            return time + Duration::weeks(increment) - Duration::days(weekday);
        }

        if let TimeTriggerInterval::Day(n) = interval {
            let ordinal0 = current.ordinal0() as i64;
            let time = Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
            let increment = if modulate { n - ordinal0 % n } else { n };
            return time + Duration::days(increment);
        }

        let hour = current.hour();
        if let TimeTriggerInterval::Hour(n) = interval {
            let time = Local
                .with_ymd_and_hms(year, month, day, hour, 0, 0)
                .unwrap();
            let increment = if modulate { n - (hour as i64) % n } else { n };
            return time + Duration::hours(increment);
        }

        let min = current.minute();
        if let TimeTriggerInterval::Minute(n) = interval {
            let time = Local
                .with_ymd_and_hms(year, month, day, hour, min, 0)
                .unwrap();
            let increment = if modulate { n - (min as i64) % n } else { n };
            return time + Duration::minutes(increment);
        }

        let sec = current.second();
        if let TimeTriggerInterval::Second(n) = interval {
            let time = Local
                .with_ymd_and_hms(year, month, day, hour, min, sec)
                .unwrap();
            let increment = if modulate { n - (sec as i64) % n } else { n };
            return time + Duration::seconds(increment);
        }
        panic!("Should not reach here!");
    }

    fn refresh_time(&self) {
        let current = get_current_time();
        let next_time = self.get_next_time(current);
        let next_roll_time = if self.config.max_random_delay > 0 {
            let random_delay = rand::rng().random_range(0..self.config.max_random_delay);
            next_time + Duration::seconds(random_delay as i64)
        } else {
            next_time
        };
        *self.next_roll_time.write().unwrap() = next_roll_time;
    }
}

impl Trigger for TimeTrigger {
    fn trigger(&self, _file: &LogFile) -> anyhow::Result<bool> {
        self.initial.call_once(|| {
            self.refresh_time();
        });

        let current = get_current_time();
        let next_roll_time = self.next_roll_time.read().unwrap();
        let is_trigger = current >= *next_roll_time;
        drop(next_roll_time);
        if is_trigger {
            self.refresh_time();
        }
        Ok(is_trigger)
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
/// # The time interval. The following units are supported (case insensitive):
/// # "second(s)", "minute(s)", "hour(s)", "day(s)", "week(s)", "month(s)", "year(s)". The unit defaults to
/// # second if not specified.
/// interval: 7 day
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
        Ok(Box::new(TimeTrigger::new(config)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(mock_time)]
    use mock_instant::thread_local::MockClock;

    #[cfg(mock_time)]
    fn trigger_with_time_and_modulate(
        interval: TimeTriggerInterval,
        modulate: bool,
        millis: u64,
    ) -> (bool, bool) {
        let file = tempfile::tempdir().unwrap();
        let logfile = LogFile {
            writer: &mut None,
            path: file.path(),
            len: 0,
        };
        let config = TimeTriggerConfig {
            interval,
            modulate,
            max_random_delay: 0,
        };

        let trigger = TimeTrigger::new(config);
        trigger.trigger(&logfile).unwrap();

        MockClock::advance_system_time(std::time::Duration::from_millis(millis / 2));
        let result1 = trigger.trigger(&logfile).unwrap();

        MockClock::advance_system_time(std::time::Duration::from_millis(millis / 2));
        let result2 = trigger.trigger(&logfile).unwrap();

        (result1, result2)
    }

    #[test]
    #[cfg(mock_time)]
    fn trigger() {
        let second_in_milli = 1000;
        let minute_in_milli = second_in_milli * 60;
        let hour_in_milli = minute_in_milli * 60;
        let day_in_milli = hour_in_milli * 24;
        let week_in_milli = day_in_milli * 7;
        let month_in_milli = day_in_milli * 31;
        let year_in_milli = day_in_milli * 365;

        let test_list = vec![
            (TimeTriggerInterval::Second(1), second_in_milli),
            (TimeTriggerInterval::Minute(1), minute_in_milli),
            (TimeTriggerInterval::Hour(1), hour_in_milli),
            (TimeTriggerInterval::Day(1), day_in_milli),
            (TimeTriggerInterval::Week(1), week_in_milli),
            (TimeTriggerInterval::Month(1), month_in_milli),
            (TimeTriggerInterval::Year(1), year_in_milli),
        ];
        let modulate = false;
        for (time_trigger_interval, time_in_milli) in &test_list {
            dbg!(time_in_milli);
            MockClock::set_system_time(std::time::Duration::from_millis(4 * day_in_milli)); // 1970/1/5 00:00:00 Monday
            assert_eq!(
                trigger_with_time_and_modulate(*time_trigger_interval, modulate, *time_in_milli),
                (false, true)
            );
            // trigger will be aligned with units.
            MockClock::set_system_time(
                std::time::Duration::from_millis(4 * day_in_milli)
                    + std::time::Duration::from_millis(time_in_milli / 2),
            );
            assert_eq!(
                trigger_with_time_and_modulate(*time_trigger_interval, modulate, *time_in_milli),
                (true, false)
            );
        }

        let test_list = vec![
            (TimeTriggerInterval::Second(3), 3 * second_in_milli),
            (TimeTriggerInterval::Minute(3), 3 * minute_in_milli),
            (TimeTriggerInterval::Hour(3), 3 * hour_in_milli),
            (TimeTriggerInterval::Day(3), 3 * day_in_milli),
            (TimeTriggerInterval::Week(3), 3 * week_in_milli),
            (TimeTriggerInterval::Month(3), 3 * month_in_milli),
            (TimeTriggerInterval::Year(3), 3 * year_in_milli),
        ];
        let modulate = true;
        for (time_trigger_interval, time_in_milli) in &test_list {
            dbg!(time_in_milli);
            MockClock::set_system_time(std::time::Duration::from_millis(
                59 * day_in_milli + 2 * hour_in_milli + 2 * minute_in_milli + 2 * second_in_milli,
            )); // 1970/3/1 02:02:02 Sunday
            assert_eq!(
                trigger_with_time_and_modulate(*time_trigger_interval, modulate, *time_in_milli),
                (true, false)
            );
        }
    }

    #[test]
    #[cfg(feature = "yaml_format")]
    fn test_serde() {
        let test_error = vec![
            "abc",   // // str none none
            "",      // none
            "5 das", // bad unit
            "-1",    // inegative integar
            "2.0",   //flaot
        ];

        for interval in test_error.iter() {
            let error = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval);
            assert!(error.is_err());
        }

        let test_ok = vec![
            // u64
            ("1", TimeTriggerInterval::Second(1)),
            // str second
            ("1 second", TimeTriggerInterval::Second(1)),
            ("1 seconds", TimeTriggerInterval::Second(1)),
            // str minute
            ("1 minute", TimeTriggerInterval::Minute(1)),
            ("1 minutes", TimeTriggerInterval::Minute(1)),
            // str hour
            ("1 hour", TimeTriggerInterval::Hour(1)),
            ("1 hours", TimeTriggerInterval::Hour(1)),
            // str day
            ("1 day", TimeTriggerInterval::Day(1)),
            ("1 days", TimeTriggerInterval::Day(1)),
            // str week
            ("1 week", TimeTriggerInterval::Week(1)),
            ("1 weeks", TimeTriggerInterval::Week(1)),
            // str month
            ("1 month", TimeTriggerInterval::Month(1)),
            ("1 months", TimeTriggerInterval::Month(1)),
            // str year
            ("1 year", TimeTriggerInterval::Year(1)),
            ("1 years", TimeTriggerInterval::Year(1)),
        ];
        for (interval, expected) in test_ok.iter() {
            let interval = format!("{}", interval);
            let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
            assert_eq!(interval, *expected);
        }
    }

    #[test]
    fn test_time_trigger_limit_default() {
        let interval = TimeTriggerInterval::default();
        assert_eq!(interval, TimeTriggerInterval::Second(1));
    }

    #[test]
    fn pre_process() {
        let config = TimeTriggerConfig {
            interval: TimeTriggerInterval::Minute(2),
            modulate: true,
            max_random_delay: 0,
        };
        let trigger = TimeTrigger::new(config);
        assert!(trigger.is_pre_process());
    }
}
