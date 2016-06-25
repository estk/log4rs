//! Implementation of syslog messages format with only the PRI part of the header.

use log::LogRecord;

use append::syslog::consts::{Facility, level_to_severity};

/// Syslog message format that only adds the priority header to the message.
#[derive(Debug)]
pub struct Format;

impl Format {
    /// Creates new `Format` object.
    pub fn new() -> Format {
        Format{}
    }
    /// Creates a plain syslog message for the given log record.
    pub fn apply(&self, rec: &LogRecord) -> String {
    	let priority = Facility::USER as u8 | level_to_severity(rec.level());
    	format!("<{}> {}\n",
    	    priority,
    	    rec.args()
    	)
    }
}
