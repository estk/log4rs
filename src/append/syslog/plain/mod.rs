//! Implementation of syslog messages format with only a PRI part.

use log::LogRecord;

use append::syslog::severity;

const FACILITY: u8 = 1 << 3; // USER facility

/// Syslog message format that only adds the priority header to the message.
#[derive(Debug)]
pub struct Format;

impl Format {
    /// Creates new `Format` object. 
    pub fn new() -> Format {
        Format{}
    }
    /// Creates RFC 5424 message for the given log record
    pub fn apply(&self, rec: &LogRecord) -> String {
    	let priority = FACILITY | severity::level_to_severity(rec.level());
    	format!("<{}> {}\n",
    	    priority,
    	    rec.args()
    	)
    }
}