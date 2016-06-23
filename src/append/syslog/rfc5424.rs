//! Implementation of RFC 5424 log messages format.

extern crate time;

use log::{LogLevel, LogRecord};
use std::str;
use std::env;

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
	SECURITY = 10 << 3,
	/// FTP daemon
	FTP      = 11 << 3,
	/// NTP subsystem
	NTP      = 12 << 3,
	/// Log audit
	LOGAU    = 13 << 3,
	/// Log alert
	LOGAL    = 14 << 3,
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

/// Serverities according to RFC 5424
#[derive(Debug)]
pub enum Severity {
	/// Emergency: system is unusable
	EMERGENCY   = 0,
	/// Alert: action must be taken immediately
	ALERT		= 1,
	/// Critical: critical conditions
	CRITICAL	= 2,
	/// Error: error conditions
	ERROR		= 3,
	/// Warning: warning conditions
	WARNING		= 4,
	/// Notice: normal but significant condition
	NOTICE		= 5,
	/// Informational: informational messages
	INFO		= 6,
	/// Debug: debug-level messages
	DEBUG		= 7
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
            bom: false
        }
    }

    /// Creates RFC 5424 message for the given log record
    pub fn apply(&self, rec: &LogRecord) -> String {
    	let priority = self.facility as u8 | severity(rec.level());
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

/// Converts log level to RFC 5424 severity
fn severity(lvl: LogLevel) -> u8 {
	match lvl {
		LogLevel::Error => Severity::ERROR as u8,
		LogLevel::Warn  => Severity::WARNING as u8,
		LogLevel::Info  => Severity::INFO as u8,
		LogLevel::Debug => Severity::DEBUG as u8,
		LogLevel::Trace => Severity::DEBUG as u8
	}
}
