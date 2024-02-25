use std::{
    fmt,
    io::{self, Stderr, StderrLock, Stdout, StdoutLock},
};

pub enum StdWriter {
    Stdout(Stdout),
    Stderr(Stderr),
}

impl StdWriter {
    pub fn stdout() -> StdWriter {
        StdWriter::Stdout(io::stdout())
    }

    pub fn stderr() -> StdWriter {
        StdWriter::Stderr(io::stderr())
    }

    pub fn lock(&self) -> StdWriterLock {
        match *self {
            StdWriter::Stdout(ref w) => StdWriterLock::Stdout(w.lock()),
            StdWriter::Stderr(ref w) => StdWriterLock::Stderr(w.lock()),
        }
    }
}

impl io::Write for StdWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            StdWriter::Stdout(ref mut w) => w.write(buf),
            StdWriter::Stderr(ref mut w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            StdWriter::Stdout(ref mut w) => w.flush(),
            StdWriter::Stderr(ref mut w) => w.flush(),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match *self {
            StdWriter::Stdout(ref mut w) => w.write_all(buf),
            StdWriter::Stderr(ref mut w) => w.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        match *self {
            StdWriter::Stdout(ref mut w) => w.write_fmt(fmt),
            StdWriter::Stderr(ref mut w) => w.write_fmt(fmt),
        }
    }
}

pub enum StdWriterLock<'a> {
    Stdout(StdoutLock<'a>),
    Stderr(StderrLock<'a>),
}

impl<'a> io::Write for StdWriterLock<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            StdWriterLock::Stdout(ref mut w) => w.write(buf),
            StdWriterLock::Stderr(ref mut w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            StdWriterLock::Stdout(ref mut w) => w.flush(),
            StdWriterLock::Stderr(ref mut w) => w.flush(),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match *self {
            StdWriterLock::Stdout(ref mut w) => w.write_all(buf),
            StdWriterLock::Stderr(ref mut w) => w.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        match *self {
            StdWriterLock::Stdout(ref mut w) => w.write_fmt(fmt),
            StdWriterLock::Stderr(ref mut w) => w.write_fmt(fmt),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_writer_lock() {
        let writer = StdWriter::stderr();
        let mut writer = writer.lock();

        assert_eq!(writer.write(b"test stdwriter ; ").unwrap(), 17);
        assert!(writer.write_all(b"test stdwriter ; ").is_ok());
        assert!(writer
            .write_fmt(format_args!("{}\n", "test stdwriter"))
            .is_ok());
        assert!(writer.flush().is_ok());
    }
}
