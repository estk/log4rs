//! 

use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::time::Duration;
use serde::de::{self, Deserialize, Deserializer};
use serde_value::Value;
use log::LogLevelFilter;

use file::Format;
use priv_serde::{Undeserializable, DeLogLevelFilter, DeDuration};

include!("serde.rs");

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

