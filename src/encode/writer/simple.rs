//! The simple writer
//!
//! Requires the `simple_writer` feature.

use crate::encode;
use std::{fmt, io};

/// An `encode::Write`r that simply delegates to an `io::Write`r and relies
/// on the default implementations of `encode::Write`r methods.
#[derive(Debug)]
pub struct SimpleWriter<W>(pub W);

impl<W: io::Write> io::Write for SimpleWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }
}

impl<W: io::Write> encode::Write for SimpleWriter<W> {}
