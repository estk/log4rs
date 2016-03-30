//! Appenders

use std::io;
use std::error::Error;
use std::fmt;
use log::LogRecord;

use encode;

pub mod file;
pub mod console;

struct SimpleWriter<W>(W);

impl<W: io::Write> io::Write for SimpleWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: io::Write> encode::Write for SimpleWriter<W> {}

/// A trait implemented by log4rs appenders.
pub trait Append: fmt::Debug + Send + Sync + 'static {
    /// Processes the provided `LogRecord`.
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>>;
}
