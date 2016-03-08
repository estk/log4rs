use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use time::Duration;
use serde::de::{self, Deserialize, Deserializer};
use serde_value::Value;
use serde_yaml;

use log::LogLevelFilter;

include!("serde.rs");

#[derive(PartialEq, Debug)]
pub struct DeDuration(pub Duration);

impl Deserialize for DeDuration {
    fn deserialize<D>(d: &mut D) -> Result<DeDuration, D::Error> where D: Deserializer {
        i64::deserialize(d).map(|r| DeDuration(Duration::seconds(r)))
    }
}

#[derive(PartialEq, Debug)]
pub struct DeLogLevelFilter(pub LogLevelFilter);

impl Deserialize for DeLogLevelFilter {
    fn deserialize<D>(d: &mut D) -> Result<DeLogLevelFilter, D::Error> where D: Deserializer {
        struct V;

        impl de::Visitor for V {
            type Value = DeLogLevelFilter;

            fn visit_str<E>(&mut self, v: &str) -> Result<DeLogLevelFilter, E> where E: de::Error {
                v.parse().map(DeLogLevelFilter).map_err(|_| E::invalid_value(v))
            }
        }

        d.deserialize_str(V)
    }
}

#[derive(PartialEq, Debug)]
pub struct Appender {
    pub kind: String,
    pub filters: Vec<Filter>,
    pub config: Value,
}

impl Deserialize for Appender {
    fn deserialize<D>(d: &mut D) -> Result<Appender, D::Error> where D: Deserializer {
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

#[derive(PartialEq, Debug)]
pub struct Filter {
    pub kind: String,
    pub config: Value,
}

impl Deserialize for Filter {
    fn deserialize<D>(d: &mut D) -> Result<Filter, D::Error> where D: Deserializer {
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

pub fn parse(config: &str) -> Result<Config, Box<Error>> {
    serde_yaml::from_str(config).map_err(|e| e.into())
}

#[cfg(test)]
mod test {
    use std::borrow::ToOwned;
    use std::collections::{HashMap, BTreeMap};
    use time::Duration;
    use log::LogLevelFilter;
    use serde_value::Value;

    use super::*;

    #[test]
    fn test_basic() {
        let cfg = r#"
refresh_rate: 60

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
  - name: foo::bar::baz
    level: warn
    appenders:
      - baz
    additive: false
"#;

        let actual = parse(cfg).unwrap();

        let expected = Config {
            refresh_rate: Some(DeDuration(Duration::seconds(60))),
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
                                                                 Value::String("debug"
                                                                                   .to_string()));
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
                level: DeLogLevelFilter(LogLevelFilter::Info),
                appenders: Some(vec!["console".to_owned()]),
            }),
            loggers: vec![
                Logger {
                    name: "foo::bar::baz".to_owned(),
                    level: DeLogLevelFilter(LogLevelFilter::Warn),
                    appenders: Some(vec!["baz".to_owned()]),
                    additive: Some(false)
                },
            ],
        };

        println!("{:#?}", actual);
        assert_eq!(expected, actual);
    }
}
