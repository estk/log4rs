//! The console writer
//!
//! Requires the `console_writer` feature.

use std::{fmt, io};

use crate::encode::{self, Style};
use std::sync::OnceLock;

static COLOR_MODE: OnceLock<ColorMode> = OnceLock::new();

fn color_mode() -> &'static ColorMode {
    COLOR_MODE.get_or_init(|| {
        let no_color = std::env::var("NO_COLOR")
            .map(|var| var != "0")
            .unwrap_or(false);
        let clicolor_force = std::env::var("CLICOLOR_FORCE")
            .map(|var| var != "0")
            .unwrap_or(false);
        if no_color {
            ColorMode::Never
        } else if clicolor_force {
            ColorMode::Always
        } else {
            let clicolor = std::env::var("CLICOLOR")
                .map(|var| var != "0")
                .unwrap_or(true);
            if clicolor {
                ColorMode::Auto
            } else {
                ColorMode::Never
            }
        }
    })
}

/// The color output mode for a `ConsoleAppender`
#[derive(Clone, Copy, Default)]
pub enum ColorMode {
    /// Print color only if the output is recognized as a console
    #[default]
    Auto,
    /// Force color output
    Always,
    /// Never print color
    Never,
}

/// An `encode::Write`r that outputs to a console.
pub struct ConsoleWriter(imp::Writer);

impl ConsoleWriter {
    /// Returns a new `ConsoleWriter` that will write to standard out.
    ///
    /// Returns `None` if standard out is not a console buffer on Windows, and
    /// if it is not a TTY on Unix.
    pub fn stdout() -> Option<ConsoleWriter> {
        imp::Writer::stdout().map(ConsoleWriter)
    }

    /// Returns a new `ConsoleWriter` that will write to standard error.
    ///
    /// Returns `None` if standard error is not a console buffer on Windows, and
    /// if it is not a TTY on Unix.
    pub fn stderr() -> Option<ConsoleWriter> {
        imp::Writer::stderr().map(ConsoleWriter)
    }

    /// Locks the console, preventing other threads from writing concurrently.
    pub fn lock(&self) -> ConsoleWriterLock {
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
    use std::{fmt, io};

    use crate::{
        encode::{
            self,
            writer::{
                ansi::AnsiWriter,
                console::{color_mode, ColorMode},
            },
            Style,
        },
        priv_io::{StdWriter, StdWriterLock},
    };

    pub struct Writer(AnsiWriter<StdWriter>);

    impl Writer {
        pub fn stdout() -> Option<Writer> {
            let writer = || Writer(AnsiWriter(StdWriter::stdout()));
            match color_mode() {
                ColorMode::Auto => {
                    if unsafe { libc::isatty(libc::STDOUT_FILENO) } != 1 {
                        None
                    } else {
                        Some(writer())
                    }
                }
                ColorMode::Always => Some(writer()),
                ColorMode::Never => None,
            }
        }

        pub fn stderr() -> Option<Writer> {
            let writer = || Writer(AnsiWriter(StdWriter::stderr()));
            match color_mode() {
                ColorMode::Auto => {
                    if unsafe { libc::isatty(libc::STDERR_FILENO) } != 1 {
                        None
                    } else {
                        Some(writer())
                    }
                }
                ColorMode::Always => Some(writer()),
                ColorMode::Never => None,
            }
        }

        pub fn lock(&self) -> WriterLock {
            WriterLock(AnsiWriter((self.0).0.lock()))
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

    pub struct WriterLock<'a>(AnsiWriter<StdWriterLock<'a>>);

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

#[cfg(windows)]
mod imp {
    use std::{
        fmt,
        io::{self, Write},
        mem,
    };
    use winapi::{
        shared::minwindef,
        um::{handleapi, processenv, winbase, wincon, winnt},
    };

    use crate::{
        encode::{
            self,
            writer::console::{color_mode, ColorMode},
            Color, Style,
        },
        priv_io::{StdWriter, StdWriterLock},
    };

    struct RawConsole {
        handle: winnt::HANDLE,
        defaults: minwindef::WORD,
    }

    unsafe impl Sync for RawConsole {}
    unsafe impl Send for RawConsole {}

    impl RawConsole {
        fn set_style(&self, style: &Style) -> io::Result<()> {
            let mut attrs = self.defaults;

            if let Some(text) = style.text {
                attrs &= !((wincon::FOREGROUND_RED
                    | wincon::FOREGROUND_GREEN
                    | wincon::FOREGROUND_BLUE) as minwindef::WORD);
                attrs |= match text {
                    Color::Black => 0,
                    Color::Red => wincon::FOREGROUND_RED,
                    Color::Green => wincon::FOREGROUND_GREEN,
                    Color::Yellow => wincon::FOREGROUND_RED | wincon::FOREGROUND_GREEN,
                    Color::Blue => wincon::FOREGROUND_BLUE,
                    Color::Magenta => wincon::FOREGROUND_RED | wincon::FOREGROUND_BLUE,
                    Color::Cyan => wincon::FOREGROUND_GREEN | wincon::FOREGROUND_BLUE,
                    Color::White => {
                        wincon::FOREGROUND_RED | wincon::FOREGROUND_GREEN | wincon::FOREGROUND_BLUE
                    }
                } as minwindef::WORD;
            }

            if let Some(background) = style.background {
                attrs &= !((wincon::BACKGROUND_RED
                    | wincon::BACKGROUND_GREEN
                    | wincon::BACKGROUND_BLUE) as minwindef::WORD);
                attrs |= match background {
                    Color::Black => 0,
                    Color::Red => wincon::BACKGROUND_RED,
                    Color::Green => wincon::BACKGROUND_GREEN,
                    Color::Yellow => wincon::BACKGROUND_RED | wincon::BACKGROUND_GREEN,
                    Color::Blue => wincon::BACKGROUND_BLUE,
                    Color::Magenta => wincon::BACKGROUND_RED | wincon::BACKGROUND_BLUE,
                    Color::Cyan => wincon::BACKGROUND_GREEN | wincon::BACKGROUND_BLUE,
                    Color::White => {
                        wincon::BACKGROUND_RED | wincon::BACKGROUND_GREEN | wincon::BACKGROUND_BLUE
                    }
                } as minwindef::WORD;
            }

            if let Some(intense) = style.intense {
                if intense {
                    attrs |= wincon::FOREGROUND_INTENSITY as minwindef::WORD;
                } else {
                    attrs &= !(wincon::FOREGROUND_INTENSITY as minwindef::WORD);
                }
            }

            if unsafe { wincon::SetConsoleTextAttribute(self.handle, attrs) } == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    pub struct Writer {
        console: RawConsole,
        inner: StdWriter,
    }

    impl Writer {
        pub fn stdout() -> Option<Writer> {
            unsafe {
                let handle = processenv::GetStdHandle(winbase::STD_OUTPUT_HANDLE);
                if handle.is_null() || handle == handleapi::INVALID_HANDLE_VALUE {
                    return None;
                }

                let mut info = mem::zeroed();
                if wincon::GetConsoleScreenBufferInfo(handle, &mut info) == 0 {
                    return None;
                }

                let writer = Writer {
                    console: RawConsole {
                        handle,
                        defaults: info.wAttributes,
                    },
                    inner: StdWriter::stdout(),
                };

                match color_mode() {
                    ColorMode::Auto | ColorMode::Always => Some(writer),
                    ColorMode::Never => None,
                }
            }
        }

        pub fn stderr() -> Option<Writer> {
            unsafe {
                let handle = processenv::GetStdHandle(winbase::STD_ERROR_HANDLE);
                if handle.is_null() || handle == handleapi::INVALID_HANDLE_VALUE {
                    return None;
                }

                let mut info = mem::zeroed();
                if wincon::GetConsoleScreenBufferInfo(handle, &mut info) == 0 {
                    return None;
                }

                let writer = Writer {
                    console: RawConsole {
                        handle,
                        defaults: info.wAttributes,
                    },
                    inner: StdWriter::stdout(),
                };

                match color_mode() {
                    ColorMode::Auto | ColorMode::Always => Some(writer),
                    ColorMode::Never => None,
                }
            }
        }

        pub fn lock<'a>(&'a self) -> WriterLock<'a> {
            WriterLock {
                console: &self.console,
                inner: self.inner.lock(),
            }
        }
    }

    impl io::Write for Writer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }

        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            self.inner.write_all(buf)
        }

        fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
            self.inner.write_fmt(fmt)
        }
    }

    impl encode::Write for Writer {
        fn set_style(&mut self, style: &Style) -> io::Result<()> {
            self.inner.flush()?;
            self.console.set_style(style)
        }
    }

    pub struct WriterLock<'a> {
        console: &'a RawConsole,
        inner: StdWriterLock<'a>,
    }

    impl<'a> io::Write for WriterLock<'a> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }

        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            self.inner.write_all(buf)
        }

        fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
            self.inner.write_fmt(fmt)
        }
    }

    impl<'a> encode::Write for WriterLock<'a> {
        fn set_style(&mut self, style: &Style) -> io::Result<()> {
            self.inner.flush()?;
            self.console.set_style(style)
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use super::*;
    use crate::encode::{Color, Style, Write as EncodeWrite};

    #[test]
    fn basic() {
        let w = match ConsoleWriter::stdout() {
            Some(w) => w,
            None => return,
        };
        let mut w = w.lock();

        w.write_all(b"normal ").unwrap();
        w.set_style(
            Style::new()
                .text(Color::Red)
                .background(Color::Blue)
                .intense(true),
        )
        .unwrap();
        w.write_all(b"styled").unwrap();
        w.set_style(Style::new().text(Color::Green)).unwrap();
        w.write_all(b" styled2").unwrap();
        w.set_style(&Style::new()).unwrap();
        w.write_all(b" normal\n").unwrap();
        w.flush().unwrap();
    }
}
