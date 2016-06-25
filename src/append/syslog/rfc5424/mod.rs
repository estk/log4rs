//! Adaptation of RFC 5424 log messages format to the standard Rust log facade.

extern crate time;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod serde;

use log::LogRecord;
use serde::de;
use std::error;
use std::str;

use append::syslog::consts::{Facility, NILVALUE, level_to_severity, parse_facility};
use append::syslog::rfc5424::serde::FormatConfig;
use file;
use serde_value::Value;

const VERSION: u8 = 1; // Format version

/// RFC 5424 formatter.
#[derive(Debug)]
pub struct Format {
	facility: Facility,
	hostname: String,
	app_name: String,
	procid: String,
	bom: bool
}

impl Format {
    /// Creates default RFC 5424 format.
    pub fn default() -> Format {
        Format {
            facility: Facility::USER,
            hostname: String::from(NILVALUE),
            app_name: String::from(NILVALUE),
            procid: String::from(NILVALUE),
            bom: true
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

    /// Defines if the `BOM` marker should be used in log messages.
    pub fn bom(&mut self, b: bool) {
        self.bom = b;
    }

    /// Creates RFC 5424 message for the given log record
    pub fn apply(&self, rec: &LogRecord) -> String {
    	let priority = self.facility as u8 | level_to_severity(rec.level());
    	let msg_id = 0;
    	let struct_data = NILVALUE;
    	let bom_str;
    	if self.bom {
    	    bom_str = "\u{EF}\u{BB}\u{BF}";
    	} else {
    	    bom_str = "";
    	}
    	format!("<{}>{} {} {} {} {} {} {} {}{}\n",
    	    priority,
    	    VERSION,
    	    time::now_utc().rfc3339(),
    	    self.hostname,
    	    self.app_name,
    	    self.procid,
    	    msg_id,
    	    struct_data,
    	    bom_str,
    	    rec.args()
    	)
    }
}

/// Deserializer for `rfc5424::Format`.
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
        if let Some(b) = cfg.bom {
            fmt.bom(b);
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
