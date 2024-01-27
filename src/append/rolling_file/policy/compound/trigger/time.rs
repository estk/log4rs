//! The time trigger.
//!
//! Requires the `time_trigger` feature.

#[cfg(test)]
use chrono::NaiveDateTime;
use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};
#[cfg(test)]
use mock_instant::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "config_parsing")]
use serde::de;
#[cfg(feature = "config_parsing")]
use std::fmt;
use std::sync::RwLock;

use crate::append::rolling_file::{policy::compound::trigger::Trigger, LogFile};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

#[cfg(feature = "config_parsing")]
/// Configuration for the time trigger.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeTriggerConfig {
    interval: TimeTriggerInterval,
    #[serde(default)]
    modulate: bool,
    #[serde(default)]
    max_random_delay: u64,
}

#[cfg(not(feature = "config_parsing"))]
/// Configuration for the time trigger.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct TimeTriggerConfig {
    interval: TimeTriggerInterval,
    modulate: bool,
    max_random_delay: u64,
}

/// A trigger which rolls the log once it has passed a certain time.
#[derive(Debug)]
pub struct TimeTrigger {
    config: TimeTriggerConfig,
    next_roll_time: RwLock<DateTime<Local>>,
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
        #[cfg(test)]
        let current = {
            let now: std::time::Duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch");
            NaiveDateTime::from_timestamp_opt(now.as_secs() as i64, now.subsec_nanos())
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        };

        #[cfg(not(test))]
        let current = Local::now();

        let next_time = TimeTrigger::get_next_time(current, config.interval, config.modulate);

        let next_roll_time = if config.max_random_delay > 0 {
            let random_delay = rand::random::<u64>() % config.max_random_delay;
            next_time + Duration::seconds(random_delay as i64)
        } else {
            next_time
        };

        TimeTrigger {
            config,
            next_roll_time: RwLock::new(next_roll_time),
        }
    }

    fn get_next_time(
        current: DateTime<Local>,
        interval: TimeTriggerInterval,
        modulate: bool,
    ) -> DateTime<Local> {
        let year = current.year();
        if let TimeTriggerInterval::Year(n) = interval {
            let n = n as i32;
            let increment = if modulate { n - year % n } else { n };
            let year_new = year + increment;
            return Local.with_ymd_and_hms(year_new, 1, 1, 0, 0, 0).unwrap();
        }

        if let TimeTriggerInterval::Month(n) = interval {
            let month0 = current.month0();
            let n = n as u32;
            let increment = if modulate { n - month0 % n } else { n };
            let num_months = (year as u32) * 12 + month0;
            let num_months_new = num_months + increment;
            let year_new = (num_months_new / 12) as i32;
            let month_new = (num_months_new) % 12 + 1;
            return Local
                .with_ymd_and_hms(year_new, month_new, 1, 0, 0, 0)
                .unwrap();
        }

        let month = current.month();
        let day = current.day();
        if let TimeTriggerInterval::Week(n) = interval {
            let week0 = current.iso_week().week0();
            let weekday = current.weekday().num_days_from_sunday() as i64;
            // let time = NaiveDate::from_isoywd_opt(year, week0, Weekday::Mon).unwrap().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap();
            let time = Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
            let increment = if modulate { n - (week0 as i64) % n } else { n };
            return time + Duration::weeks(increment) - Duration::days(weekday); // Set Sunday as the first day of the week
        }

        if let TimeTriggerInterval::Day(n) = interval {
            let ordinal0 = current.ordinal0();
            let time = Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
            let increment = if modulate {
                n - (ordinal0 as i64) % n
            } else {
                n
            };
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
}

impl Trigger for TimeTrigger {
    fn trigger(&self, _file: &LogFile) -> anyhow::Result<bool> {
        #[cfg(test)]
        let current = {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch");
            NaiveDateTime::from_timestamp_opt(now.as_secs() as i64, now.subsec_nanos())
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        };

        #[cfg(not(test))]
        let current: DateTime<Local> = Local::now();
        let mut next_roll_time = self.next_roll_time.write().unwrap();
        let is_triger = current >= *next_roll_time;
        if is_triger {
            let tmp = TimeTrigger::new(self.config);
            let time_new = tmp.next_roll_time.read().unwrap();
            *next_roll_time = *time_new;
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
    use mock_instant::MockClock;
    use std::time::Duration;

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

        MockClock::advance_system_time(Duration::from_millis(millis / 2));
        let result1 = trigger.trigger(&logfile).unwrap();

        MockClock::advance_system_time(Duration::from_millis(millis / 2));
        let result2 = trigger.trigger(&logfile).unwrap();

        (result1, result2)
    }

    #[test]
    fn trigger() {
        let second_in_milli = 1000;
        let minute_in_milli = second_in_milli * 60;
        let hour_in_milli = minute_in_milli * 60;
        let day_in_milli = hour_in_milli * 24;
        let week_in_milli = day_in_milli * 7;
        let month_in_milli = day_in_milli * 31;
        let year_in_milli = day_in_milli * 365;
        let modulate = false;
        //Second
        MockClock::set_system_time(Duration::from_millis(0));
        assert_eq!(
            trigger_with_time_and_modulate(
                TimeTriggerInterval::Second(1),
                modulate,
                second_in_milli
            ),
            (false, true)
        );
        // Minute
        MockClock::set_system_time(Duration::from_millis(0));
        assert_eq!(
            trigger_with_time_and_modulate(
                TimeTriggerInterval::Minute(1),
                modulate,
                minute_in_milli
            ),
            (false, true)
        );
        // Hour
        MockClock::set_system_time(Duration::from_millis(0));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Hour(1), modulate, hour_in_milli),
            (false, true)
        );
        // Day
        MockClock::set_system_time(Duration::from_millis(0));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Day(1), modulate, day_in_milli),
            (false, true)
        );
        // Week
        MockClock::set_system_time(Duration::from_millis(3 * day_in_milli)); // Sunday
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Week(1), modulate, week_in_milli),
            (false, true)
        );
        // Month
        MockClock::set_system_time(Duration::from_millis(0));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Month(1), modulate, month_in_milli),
            (false, true)
        );
        // Year
        MockClock::set_system_time(Duration::from_millis(0));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Year(1), modulate, year_in_milli),
            (false, true)
        );
    }

    #[test]
    fn trigger2() {
        let second_in_milli = 1000;
        let minute_in_milli = second_in_milli * 60;
        let hour_in_milli = minute_in_milli * 60;
        let day_in_milli = hour_in_milli * 24;
        let week_in_milli = day_in_milli * 7;
        let month_in_milli = day_in_milli * 31;
        let year_in_milli = day_in_milli * 365;
        let modulate = false;
        // Second
        MockClock::set_system_time(Duration::from_millis(second_in_milli / 2));
        assert_eq!(
            trigger_with_time_and_modulate(
                TimeTriggerInterval::Second(1),
                modulate,
                second_in_milli
            ),
            (true, false)
        );
        // Minute
        MockClock::set_system_time(Duration::from_millis(minute_in_milli / 2));
        assert_eq!(
            trigger_with_time_and_modulate(
                TimeTriggerInterval::Minute(1),
                modulate,
                minute_in_milli
            ),
            (true, false)
        );
        // Hour
        MockClock::set_system_time(Duration::from_millis(hour_in_milli / 2));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Hour(1), modulate, hour_in_milli),
            (true, false)
        );
        // Day
        MockClock::set_system_time(Duration::from_millis(day_in_milli / 2));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Day(1), modulate, day_in_milli),
            (true, false)
        );
        // Week
        MockClock::set_system_time(Duration::from_millis(3 * day_in_milli + week_in_milli / 2)); // Sunday
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Week(1), modulate, week_in_milli),
            (true, false)
        );
        // Month
        MockClock::set_system_time(Duration::from_millis(month_in_milli / 2));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Month(1), modulate, month_in_milli),
            (true, false)
        );
        // Year
        MockClock::set_system_time(Duration::from_millis(year_in_milli / 2));
        assert_eq!(
            trigger_with_time_and_modulate(TimeTriggerInterval::Year(1), modulate, year_in_milli),
            (true, false)
        );
    }

    #[test]
    #[cfg(feature = "yaml_format")]
    fn test_serde() {
        // str none
        let interval = format!("abc",);
        let error = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval);
        assert!(error.is_err());

        // none
        let interval = format!("",);
        let error = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval);
        assert!(error.is_err());

        // bad unit
        let interval = format!("5 das",);
        let error = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval);
        assert!(error.is_err());

        // i64
        let interval = format!("-1",);
        let error = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval);
        assert!(error.is_err());

        // u64
        let interval = format!("1",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Second(1));

        // str second
        let interval = format!("1 second",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Second(1));

        let interval = format!("1 seconds",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Second(1));

        // str minute
        let interval = format!("1 minute",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Minute(1));

        let interval = format!("1 minutes",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Minute(1));

        // str hour
        let interval = format!("1 hour",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Hour(1));

        let interval = format!("1 hours",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Hour(1));

        // str day
        let interval = format!("1 day",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Day(1));

        let interval = format!("1 days",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Day(1));

        // str week
        let interval = format!("1 week",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Week(1));

        let interval = format!("1 weeks",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Week(1));

        // str month
        let interval = format!("1 month",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Month(1));

        let interval = format!("1 months",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Month(1));

        // str year
        let interval = format!("1 year",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Year(1));

        let interval = format!("1 years",);
        let interval = ::serde_yaml::from_str::<TimeTriggerInterval>(&interval).unwrap();
        assert_eq!(interval, TimeTriggerInterval::Year(1));
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
