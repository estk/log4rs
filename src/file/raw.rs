use humantime;
use log::LogLevelFilter;
use serde::de::{self, Deserialize, Deserializer};
use serde_value::Value;
use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::time::Duration;

use filter::FilterConfig;

#[derive(PartialEq, Eq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(deserialize_with = "de_duration", default)]
    pub refresh_rate: Option<Duration>,
    pub root: Option<Root>,
    #[serde(default)]
    pub appenders: HashMap<String, Appender>,
    #[serde(default)]
    pub loggers: HashMap<String, Logger>,
}

#[derive(PartialEq, Eq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    #[serde(deserialize_with = "::priv_serde::de_filter")]
    pub level: LogLevelFilter,
    #[serde(default)]
    pub appenders: Vec<String>,
}

#[derive(PartialEq, Eq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Logger {
    #[serde(deserialize_with = "::priv_serde::de_filter")]
    pub level: LogLevelFilter,
    #[serde(default)]
    pub appenders: Vec<String>,
    pub additive: Option<bool>,
}

fn de_duration<D>(d: D) -> Result<Option<Duration>, D::Error>
    where D: de::Deserializer
{
    struct S(Duration);

    impl de::Deserialize for S {
        fn deserialize<D>(d: D) -> Result<S, D::Error>
            where D: de::Deserializer
        {
            struct V;

            impl de::Visitor for V {
                type Value = S;

                fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                    fmt.write_str("a duration")
                }

                fn visit_str<E>(self, v: &str) -> Result<S, E>
                    where E: de::Error
                {
                    humantime::parse_duration(v)
                        .map(S)
                        .map_err(|e| E::custom(e))
                }
            }

            d.deserialize(V)
        }
    }

    Option::<S>::deserialize(d).map(|r| r.map(|s| s.0))
}

#[derive(PartialEq, Eq, Debug)]
pub struct Appender {
    pub kind: String,
    pub filters: Vec<FilterConfig>,
    pub config: Value,
}

impl Deserialize for Appender {
    fn deserialize<D>(d: D) -> Result<Appender, D::Error>
        where D: Deserializer
    {
        let mut map = try!(BTreeMap::<Value, Value>::deserialize(d));

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => try!(kind.deserialize_into().map_err(|e| e.into_error())),
            None => return Err(de::Error::missing_field("kind")),
        };

        let filters = match map.remove(&Value::String("filters".to_owned())) {
            Some(filters) => try!(filters.deserialize_into().map_err(|e| e.into_error())),
            None => vec![],
        };

        Ok(Appender {
            kind: kind,
            filters: filters,
            config: Value::Map(map),
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::time::Duration;
    use log::LogLevelFilter;
    use serde_value::Value;

    use super::*;
    use filter::FilterConfig;

    #[allow(unused_imports)]
    use file::{Format, parse};

    #[allow(dead_code)]
    fn expected() -> Config {
        Config {
            refresh_rate: Some(Duration::from_secs(60)),
            appenders: {
                let mut m = HashMap::new();
                m.insert("console".to_owned(),
                         Appender {
                             kind: "console".to_owned(),
                             config: Value::Map(BTreeMap::new()),
                             filters: vec![FilterConfig {
                                               kind: "threshold".to_owned(),
                                               config: {
                                                   let mut m = BTreeMap::new();
                                                   m.insert(Value::String("level".to_owned()),
                                                            Value::String("debug".to_owned()));
                                                   Value::Map(m)
                                               },
                                           }],
                         });
                m.insert("baz".to_owned(),
                         Appender {
                             kind: "file".to_owned(),
                             config: {
                                 let mut m = BTreeMap::new();
                                 m.insert(Value::String("file".to_owned()),
                                          Value::String("log/baz.log".to_owned()));
                                 Value::Map(m)
                             },
                             filters: vec![],
                         });
                m
            },
            root: Some(Root {
                level: LogLevelFilter::Info,
                appenders: vec!["console".to_owned()],
            }),
            loggers: {
                let mut m = HashMap::new();
                m.insert("foo::bar::baz".to_owned(),
                         Logger {
                             level: LogLevelFilter::Warn,
                             appenders: vec!["baz".to_owned()],
                             additive: Some(false),
                         });
                m
            },
        }
    }

    #[test]
    #[cfg(feature = "yaml_format")]
    fn basic_yaml() {
        let cfg = r#"
refresh_rate: 60 seconds

appenders:
  console:
    kind: console
    filters:
      - kind: threshold
        level: debug
  baz:
    kind: file
    file: log/baz.log

root:
  appenders:
    - console
  level: info

loggers:
  foo::bar::baz:
    level: warn
    appenders:
      - baz
    additive: false
"#;

        let actual = parse(Format::Yaml, cfg).unwrap();
        let expected = expected();
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg(feature = "json_format")]
    fn basic_json() {
        let cfg = r#"
{
    "refresh_rate": "60 seconds",
    "appenders": {
        "console": {
            "kind": "console",
            "filters": [
                {
                    "kind": "threshold",
                    "level": "debug"
                }
            ]
        },
        "baz": {
            "kind": "file",
            "file": "log/baz.log"
        }
    },
    "root": {
        "appenders": ["console"],
        "level": "info"
    },
    "loggers": {
        "foo::bar::baz": {
            "level": "warn",
            "appenders": ["baz"],
            "additive": false
        }
    }
}"#;

        let actual = parse(Format::Json, cfg).unwrap();
        let expected = expected();
        assert_eq!(expected, actual);
    }
}
