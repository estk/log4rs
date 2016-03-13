//! Encoders

use std::fmt;
use std::io::{self, Write};
use log::LogRecord;

pub mod pattern;

/// A trait implemented by types that can serialize a `LogRecord` into a
/// `Write`r.
pub trait Encode: fmt::Debug + Send + 'static {
    /// Encodes the `LogRecord` into bytes and writes them.
    fn encode(&mut self, w: &mut Write, record: &LogRecord) -> io::Result<()>;
}
