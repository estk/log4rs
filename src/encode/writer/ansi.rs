use std::io;
use encode::{self, Color, Style};

/// A `encode::Write`r that wraps an `io::Write`r, emitting ANSI escape codes
/// for text style.
pub struct AnsiWriter<W>(W);

impl<W: io::Write> AnsiWriter<W> {
    /// Constructs a new `AnsiWriter`.
    pub fn new(w: W) -> AnsiWriter<W> {
        AnsiWriter(w)
    }
}

impl<W: io::Write> io::Write for AnsiWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: io::Write> encode::Write for AnsiWriter<W> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        let mut buf = *b"\x1b[3_;4_;___";
        buf[3] = color_byte(style.text);
        buf[6] = color_byte(style.background);
        let len = if style.intense {
            buf[8] = b'1';
            buf[9] = b'm';
            10
        } else {
            buf[8] = b'2';
            buf[9] = b'2';
            buf[10] = b'm';
            11
        };
        self.0.write_all(&buf[..len])
    }

    fn reset_style(&mut self) -> io::Result<()> {
        self.0.write_all(b"\x1b[0m")
    }
}

fn color_byte(c: Color) -> u8 {
    match c {
        Color::Black => b'0',
        Color::Red => b'1',
        Color::Green => b'2',
        Color::Yellow => b'3',
        Color::Blue => b'4',
        Color::Magenta => b'5',
        Color::Cyan => b'6',
        Color::White => b'7',
        Color::Default => b'9',
    }
}
