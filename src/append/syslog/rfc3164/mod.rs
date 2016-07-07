//! Adaptation of RFC 3164 log messages format to the standard Rust log facade.

extern crate time;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod serde;

use log::LogRecord;
use serde::de;
use std::error;
use std::str;

use append::syslog::consts::{Facility, NILVALUE, level_to_severity, parse_facility};
use append::syslog::rfc3164::serde::FormatConfig;
use file;
use serde_value::Value;

const MAX_MSG_LEN: u32 = 1024;	// Max message length allowed by RFC 3164

/// RFC 3164 formatter.
#[derive(Debug)]
pub struct Format {
	facility: Facility,
	hostname: String,
	app_name: String
}

impl Format {
    /// Creates default RFC 3164 format.
    pub fn default() -> Format {
        Format {
            facility: Facility::USER,
            hostname: String::from(NILVALUE),
			app_name: String::from(NILVALUE)
        }
    }

    /// Sets the facility for to log messages.
    pub fn facility(&mut self, f: Facility) {
        self.facility = f;
    }

    /// Sets the `HOSTNAME` value to put to log messages.
    pub fn hostname(&mut self, h: String) {
        self.hostname = h;
    }

    /// Sets the `APP-NAME` value to put to log messages.
    pub fn app_name(&mut self, a: String) {
        self.app_name = a;
    }

    /// Creates RFC 3164 message for the given log record
    pub fn apply(&self, rec: &LogRecord) -> (String, u32) {
    	let priority = self.facility as u8 | level_to_severity(rec.level());
    	let msg = format!("<{}>{} {} {}: {}\n",
    	    priority,
    	    time::now().strftime("%b %d %T").unwrap(),
    	    self.hostname,
			self.app_name,
    	    rec.args()
    	);
		(msg, MAX_MSG_LEN)
    }
}

/// Deserializer for `rfc3164::Format`.
pub struct FormatDeserializer;

impl file::Deserialize for FormatDeserializer {
    type Trait = Format;

    fn deserialize(&self, config: Value, _: &file::Deserializers) -> Result<Box<Format>, Box<error::Error>> {
        let cfg = try!(config.deserialize_into::<FormatConfig>());
        let mut fmt = Format::default();
        if let Some(fcl) = cfg.facility {
            let fcl = parse_facility(&fcl);
            match fcl {
                Ok(f)    => fmt.facility(f),
                Err(err) => return Err(Box::new(err))
            }
        }
        if let Some(host) = cfg.hostname {
            fmt.hostname(host);
        }
        if let Some(app) = cfg.app_name {
            fmt.app_name(app);
        }
        Ok(Box::new(fmt))
    }
}

impl de::Deserialize for Format {
    fn deserialize<D>(_: &mut D) -> Result<Format, D::Error>
        where D: de::Deserializer
    {
        Ok(Format::default())
    }
}
