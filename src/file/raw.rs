//! Types used to deserialize config files.
#![allow(missing_docs)]

use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use serde::de::{self, Deserialize, Deserializer};
use serde_value::Value;
use log::LogLevelFilter;

use priv_serde::DeDuration;

include!("serde.rs");

#[derive(PartialEq, Eq, Debug)]
pub struct Config {
    pub refresh_rate: Option<Duration>,
    pub root: Option<Root>,
    pub appenders: HashMap<String, Appender>,
    pub loggers: HashMap<String, Logger>,
    _p: (),
}

impl de::Deserialize for Config {
    fn deserialize<D>(d: &mut D) -> Result<Config, D::Error>
        where D: de::Deserializer
    {
        let PrivConfig { refresh_rate, root, appenders, loggers } =
            try!(PrivConfig::deserialize(d));

        Ok(Config {
            refresh_rate: refresh_rate.map(|r| r.0),
            root: root,
            appenders: appenders,
            loggers: loggers,
            _p: (),
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Root {
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
    _p: (),
}

impl de::Deserialize for Root {
    fn deserialize<D>(d: &mut D) -> Result<Root, D::Error>
        where D: de::Deserializer
    {
        let PrivRoot { level, appenders } = try!(PrivRoot::deserialize(d));

        Ok(Root {
            level: level,
            appenders: appenders,
            _p: (),
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Logger {
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
    pub additive: Option<bool>,
    _p: (),
}

impl de::Deserialize for Logger {
    fn deserialize<D>(d: &mut D) -> Result<Logger, D::Error>
        where D: de::Deserializer
    {
        let PrivLogger { level, appenders, additive } = try!(PrivLogger::deserialize(d));

        Ok(Logger {
            level: level,
            appenders: appenders,
            additive: additive,
            _p: (),
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Appender {
    pub kind: String,
    pub filters: Vec<Filter>,
    pub config: Value,
}

impl Deserialize for Appender {
    fn deserialize<D>(d: &mut D) -> Result<Appender, D::Error>
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

#[derive(PartialEq, Eq, Debug)]
pub struct Filter {
    pub kind: String,
    pub config: Value,
}

impl Deserialize for Filter {
    fn deserialize<D>(d: &mut D) -> Result<Filter, D::Error>
        where D: Deserializer
    {
        let mut map = try!(BTreeMap::<Value, Value>::deserialize(d));

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => try!(kind.deserialize_into().map_err(|e| e.to_error())),
            None => return Err(de::Error::missing_field("kind")),
        };

        Ok(Filter {
            kind: kind,
            config: Value::Map(map),
        })
    }
}

pub struct Encoder {
    pub kind: String,
    pub config: Value,
}

impl Deserialize for Encoder {
    fn deserialize<D>(d: &mut D) -> Result<Encoder, D::Error>
        where D: Deserializer
    {
        let mut map = try!(BTreeMap::<Value, Value>::deserialize(d));

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => try!(kind.deserialize_into().map_err(|e| e.to_error())),
            None => "pattern".to_owned(),
        };

        Ok(Encoder {
            kind: kind,
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
                             filters: vec![Filter {
                                               kind: "threshold".to_string(),
                                               config: {
                                                   let mut m = BTreeMap::new();
                                                   m.insert(Value::String("level".to_string()),
                                                            Value::String("debug".to_string()));
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
                _p: (),
            }),
            loggers: {
                let mut m = HashMap::new();
                m.insert("foo::bar::baz".to_owned(),
                         Logger {
                             level: LogLevelFilter::Warn,
                             appenders: vec!["baz".to_owned()],
                             additive: Some(false),
                             _p: (),
                         });
                m
            },
            _p: (),
        }
    }

    #[test]
    #[cfg(feature = "yaml")]
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
    #[cfg(feature = "json")]
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

    #[test]
    #[cfg(feature = "toml")]
    fn basic_toml() {
        let cfg = r#"
refresh_rate = "60 seconds"

[appenders.console]
kind = "console"
[[appenders.console.filters]]
kind = "threshold"
level = "debug"

[appenders.baz]
kind = "file"
file = "log/baz.log"

[root]
appenders = ["console"]
level = "info"

[loggers."foo::bar::baz"]
level = "warn"
appenders = ["baz"]
additive = false
"#;

        let actual = parse(Format::Toml, cfg).unwrap();
        let expected = expected();
        assert_eq!(expected, actual);
    }
}
