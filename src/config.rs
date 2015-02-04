use std::borrow::ToOwned;
use std::collections::HashMap;
use std::time::Duration;

use toml::{self, Value};
use log::LogLevelFilter;

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct RawConfig {
    pub refresh_rate: Option<Duration>,
    pub loggers: HashMap<String, LoggerSpec>,
    pub directives: Vec<LogDirective>,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct LoggerSpec {
    pub kind: String,
    pub config: toml::Table,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct LogDirective {
    pub path: String,
    pub level: LogLevelFilter,
    pub loggers: Vec<String>,
}

pub fn parse_config(config: &str) -> Result<RawConfig, Vec<String>> {
    let mut parser = toml::Parser::new(config);
    match parser.parse() {
        Some(table) => {
            if parser.errors.is_empty() {
                finish_parse_config(table)
            } else {
                Err(make_errors(&parser))
            }
        }
        _ => Err(make_errors(&parser)),
    }
}

fn finish_parse_config(mut table: toml::Table) -> Result<RawConfig, Vec<String>> {
    let mut errors = vec![];

    let refresh_rate = match table.remove("refresh_rate") {
        Some(Value::Integer(refresh_rate)) => Some(Duration::seconds(refresh_rate)),
        Some(_) => {
            errors.push("`refresh_rate` must be an integer".to_owned());
            None
        }
        None => None
    };

    let loggers = match table.remove("logger") {
        Some(Value::Table(table)) => {
            table.into_iter().filter_map(|(name, spec)| {
                let mut spec = match spec {
                    Value::Table(spec) => spec,
                    _ => {
                        errors.push(format!("{} should be a table", name));
                        return None;
                    }
                };

                let kind = match spec.remove("kind") {
                    Some(Value::String(kind)) => kind,
                    Some(_) => {
                        errors.push(format!("`kind` must be a string in logger {}", name));
                        return None;
                    }
                    None => {
                        errors.push(format!("`kind` must be present in logger {}", name));
                        return None;
                    }
                };

                let spec = LoggerSpec {
                    kind: kind,
                    config: spec,
                };

                Some((name, spec))
            }).collect()
        }
        None => HashMap::new(),
        _ => {
            errors.push("`logger` should be a table".to_owned());
            HashMap::new()
        }
    };

    let directives = match table.remove("directive") {
        Some(Value::Array(array)) => {
            array.into_iter().filter_map(|directive| {
                if let Value::Table(mut table) = directive {
                    let path = match table.remove("path") {
                        Some(Value::String(path)) => path,
                        None => String::new(),
                        _ => {
                            errors.push("`path` should be a string".to_owned());
                            String::new()
                        }
                    };

                    let level = match table.remove("level") {
                        Some(Value::String(level)) => {
                            if let Ok(level) = level.parse() {
                                level
                            } else {
                                errors.push("Invalid `level`".to_owned());
                                LogLevelFilter::Off
                            }
                        }
                        _ => {
                            errors.push("`level` must be a string".to_owned());
                            LogLevelFilter::Off
                        }
                    };

                    let logger_list = match table.remove("logger") {
                        Some(Value::String(logger)) => vec![logger],
                        Some(Value::Array(array)) => {
                            array.into_iter().filter_map(|logger| {
                                if let Value::String(logger) = logger {
                                    Some(logger)
                                } else {
                                    errors.push("`logger` should be a string or an array of \
                                                 strings".to_owned());
                                    None
                                }
                            }).collect()
                        }
                        _ => {
                            errors.push("`logger` should be a string or an array of strings"
                                            .to_owned());
                            vec![]
                        }
                    };

                    for key in table.keys() {
                        errors.push(format!("Invalid key `{}`", key));
                    }

                    Some(LogDirective {
                        path: path,
                        level: level,
                        loggers: logger_list,
                    })
                } else {
                    errors.push("`directive` should contain tables".to_owned());
                    None
                }
            }).collect()
        }
        None => vec![],
        _ => {
            errors.push("`directive` should be an array".to_owned());
            vec![]
        }
    };

    if errors.is_empty() {
        Ok(RawConfig {
            refresh_rate: refresh_rate,
            loggers: loggers,
            directives: directives,
        })
    } else {
        Err(errors)
    }
}

fn make_errors(parser: &toml::Parser) -> Vec<String> {
    parser.errors.iter().map(|error| {
        let (lo_line, lo_col) = parser.to_linecol(error.lo);
        let (hi_line, hi_col) = parser.to_linecol(error.hi);
        format!("{}:{}: {}:{} {}", lo_line, lo_col, hi_line, hi_col, error.desc)
    }).collect()
}

#[cfg(test)]
mod test {
    use std::borrow::ToOwned;
    use std::collections::{HashMap, BTreeMap};
    use std::time::Duration;
    use toml::Value;
    use log::LogLevelFilter;

    use super::*;

    #[test]
    fn test_basic() {
        let cfg = r#"
refresh_rate = 60

[logger.console]
kind = "console"

[logger.error]
kind = "file"
file = "log/error.log"

[[directive]]
path = "foo::bar::baz"
level = "warn"
logger = "console"

[[directive]]
level = "error"
logger = "error"
"#;

        let actual = parse_config(cfg).ok().unwrap();

        let expected = RawConfig {
            refresh_rate: Some(Duration::seconds(60)),
            loggers: {
                let mut m = HashMap::new();
                m.insert("console".to_owned(),
                         LoggerSpec {
                             kind: "console".to_owned(),
                             config: BTreeMap::new(),
                         });
                m.insert("error".to_owned(),
                         LoggerSpec {
                             kind: "file".to_owned(),
                             config: {
                                 let mut m = BTreeMap::new();
                                 m.insert("file".to_owned(),
                                          Value::String("log/error.log".to_owned()));
                                 m
                             }
                         });
                m
            },
            directives: vec![
                LogDirective {
                    path: "foo::bar::baz".to_owned(),
                    level: LogLevelFilter::Warn,
                    loggers: vec!["console".to_owned()],
                },
                LogDirective {
                    path: "".to_owned(),
                    level: LogLevelFilter::Error,
                    loggers: vec!["error".to_owned()],
                }
            ]
        };

        assert_eq!(expected, actual);
    }
}
