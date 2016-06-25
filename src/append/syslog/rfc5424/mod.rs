//! Adaptation of RFC 5424 log messages format to the standard Rust log facade.

extern crate time;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod serde;

use log::LogRecord;
use serde::de;
use std::error;
use std::io;
use std::str;

use append::syslog::rfc5424::serde::FormatConfig;
use append::syslog::severity;
use file;
use serde_value::Value;

const VERSION: u8 = 1; // Format version
const NILVALUE: &'static str = "-";

/// Facilities according to RFC 5424
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Facility {
	/// Kernel messages
	KERN     = 0,
	/// User-level messages
	USER     = 1  << 3,
	/// Mail system
	MAIL     = 2  << 3,
	/// System daemons
	DAEMON   = 3  << 3,
	/// Security/authorization messages
	AUTH     = 4  << 3,
	/// Messages generated internally by syslogd
	SYSLOG   = 5  << 3,
	/// Line printer subsystem
	LPR      = 6  << 3,
	/// Network news subsystem
	NEWS     = 7  << 3,
	/// UUCP subsystem
	UUCP     = 8  << 3,
	/// Clock daemon
	CRON     = 9  << 3,
	/// Security/authorization messages
	AUTHPRIV = 10 << 3,
	/// FTP daemon
	FTP      = 11 << 3,
	/// NTP subsystem
	NTP      = 12 << 3,
	/// Log audit
	LOGAU    = 13 << 3,
	/// Log alert
	LOGALT   = 14 << 3,
	/// Clock daemon (note 2)
	CRON2    = 15 << 3,
	/// Local use 0  (local0)
	LOCAL0   = 16 << 3,
	/// Local use 1  (local1)
	LOCAL1   = 17 << 3,
	/// Local use 2  (local2)
	LOCAL2   = 18 << 3,
	/// Local use 3  (local3)
	LOCAL3   = 19 << 3,
	/// Local use 4  (local4)
	LOCAL4   = 20 << 3,
	/// Local use 5  (local5)
	LOCAL5   = 21 << 3,
	/// Local use 6  (local6)
	LOCAL6   = 22 << 3,
	/// Local use 7  (local7)
	LOCAL7   = 23 << 3
}

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
    	let priority = self.facility as u8 | severity::level_to_severity(rec.level());
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

fn parse_facility(f: &String) -> Result<Facility, io::Error> {
    let res = match f.to_lowercase().as_str() {
        "kern"       => Facility::KERN,
        "user"       => Facility::USER,
        "mail"       => Facility::MAIL,
        "daemon"     => Facility::DAEMON,
        "auth"       => Facility::AUTH,
        "syslog"     => Facility::SYSLOG,
        "lpr"        => Facility::LPR,
        "news"       => Facility::NEWS,
        "uucp"       => Facility::UUCP,
        "cron"       => Facility::CRON,
        "authpriv"   => Facility::AUTHPRIV,
        "ftp"        => Facility::FTP,
        "ntp"        => Facility::NTP,
        "logau"      => Facility::LOGAU,
        "logalt"     => Facility::LOGALT,
        "cron2"      => Facility::CRON2,
        "local0"     => Facility::LOCAL0,
        "local1"     => Facility::LOCAL1,
        "local2"     => Facility::LOCAL2,
        "local3"     => Facility::LOCAL3,
        "local4"     => Facility::LOCAL4,
        "local5"     => Facility::LOCAL5,
        "local6"     => Facility::LOCAL6,
        "local7"     => Facility::LOCAL7,
        _ => return Err(io::Error::new(io::ErrorKind::Other, format!("Unsupported facility {}", f).as_str()))
    };
    Ok(res)
}
