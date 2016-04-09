use std::io;
use std::fmt;

use encode::{self, Style};

/// A writer that outputs to a console.
pub struct ConsoleWriter(imp::Writer);

impl ConsoleWriter {
    /// Returns a new `ConsoleWriter` that will write to standard out if it is
    /// a console.
    pub fn stdout() -> Option<ConsoleWriter> {
        imp::Writer::stdout().map(ConsoleWriter)
    }

    /// Locks the console, preventing other threads from writing concurrently.
    pub fn lock<'a>(&'a self) -> ConsoleWriterLock<'a> {
        ConsoleWriterLock(self.0.lock())
    }
}

impl io::Write for ConsoleWriter {
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

impl encode::Write for ConsoleWriter {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.0.set_style(style)
    }
}

/// An RAII lock over a console.
pub struct ConsoleWriterLock<'a>(imp::WriterLock<'a>);

impl<'a> io::Write for ConsoleWriterLock<'a> {
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

impl<'a> encode::Write for ConsoleWriterLock<'a> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.0.set_style(style)
    }
}

#[cfg(unix)]
mod imp {
    use std::io::{self, Stdout, StdoutLock};
    use std::fmt;
    use libc;

    use encode::{self, Style};
    use encode::writer::AnsiWriter;

    pub struct Writer(AnsiWriter<Stdout>);

    impl Writer {
        pub fn stdout() -> Option<Writer> {
            if unsafe { libc::isatty(libc::STDOUT_FILENO) } != 1 {
                return None;
            }

            Some(Writer(AnsiWriter::new(io::stdout())))
        }

        pub fn lock<'a>(&'a self) -> WriterLock<'a> {
            WriterLock(AnsiWriter::new(self.0.get_ref().lock()))
        }
    }

    impl io::Write for Writer {
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

    impl encode::Write for Writer {
        fn set_style(&mut self, style: &Style) -> io::Result<()> {
            self.0.set_style(style)
        }
    }

    pub struct WriterLock<'a>(AnsiWriter<StdoutLock<'a>>);

    impl<'a> io::Write for WriterLock<'a> {
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

    impl<'a> encode::Write for WriterLock<'a> {
        fn set_style(&mut self, style: &Style) -> io::Result<()> {
            self.0.set_style(style)
        }
    }
}
