//! Appenders

use log::{Log, Record};
#[cfg(feature = "config_parsing")]
use serde::{de, Deserialize, Deserializer};
use serde::{Serialize, Serializer};
#[cfg(feature = "config_parsing")]
use serde_value::Value;
#[cfg(feature = "config_parsing")]
use std::collections::BTreeMap;
use std::fmt;

#[cfg(feature = "config_parsing")]
use crate::config::Deserializable;
#[cfg(feature = "config_parsing")]
use crate::filter::FilterConfig;

#[cfg(feature = "console_appender")]
pub mod console;
#[cfg(feature = "file_appender")]
pub mod file;
#[cfg(feature = "rolling_file_appender")]
pub mod rolling_file;

#[cfg(any(feature = "file_appender", feature = "rolling_file_appender"))]
mod env_util {
    use std::borrow::Cow;

    const ENV_PREFIX: &str = "$ENV{";
    const ENV_PREFIX_LEN: usize = ENV_PREFIX.len();
    const ENV_SUFFIX: char = '}';
    const ENV_SUFFIX_LEN: usize = 1;

    fn is_env_var_start(c: char) -> bool {
        // Close replacement for old [\w]
        // Note that \w implied \d and '_' and non-ASCII letters/digits.
        c.is_alphanumeric() || c == '_'
    }

    fn is_env_var_part(c: char) -> bool {
        // Close replacement for old [\w\d_.]
        c.is_alphanumeric() || c == '_' || c == '.'
    }

    pub fn expand_env_vars<'str, Str>(path: Str) -> Cow<'str, str>
    where
        Str: Into<Cow<'str, str>>,
    {
        let mut outpath: Cow<str> = path.into();
        let path = outpath.clone();
        for (match_start, _) in path.match_indices(ENV_PREFIX) {
            let env_name_start = match_start + ENV_PREFIX_LEN;
            let (_, tail) = path.split_at(env_name_start);
            let mut cs = tail.chars();
            // Check first character.
            if let Some(ch) = cs.next() {
                if is_env_var_start(ch) {
                    let mut env_name = String::new();
                    env_name.push(ch);
                    // Consume following characters.
                    let valid = loop {
                        match cs.next() {
                            Some(ch) if is_env_var_part(ch) => env_name.push(ch),
                            Some(ENV_SUFFIX) => break true,
                            _ => break false,
                        }
                    };
                    // Try replacing properly terminated env var.
                    if valid {
                        if let Ok(env_value) = std::env::var(&env_name) {
                            let match_end = env_name_start + env_name.len() + ENV_SUFFIX_LEN;
                            // This simply rewrites the entire outpath with all instances
                            // of this var replaced. Could be done more efficiently by building
                            // `outpath` as we go when processing `path`. Not critical.
                            outpath = outpath
                                .replace(&path[match_start..match_end], &env_value)
                                .into();
                        }
                    }
                }
            }
        }
        outpath
    }
}

/// A trait implemented by log4rs appenders.
///
/// Appenders take a log record and processes them, for example, by writing it
/// to a file or the console.
pub trait Append: fmt::Debug + Send + Sync + 'static {
    /// Processes the provided `Record`.
    fn append(&self, record: &Record) -> anyhow::Result<()>;

    /// Flushes all in-flight records.
    fn flush(&self);
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Append {
    fn name() -> &'static str {
        "appender"
    }
}

impl<T: Log + fmt::Debug + 'static> Append for T {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        self.log(record);
        Ok(())
    }

    fn flush(&self) {
        Log::flush(self)
    }
}

/// Configuration for an appender.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct AppenderConfig {
    /// The appender kind.
    pub kind: String,
    /// The filters attached to the appender.
    pub filters: Vec<FilterConfig>,
    /// The appender configuration.
    pub config: Value,
}

#[cfg(feature = "config_parsing")]
impl<'de> Deserialize<'de> for AppenderConfig {
    fn deserialize<D>(d: D) -> Result<AppenderConfig, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => kind.deserialize_into().map_err(|e| e.into_error())?,
            None => return Err(de::Error::missing_field("kind")),
        };

        let filters = match map.remove(&Value::String("filters".to_owned())) {
            Some(filters) => filters.deserialize_into().map_err(|e| e.into_error())?,
            None => vec![],
        };

        Ok(AppenderConfig {
            kind,
            filters,
            config: Value::Map(map),
        })
    }
}

#[cfg(feature = "config_parsing")]
impl Serialize for AppenderConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map: BTreeMap<Value, Value> = BTreeMap::new();

        map.insert(
            Value::String("kind".to_string()),
            Value::String(self.kind.clone()),
        );

        // Insert the `filters` field into the map if it's not empty
        if !self.filters.is_empty() {
            let filters: Vec<Value> = self
                .filters
                .iter()
                .map(|f| serde_value::to_value(f).unwrap())
                .collect();

            map.insert(Value::String("filters".to_string()), Value::Seq(filters));
        } else {
            map.insert(
                Value::String("filters".to_string()),
                Value::String("[]".to_string()),
            );
        }

        if let Value::Map(ref config_map) = self.config {
            for (key, value) in config_map {
                map.insert(key.clone(), value.clone());
            }
        }

        // Serialize the map
        map.serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    #[cfg(any(feature = "file_appender", feature = "rolling_file_appender"))]
    use std::env::{set_var, var};

    #[test]
    #[cfg(any(feature = "file_appender", feature = "rolling_file_appender"))]
    fn expand_env_vars_tests() {
        set_var("HELLO_WORLD", "GOOD BYE");
        #[cfg(not(target_os = "windows"))]
        let test_cases = vec![
            ("$ENV{HOME}", var("HOME").unwrap()),
            ("$ENV{HELLO_WORLD}", var("HELLO_WORLD").unwrap()),
            ("$ENV{HOME}/test", format!("{}/test", var("HOME").unwrap())),
            (
                "/test/$ENV{HOME}",
                format!("/test/{}", var("HOME").unwrap()),
            ),
            (
                "/test/$ENV{HOME}/test",
                format!("/test/{}/test", var("HOME").unwrap()),
            ),
            (
                "/test$ENV{HOME}/test",
                format!("/test{}/test", var("HOME").unwrap()),
            ),
            (
                "test/$ENV{HOME}/test",
                format!("test/{}/test", var("HOME").unwrap()),
            ),
            (
                "/$ENV{HOME}/test/$ENV{USER}",
                format!("/{}/test/{}", var("HOME").unwrap(), var("USER").unwrap()),
            ),
            (
                "$ENV{SHOULD_NOT_EXIST}",
                "$ENV{SHOULD_NOT_EXIST}".to_string(),
            ),
            (
                "/$ENV{HOME}/test/$ENV{SHOULD_NOT_EXIST}",
                format!("/{}/test/$ENV{{SHOULD_NOT_EXIST}}", var("HOME").unwrap()),
            ),
            (
                "/unterminated/$ENV{USER",
                "/unterminated/$ENV{USER".to_string(),
            ),
        ];

        #[cfg(target_os = "windows")]
        let test_cases = vec![
            ("$ENV{HOMEPATH}", var("HOMEPATH").unwrap()),
            ("$ENV{HELLO_WORLD}", var("HELLO_WORLD").unwrap()),
            (
                "$ENV{HOMEPATH}/test",
                format!("{}/test", var("HOMEPATH").unwrap()),
            ),
            (
                "/test/$ENV{USERNAME}",
                format!("/test/{}", var("USERNAME").unwrap()),
            ),
            (
                "/test/$ENV{USERNAME}/test",
                format!("/test/{}/test", var("USERNAME").unwrap()),
            ),
            (
                "/test$ENV{USERNAME}/test",
                format!("/test{}/test", var("USERNAME").unwrap()),
            ),
            (
                "test/$ENV{USERNAME}/test",
                format!("test/{}/test", var("USERNAME").unwrap()),
            ),
            (
                "$ENV{HOMEPATH}/test/$ENV{USERNAME}",
                format!(
                    "{}/test/{}",
                    var("HOMEPATH").unwrap(),
                    var("USERNAME").unwrap()
                ),
            ),
            (
                "$ENV{SHOULD_NOT_EXIST}",
                "$ENV{SHOULD_NOT_EXIST}".to_string(),
            ),
            (
                "$ENV{HOMEPATH}/test/$ENV{SHOULD_NOT_EXIST}",
                format!("{}/test/$ENV{{SHOULD_NOT_EXIST}}", var("HOMEPATH").unwrap()),
            ),
            (
                "/unterminated/$ENV{USERNAME",
                "/unterminated/$ENV{USERNAME".to_string(),
            ),
        ];

        for (input, expected) in test_cases {
            let res = super::env_util::expand_env_vars(input);
            assert_eq!(res, expected)
        }
    }
}
