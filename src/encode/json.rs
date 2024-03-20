//! An encoder which writes a JSON object.
//!
//! Each log event will be written as a JSON object on its own line.
//!
//! Requires the `json_encoder` feature.
//!
//! # Contents
//!
//! An example object (note that real output will not be pretty-printed):
//!
//! ```json
//! {
//!     "time": "2016-03-20T14:22:20.644420340-08:00",
//!     "message": "the log message",
//!     "module_path": "foo::bar",
//!     "file": "foo/bar/mod.rs",
//!     "line": 100,
//!     "level": "INFO",
//!     "target": "foo::bar",
//!     "thread": "main",
//!     "thread_id": 123,
//!     "mdc": {
//!         "request_id": "123e4567-e89b-12d3-a456-426655440000"
//!     }
//! }
//! ```
//! If the `log_kv` feature is enabled, an additional `attributes` field will
//! contain a map of the record's [log::kv][log_kv] structured logging
//! attributes.
//!
//! [log_kv]: https://docs.rs/log/latest/log/kv/index.html

use chrono::{
    format::{DelayedFormat, Fixed, Item},
    DateTime, Local,
};
use log::{Level, Record};
use serde::ser::{self, Serialize, SerializeMap};
use std::{fmt, option, thread};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};
use crate::encode::{Encode, Write, NEWLINE};

/// The JSON encoder's configuration
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonEncoderConfig {
    #[serde(skip_deserializing)]
    _p: (),
}

/// An `Encode`r which writes a JSON object.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct JsonEncoder(());

impl JsonEncoder {
    /// Returns a new `JsonEncoder` with a default configuration.
    pub fn new() -> Self {
        Self::default()
    }
}

impl JsonEncoder {
    fn encode_inner(
        &self,
        w: &mut dyn Write,
        time: DateTime<Local>,
        record: &Record,
    ) -> anyhow::Result<()> {
        let thread = thread::current();
        let message = Message {
            time: time.format_with_items(Some(Item::Fixed(Fixed::RFC3339)).into_iter()),
            level: record.level(),
            message: record.args(),
            module_path: record.module_path(),
            file: record.file(),
            line: record.line(),
            target: record.target(),
            thread: thread.name(),
            thread_id: thread_id::get(),
            mdc: Mdc,
            #[cfg(feature = "log_kv")]
            attributes: kv::get_attributes(record.key_values())?,
        };
        message.serialize(&mut serde_json::Serializer::new(&mut *w))?;
        w.write_all(NEWLINE.as_bytes())?;
        Ok(())
    }
}

impl Encode for JsonEncoder {
    fn encode(&self, w: &mut dyn Write, record: &Record) -> anyhow::Result<()> {
        self.encode_inner(w, Local::now(), record)
    }
}

#[derive(serde::Serialize)]
struct Message<'a> {
    #[serde(serialize_with = "ser_display")]
    time: DelayedFormat<option::IntoIter<Item<'a>>>,
    level: Level,
    #[serde(serialize_with = "ser_display")]
    message: &'a fmt::Arguments<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    module_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<u32>,
    target: &'a str,
    thread: Option<&'a str>,
    thread_id: usize,
    mdc: Mdc,
    #[cfg(feature = "log_kv")]
    attributes: kv::Map,
}

fn ser_display<T, S>(v: &T, s: S) -> Result<S::Ok, S::Error>
where
    T: fmt::Display,
    S: ser::Serializer,
{
    s.collect_str(v)
}

struct Mdc;

impl ser::Serialize for Mdc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        let mut err = Ok(());
        log_mdc::iter(|k, v| {
            if let Ok(()) = err {
                err = map.serialize_key(k).and_then(|()| map.serialize_value(v));
            }
        });
        err?;

        map.end()
    }
}

/// A deserializer for the `JsonEncoder`.
///
/// # Configuration
///
/// ```yaml
/// kind: json
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct JsonEncoderDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for JsonEncoderDeserializer {
    type Trait = dyn Encode;

    type Config = JsonEncoderConfig;

    fn deserialize(
        &self,
        _: JsonEncoderConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Encode>> {
        Ok(Box::<JsonEncoder>::default())
    }
}
#[cfg(feature = "log_kv")]
mod kv {
    use log::kv::VisitSource;
    use std::collections::BTreeMap;

    pub(crate) type Map = BTreeMap<String, String>;

    pub(crate) fn get_attributes(source: &dyn log::kv::Source) -> anyhow::Result<Map> {
        struct Visitor {
            inner: Map,
        }
        impl<'kvs> VisitSource<'kvs> for Visitor {
            fn visit_pair(
                &mut self,
                key: log::kv::Key<'kvs>,
                value: log::kv::Value<'kvs>,
            ) -> Result<(), log::kv::Error> {
                self.inner.insert(format!("{key}"), format!("{value}"));
                Ok(())
            }
        }
        let mut visitor = Visitor {
            inner: BTreeMap::new(),
        };
        source.visit(&mut visitor)?;
        Ok(visitor.inner)
    }
}

#[cfg(test)]
#[cfg(feature = "simple_writer")]
mod test {
    #[cfg(feature = "chrono")]
    use chrono::{DateTime, Local};
    use log::Level;

    use super::*;
    use crate::encode::writer::simple::SimpleWriter;

    #[test]
    fn default() {
        let time = DateTime::parse_from_rfc3339("2016-03-20T14:22:20.644420340-08:00")
            .unwrap()
            .with_timezone(&Local);
        let level = Level::Debug;
        let target = "target";
        let module_path = "module_path";
        let file = "file";
        let line = 100;
        let message = "message";
        let thread = "encode::json::test::default";
        log_mdc::insert("foo", "bar");

        let encoder = JsonEncoder::new();

        let mut record_builder = Record::builder();
        record_builder
            .level(level)
            .target(target)
            .module_path(Some(module_path))
            .file(Some(file))
            .line(Some(line));

        #[cfg(feature = "log_kv")]
        record_builder.key_values(&[("log_foo", "log_bar")]);

        let mut buf = vec![];
        encoder
            .encode_inner(
                &mut SimpleWriter(&mut buf),
                time,
                &record_builder.args(format_args!("{}", message)).build(),
            )
            .unwrap();

        #[cfg(feature = "log_kv")]
        let expected_attributes = ",\"attributes\":{\"log_foo\":\"log_bar\"}";
        #[cfg(not(feature = "log_kv"))]
        let expected_attributes = "";

        let expected = format!(
            "{{\"time\":\"{}\",\"level\":\"{}\",\"message\":\"{}\",\"module_path\":\"{}\",\
            \"file\":\"{}\",\"line\":{},\"target\":\"{}\",\
            \"thread\":\"{}\",\"thread_id\":{},\"mdc\":{{\"foo\":\"bar\"}}{}}}",
            time.to_rfc3339(),
            level,
            message,
            module_path,
            file,
            line,
            target,
            thread,
            thread_id::get(),
            expected_attributes
        );
        assert_eq!(expected, String::from_utf8(buf).unwrap().trim());
    }
}
