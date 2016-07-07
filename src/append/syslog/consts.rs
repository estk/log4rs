//! Common syslog constants.

use log::LogLevel;
use std::io;

/// The syslog `NILVALUE` constant.
pub const NILVALUE: &'static str = "-";

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

/// Severities as defined in RFC 3164/5424
#[derive(Debug)]
pub enum Severity {
	/// Emergency: system is unusable
	EMERGENCY = 0,
	/// Alert: action must be taken immediately
	ALERT     = 1,
	/// Critical: critical conditions
	CRITICAL  = 2,
	/// Error: error conditions
	ERROR     = 3,
	/// Warning: warning conditions
	WARNING   = 4,
	/// Notice: normal but significant condition
	NOTICE    = 5,
	/// Informational: informational messages
	INFO      = 6,
	/// Debug: debug-level messages
	DEBUG     = 7
}

/// Converts log level to syslog severity
#[doc(hidden)]
pub fn level_to_severity(lvl: LogLevel) -> u8 {
	match lvl {
		LogLevel::Error => Severity::ERROR as u8,
		LogLevel::Warn  => Severity::WARNING as u8,
		LogLevel::Info  => Severity::INFO as u8,
		LogLevel::Debug => Severity::DEBUG as u8,
		LogLevel::Trace => Severity::DEBUG as u8
	}
}

#[doc(hidden)]
pub fn parse_facility(f: &String) -> Result<Facility, io::Error> {
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
