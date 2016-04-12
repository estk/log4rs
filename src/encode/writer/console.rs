use std::io;
use std::fmt;

use encode::{self, Style};

/// An `encode::Write`r that outputs to a console.
pub struct ConsoleWriter(imp::Writer);

impl ConsoleWriter {
    /// Returns a new `ConsoleWriter` that will write to standard out.
    ///
    /// Returns `None` if standard out is not a console buffer on Windows,
    /// and if it is not a tty on Unix.
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

            Some(Writer(AnsiWriter(io::stdout())))
        }

        pub fn lock<'a>(&'a self) -> WriterLock<'a> {
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

#[cfg(windows)]
mod imp {
    use winapi;
    use kernel32;
    use std::io::{self, Stdout, StdoutLock, Write};
    use std::fmt;
    use std::mem;

    use encode::{self, Style, Color};

    struct RawConsole {
        handle: winapi::HANDLE,
        defaults: winapi::WORD,
    }

    unsafe impl Sync for RawConsole {}
    unsafe impl Send for RawConsole {}

    impl RawConsole {
        fn set_style(&self, style: &Style) -> io::Result<()> {
            let mut attrs = self.defaults;

            if let Some(text) = style.text {
                attrs &= !((winapi::FOREGROUND_RED | winapi::FOREGROUND_GREEN |
                            winapi::FOREGROUND_BLUE) as winapi::WORD);
                attrs |= match text {
                    Color::Black => 0,
                    Color::Red => winapi::FOREGROUND_RED,
                    Color::Green => winapi::FOREGROUND_GREEN,
                    Color::Yellow => winapi::FOREGROUND_RED | winapi::FOREGROUND_GREEN,
                    Color::Blue => winapi::FOREGROUND_BLUE,
                    Color::Magenta => winapi::FOREGROUND_RED | winapi::FOREGROUND_BLUE,
                    Color::Cyan => winapi::FOREGROUND_GREEN | winapi::FOREGROUND_BLUE,
                    Color::White => {
                        winapi::FOREGROUND_RED | winapi::FOREGROUND_GREEN | winapi::FOREGROUND_BLUE
                    }
                } as winapi::WORD;
            }

            if let Some(background) = style.background {
                attrs &= !((winapi::BACKGROUND_RED | winapi::BACKGROUND_GREEN |
                            winapi::BACKGROUND_BLUE) as winapi::WORD);
                attrs |= match background {
                    Color::Black => 0,
                    Color::Red => winapi::BACKGROUND_RED,
                    Color::Green => winapi::BACKGROUND_GREEN,
                    Color::Yellow => winapi::BACKGROUND_RED | winapi::BACKGROUND_GREEN,
                    Color::Blue => winapi::BACKGROUND_BLUE,
                    Color::Magenta => winapi::BACKGROUND_RED | winapi::BACKGROUND_BLUE,
                    Color::Cyan => winapi::BACKGROUND_GREEN | winapi::BACKGROUND_BLUE,
                    Color::White => {
                        winapi::BACKGROUND_RED | winapi::BACKGROUND_GREEN | winapi::BACKGROUND_BLUE
                    }
                } as winapi::WORD;
            }

            if let Some(intense) = style.intense {
                if intense {
                    attrs |= winapi::FOREGROUND_INTENSITY as winapi::WORD;
                } else {
                    attrs &= !(winapi::FOREGROUND_INTENSITY as winapi::WORD);
                }
            }

            if unsafe { kernel32::SetConsoleTextAttribute(self.handle, attrs) } == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    pub struct Writer {
        console: RawConsole,
        stdout: Stdout,
    }

    impl Writer {
        pub fn stdout() -> Option<Writer> {
            unsafe {
                let handle = kernel32::GetStdHandle(winapi::STD_OUTPUT_HANDLE);
                if handle.is_null() || handle == winapi::INVALID_HANDLE_VALUE {
                    return None;
                }

                let mut info = mem::zeroed();
                if kernel32::GetConsoleScreenBufferInfo(handle, &mut info) == 0 {
                    return None;
                }

                Some(Writer {
                    console: RawConsole {
                        handle: handle,
                        defaults: info.wAttributes,
                    },
                    stdout: io::stdout(),
                })
            }
        }

        pub fn lock<'a>(&'a self) -> WriterLock<'a> {
            WriterLock {
                console: &self.console,
                stdout: self.stdout.lock(),
            }
        }
    }

    impl io::Write for Writer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.stdout.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.stdout.flush()
        }

        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            self.stdout.write_all(buf)
        }

        fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
            self.stdout.write_fmt(fmt)
        }
    }

    impl encode::Write for Writer {
        fn set_style(&mut self, style: &Style) -> io::Result<()> {
            try!(self.stdout.flush());
            self.console.set_style(style)
        }
    }

    pub struct WriterLock<'a> {
        console: &'a RawConsole,
        stdout: StdoutLock<'a>,
    }

    impl<'a> io::Write for WriterLock<'a> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.stdout.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.stdout.flush()
        }

        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            self.stdout.write_all(buf)
        }

        fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
            self.stdout.write_fmt(fmt)
        }
    }

    impl<'a> encode::Write for WriterLock<'a> {
        fn set_style(&mut self, style: &Style) -> io::Result<()> {
            try!(self.stdout.flush());
            self.console.set_style(style)
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use encode::{Style, Color};
    use encode::Write as EncodeWrite;
    use super::*;

    #[test]
    fn basic() {
        let w = match ConsoleWriter::stdout() {
            Some(w) => w,
            None => return,
        };
        let mut w = w.lock();

        w.write_all(b"normal ").unwrap();
        w.set_style(Style::new().text(Color::Red).background(Color::Blue).intense(true)).unwrap();
        w.write_all(b"styled").unwrap();
        w.set_style(Style::new().text(Color::Green)).unwrap();
        w.write_all(b" styled2").unwrap();
        w.set_style(&Style::new()).unwrap();
        w.write_all(b" normal\n").unwrap();
        w.flush().unwrap();
    }
}
