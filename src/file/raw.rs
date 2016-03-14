use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::time::Duration;
use serde::de::{self, Deserialize, Deserializer};
use serde_value::Value;
use log::LogLevelFilter;

use file::Format;
use priv_serde::Undeserializable;

include!("serde.rs");

#[derive(PartialEq, Debug)]
pub struct DeDuration(pub Duration);

impl Deserialize for DeDuration {
    fn deserialize<D>(d: &mut D) -> Result<DeDuration, D::Error>
        where D: Deserializer
    {
        u64::deserialize(d).map(|r| DeDuration(Duration::from_secs(r)))
    }
}

#[derive(PartialEq, Debug)]
pub struct DeLogLevelFilter(pub LogLevelFilter);

impl Deserialize for DeLogLevelFilter {
    fn deserialize<D>(d: &mut D) -> Result<DeLogLevelFilter, D::Error>
        where D: Deserializer
    {
        struct V;

        impl de::Visitor for V {
            type Value = DeLogLevelFilter;

            fn visit_str<E>(&mut self, v: &str) -> Result<DeLogLevelFilter, E>
                where E: de::Error
            {
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

#[derive(PartialEq, Debug)]
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

pub fn parse(format: Format, _config: &str) -> Result<Config, Box<Error>> {
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => ::serde_yaml::from_str(_config).map_err(Into::into),
        #[cfg(feature = "json")]
        Format::Json => ::serde_json::from_str(_config).map_err(Into::into),
        #[cfg(feature = "toml")]
        Format::Toml => {
            let mut parser = ::toml::Parser::new(_config);
            let table = match parser.parse() {
                Some(table) => ::toml::Value::Table(table),
                None => return Err(parser.errors.pop().unwrap().into()),
            };
            Config::deserialize(&mut ::toml::Decoder::new(table)).map_err(Into::into)
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use std::borrow::ToOwned;
    use std::collections::{HashMap, BTreeMap};
    use std::time::Duration;
    use log::LogLevelFilter;
    use serde_value::Value;

    use super::*;
    use file::Format;
    use priv_serde::Undeserializable;

    #[allow(dead_code)]
    fn expected() -> Config {
        Config {
            refresh_rate: Some(DeDuration(Duration::from_secs(60))),
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
                level: DeLogLevelFilter(LogLevelFilter::Info),
                appenders: vec!["console".to_owned()],
                _p: Undeserializable,
            }),
            loggers: {
                let mut m = HashMap::new();
                m.insert("foo::bar::baz".to_owned(),
                         Logger {
                             level: DeLogLevelFilter(LogLevelFilter::Warn),
                             appenders: vec!["baz".to_owned()],
                             additive: Some(false),
                             _p: Undeserializable,
                         });
                m
            },
            _p: Undeserializable,
        }
    }

    #[test]
    #[cfg(feature = "yaml")]
    fn basic_yaml() {
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
    "refresh_rate": 60,
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
refresh_rate = 60

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
