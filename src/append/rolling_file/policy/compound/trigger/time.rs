//! The time trigger.
//!
//! Requires the `time_trigger` feature.

#[cfg(test)]
use chrono::NaiveDateTime;
use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};
#[cfg(test)]
use mock_instant::{SystemTime, UNIX_EPOCH};
use rand::Rng;
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
            let random_delay = rand::thread_rng().gen_range(0..config.max_random_delay);
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
        let is_trigger = current >= *next_roll_time;
        if is_trigger {
            let tmp = TimeTrigger::new(self.config);
            let time_new = tmp.next_roll_time.read().unwrap();
            *next_roll_time = *time_new;
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
    use mock_instant::MockClock;
    use std::time::Duration;

    #[cfg(feature = "config_parsing")]
    use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

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
    fn test_trigger() {
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
        for (time_trigger_interval, time_in_milli) in test_list.iter() {
            MockClock::set_system_time(Duration::from_millis(4 * day_in_milli)); // 1970/1/5 00:00:00 Monday
            assert_eq!(
                trigger_with_time_and_modulate(*time_trigger_interval, modulate, *time_in_milli),
                (false, true)
            );
            // trigger will be aligned with units.
            MockClock::set_system_time(
                Duration::from_millis(4 * day_in_milli) + Duration::from_millis(time_in_milli / 2),
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
        for (time_trigger_interval, time_in_milli) in test_list.iter() {
            MockClock::set_system_time(Duration::from_millis(
                59 * day_in_milli + 2 * hour_in_milli + 2 * minute_in_milli + 2 * second_in_milli,
            )); // 1970/3/1 02:02:02 Sunday
            assert_eq!(
                trigger_with_time_and_modulate(*time_trigger_interval, modulate, *time_in_milli),
                (true, false)
            );
        }
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_defaults_deserialize() {
        let trigger = TimeTriggerConfig {
            interval: TimeTriggerInterval::Second(1),
            modulate: false,
            max_random_delay: 0,
        };

        assert_de_tokens(
            &trigger,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 3,
                },
                Token::Str("interval"),
                Token::I64(1),
                Token::Str("modulate"),
                Token::Bool(false),
                Token::Str("max_random_delay"),
                Token::U64(0),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_interval_deser_errors() {
        assert_de_tokens_error::<TimeTriggerConfig>(
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("abc"),
                Token::StructEnd,
            ],
            "invalid value: string \"\", expected a number",
        );

        assert_de_tokens_error::<TimeTriggerConfig>(
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str(""),
                Token::StructEnd,
            ],
            "invalid value: string \"\", expected a number",
        );

        assert_de_tokens_error::<TimeTriggerConfig>(
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("5 das"),
                Token::StructEnd,
            ],
            "invalid value: string \"das\", expected a valid unit",
        );

        // Can't test with a STR "-1" because the negative sign parses as a
        // non-ascii and the whole value goes into unit
        assert_de_tokens_error::<TimeTriggerConfig>(
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::I64(-1),
                Token::StructEnd,
            ],
            "invalid value: integer `-1`, expected a non-negative number",
        );

        assert_de_tokens_error::<TimeTriggerConfig>(
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::F32(2.0),
                Token::StructEnd,
            ],
            "invalid type: floating point `2.0`, expected a time",
        );
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_interval_deser() {
        let mut trigger_cfg = TimeTriggerConfig {
            interval: TimeTriggerInterval::Second(1),
            modulate: false,
            max_random_delay: 0,
        };

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::U64(1),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 second"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 seconds"),
                Token::StructEnd,
            ],
        );

        trigger_cfg.interval = TimeTriggerInterval::Minute(1);
        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 minute"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 minutes"),
                Token::StructEnd,
            ],
        );

        trigger_cfg.interval = TimeTriggerInterval::Hour(1);
        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 hour"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 hours"),
                Token::StructEnd,
            ],
        );

        trigger_cfg.interval = TimeTriggerInterval::Day(1);
        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 day"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 days"),
                Token::StructEnd,
            ],
        );

        trigger_cfg.interval = TimeTriggerInterval::Week(1);
        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 week"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 weeks"),
                Token::StructEnd,
            ],
        );

        trigger_cfg.interval = TimeTriggerInterval::Month(1);
        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 month"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 months"),
                Token::StructEnd,
            ],
        );

        trigger_cfg.interval = TimeTriggerInterval::Year(1);
        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 year"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &trigger_cfg,
            &[
                Token::Struct {
                    name: "TimeTriggerConfig",
                    len: 1,
                },
                Token::Str("interval"),
                Token::Str("1 years"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_time_trigger_limit_default() {
        let interval = TimeTriggerInterval::default();
        assert_eq!(interval, TimeTriggerInterval::Second(1));
    }

    #[test]
    fn test_is_pre_process() {
        let config = TimeTriggerConfig {
            interval: TimeTriggerInterval::Minute(2),
            modulate: true,
            max_random_delay: 0,
        };
        let trigger = TimeTrigger::new(config);
        assert!(trigger.is_pre_process());
    }

    #[test]
    fn test_max_rand_delay() {
        let trigger_cfg = TimeTriggerConfig {
            interval: TimeTriggerInterval::Second(1),
            modulate: false,
            max_random_delay: 0,
        };

        let current = {
            let now: std::time::Duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch");
            NaiveDateTime::from_timestamp_opt(now.as_secs() as i64 + 1, now.subsec_nanos())
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        };

        let trigger = TimeTrigger::new(trigger_cfg);
        let trigger_time = trigger.next_roll_time.read().unwrap();
        assert_eq!(*trigger_time, current);

        // Using a delay of 1 will test the block, but will always add a 0 allowing us
        // to bypass the unknown of rand
        let trigger_cfg = TimeTriggerConfig {
            interval: TimeTriggerInterval::Second(1),
            modulate: false,
            max_random_delay: 1,
        };

        let current = {
            let now: std::time::Duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch");
            NaiveDateTime::from_timestamp_opt(now.as_secs() as i64 + 1, now.subsec_nanos())
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        };

        let trigger = TimeTrigger::new(trigger_cfg);
        let trigger_time = trigger.next_roll_time.read().unwrap();
        assert_eq!(*trigger_time, current);
    }
}
