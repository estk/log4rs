//! Definitions of syslog severities and their adaptation to the Rust log facade.

use log::LogLevel;

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
