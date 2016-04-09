use std::io;
use std::fmt;
use encode;

/// An `encode::Write`r that simply delegates to an `io::Write`r, and relying
/// on the default implementations of `encode::Write`r methods.
#[derive(Debug)]
pub struct SimpleWriter<W>(W);

impl<W: io::Write> SimpleWriter<W> {
    /// Constructs a new `SimpleWriter`.
    pub fn new(w: W) -> SimpleWriter<W> {
        SimpleWriter(w)
    }
}

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
