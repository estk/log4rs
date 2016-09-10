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
//!     "thread": "main"
//! }
//! ```

use chrono::{DateTime, Local};
use log::{LogLevel, LogRecord};
use std::error::Error;
use std::fmt;
use std::thread;
use serde::ser::{self, Serialize};
use serde_json;

use encode::{Encode, Write, NEWLINE};
#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};

include!("serde.rs");

/// An `Encode`r which writes a JSON object.
#[derive(Debug)]
pub struct JsonEncoder(());

impl JsonEncoder {
    /// Returns a new `JsonEncoder` with a default configuration.
    pub fn new() -> JsonEncoder {
        JsonEncoder(())
    }
}

impl JsonEncoder {
    fn encode_inner(&self,
                    w: &mut Write,
                    time: DateTime<Local>,
                    level: LogLevel,
                    target: &str,
                    module_path: &str,
                    file: &str,
                    line: u32,
                    args: &fmt::Arguments)
                    -> Result<(), Box<Error>> {
        let message = Message {
            time: time,
            level: level,
            target: target,
            module_path: module_path,
            file: file,
            line: line,
            args: args,
        };
        try!(message.serialize(&mut serde_json::Serializer::new(&mut *w)));
        try!(w.write_all(NEWLINE.as_bytes()));
        Ok(())
    }
}

impl Encode for JsonEncoder {
    fn encode(&self, w: &mut Write, record: &LogRecord) -> Result<(), Box<Error>> {
        self.encode_inner(w,
                          Local::now(),
                          record.level(),
                          record.target(),
                          record.location().module_path(),
                          record.location().file(),
                          record.location().line(),
                          record.args())
    }
}

struct Message<'a> {
    time: DateTime<Local>,
    level: LogLevel,
    target: &'a str,
    module_path: &'a str,
    file: &'a str,
    line: u32,
    args: &'a fmt::Arguments<'a>,
}

impl<'a> ser::Serialize for Message<'a> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer
    {
        let mut state = try!(serializer.serialize_map(None));

        try!(serializer.serialize_map_key(&mut state, "time"));
        try!(serializer.serialize_map_value(&mut state, self.time.to_rfc3339()));

        try!(serializer.serialize_map_key(&mut state, "message"));
        try!(serializer.serialize_map_value(&mut state, format!("{}", self.args)));

        try!(serializer.serialize_map_key(&mut state, "module_path"));
        try!(serializer.serialize_map_value(&mut state, self.module_path));

        try!(serializer.serialize_map_key(&mut state, "file"));
        try!(serializer.serialize_map_value(&mut state, self.file));

        try!(serializer.serialize_map_key(&mut state, "line"));
        try!(serializer.serialize_map_value(&mut state, self.line));

        try!(serializer.serialize_map_key(&mut state, "level"));
        try!(serializer.serialize_map_value(&mut state, level_str(self.level)));

        try!(serializer.serialize_map_key(&mut state, "target"));
        try!(serializer.serialize_map_value(&mut state, self.target));

        try!(serializer.serialize_map_key(&mut state, "thread"));
        try!(serializer.serialize_map_value(&mut state, thread::current().name()));

        serializer.serialize_map_end(state)
    }
}

fn level_str(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => "WARN",
        LogLevel::Info => "INFO",
        LogLevel::Debug => "DEBUG",
        LogLevel::Trace => "TRACE",
    }
}

/// A deserializer for the `JsonEncoder`.
///
/// # Configuration
///
/// ```yaml
/// kind: json
/// ```
#[cfg(feature = "file")]
pub struct JsonEncoderDeserializer;

#[cfg(feature = "file")]
impl Deserialize for JsonEncoderDeserializer {
    type Trait = Encode;

    type Config = JsonEncoderConfig;

    fn deserialize(&self,
                   _: JsonEncoderConfig,
                   _: &Deserializers)
                   -> Result<Box<Encode>, Box<Error>> {
        Ok(Box::new(JsonEncoder::new()))
    }
}

#[cfg(test)]
#[cfg(feature = "simple_writer")]
mod test {
    use chrono::{DateTime, Local};
    use log::LogLevel;

    use encode::writer::simple::SimpleWriter;
    use super::*;

    #[test]
    fn default() {
        let time = DateTime::parse_from_rfc3339("2016-03-20T14:22:20.644420340-08:00")
            .unwrap()
            .with_timezone(&Local);
        let level = LogLevel::Debug;
        let target = "target";
        let module_path = "module_path";
        let file = "file";
        let line = 100;
        let message = "message";
        let thread = "encode::json::test::default";

        let encoder = JsonEncoder::new();

        let mut buf = vec![];
        encoder.encode_inner(&mut SimpleWriter(&mut buf),
                          time,
                          level,
                          target,
                          module_path,
                          file,
                          line,
                          &format_args!("{}", message))
            .unwrap();

        let expected = format!("{{\"time\":\"{}\",\"message\":\"{}\",\"module_path\":\"{}\",\
                                \"file\":\"{}\",\"line\":{},\"level\":\"{}\",\"target\":\"{}\",\
                                \"thread\":\"{}\"}}\n",
                               time.to_rfc3339(),
                               message,
                               module_path,
                               file,
                               line,
                               level,
                               target,
                               thread);
        assert_eq!(expected, String::from_utf8(buf).unwrap());
    }
}
