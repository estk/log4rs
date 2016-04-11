//! A simple pattern-based encoder.
//!
//! The pattern syntax is similar to Rust's string formatting syntax. It
//! consists of raw text interspersed with format arguments. The grammar is:
//!
//! ```not_rust
//! format_string := <text> [ format <text> ] *
//! format := '{' formatter [ ':' format_spec ] '}'
//! formatter := [ name ] [ '(' argument ')' ]
//! name := identifier
//! argument := format_string
//!
//! format_spec := [ [ fill ] align ] [ min_width ] [ '.' max_width ]
//! fill := character
//! align := '<' | '>'
//! min_width := number
//! max_width := number
//! ```
//!
//! # Formatters
//!
//! A formatter inserts a dynamic portion of text into the pattern. It may be
//! text derived from a log event or from some other context like the current
//! time. Formatters may be passed an argument consisting of a parenthesized
//! format string. If an argument is not provided, it is equivalent to an empty
//! format string (i.e.  `{foo}` `{foo()}` are equivalent).
//!
//! The following formatters are currently supported. Unless otherwise stated,
//! a formatter does not accept any argument.
//!
//! * `d`, `date` - The current time. By default, the ISO 8601 format is used.
//!     A custom format may be provided in the syntax accepted by `chrono`.
//!     * `{d}` - `2016-03-20T22:22:20.644420340+00:00`
//!     * `{d(%Y-%m-%d %H:%M:%S)}` - `2016-03-20 22:22:20`
//! * `f`, `file` - The source file that the log message came from.
//! * `h`, `highlight` - Styles its argument according to the log level. The
//!     style is intense red for errors, red for warnings, blue for info, and
//!     the default style for all other levels.
//!     * `{h(the level is {l})}` - <code style="color: red; font-weight: bold">the level is ERROR</code>
//! * `l``, level` - The log level.
//! * `L`, `line` - The line that the log message came from.
//! * `m`, `message` - The log message.
//! * `M`, `module` - The module that the log message came from.
//! * `t`, `target` - The target of the log message.
//! * `T`, `thread` - The name of the current thread.
//! * `n` - A platform-specific newline.
//! * An "unnamed" formatter simply formats its argument, applying the format
//!     specification.
//!     * `{({l} {m})}` - `INFO hello`
//!
//! # Format Specification
//!
//! The format specification determines how the output of a formatter is
//! adjusted before being returned.
//!
//! ## Fill/Alignment
//!
//! The fill and alignment values are used in conjunction with a minimum width
//! value (see below) to control the behavior when a formatter's output is less
//! than the minimum. While the default behavior is to pad the output to the
//! right with space characters (i.e. left align it), the fill value specifies
//! the character used, and the alignment value is one of:
//!
//! * `<` - Left align by appending the fill character to the formatter output
//! * `>` - Right align by prepending the fill character to the formatter
//!     output.
//!
//! ## Width
//!
//! By default, the full contents of a formatter's output will be inserted into
//! the pattern output, but both the minimum and maximum lengths can be
//! configured. Any output over the maximum length will be truncated, and
//! output under the minimum length will be padded (see above).
//!
//! # Examples
//!
//! The default pattern is `{d} {l} {t} - {m}{n}` which produces output like
//! `2016-03-20T22:22:20.644420340+00:00 INFO module::path - this is a log
//! message`.
//!
//! The pattern `{m:>10.15}` will right-align the log message to a minimum of
//! 10 bytes, filling in with space characters, and truncate output after 15
//! bytes. The message `hello` will therefore be displayed as
//! <code>     hello</code>, while the message `hello there, world!` will be
//! displayed as `hello there, wo`.
//!
//! The pattern `{({l} {m}):15.15}` will output the log level and message
//! limited to exactly 15 bytes, padding with space characters on the right if
//! necessary. The message `hello` and log level `INFO` will be displayed as
//! <code>INFO hello     </code>, while the message `hello, world!` and log
//! level `DEBUG` will be truncated to `DEBUG hello, wo`.

use chrono::UTC;
use log::{LogRecord, LogLevel};
use serde_value::Value;
use std::default::Default;
use std::cmp;
use std::error;
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::Write;
use std::thread;

use encode::pattern::parser::{Parser, Piece, Parameters, Alignment};
use encode::pattern::serde::PatternEncoderConfig;
use encode::{self, Encode, Style, Color};
use encode::Write as EncodeWrite;
use file::{Deserialize, Deserializers};
use ErrorInternals;

mod parser;
#[cfg_attr(rustfmt, rustfmt_skip)]
mod serde;

#[cfg(windows)]
const NEWLINE: &'static str = "\r\n";
#[cfg(not(windows))]
const NEWLINE: &'static str = "\n";

struct MaxWidthWriter<'a> {
    remaining: usize,
    w: &'a mut encode::Write,
}

impl<'a> io::Write for MaxWidthWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // we don't want to report EOF, so just act as a sink past this point
        if self.remaining == 0 {
            return Ok(buf.len());
        }

        let buf = &buf[..cmp::min(buf.len(), self.remaining)];
        match self.w.write(buf) {
            Ok(len) => {
                self.remaining -= len;
                Ok(len)
            }
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

impl<'a> encode::Write for MaxWidthWriter<'a> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.w.set_style(style)
    }
}

struct LeftAlignWriter<'a> {
    to_fill: usize,
    fill: char,
    w: MaxWidthWriter<'a>,
}

impl<'a> LeftAlignWriter<'a> {
    fn finish(mut self) -> io::Result<()> {
        for _ in 0..self.to_fill {
            try!(write!(self.w, "{}", self.fill));
        }
        Ok(())
    }
}

impl<'a> io::Write for LeftAlignWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.w.write(buf) {
            Ok(len) => {
                self.to_fill = self.to_fill.saturating_sub(len);
                Ok(len)
            }
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

impl<'a> encode::Write for LeftAlignWriter<'a> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.w.set_style(style)
    }
}

enum BufferedOutput {
    Data(Vec<u8>),
    Style(Style),
}

struct RightAlignWriter<'a> {
    to_fill: usize,
    fill: char,
    w: MaxWidthWriter<'a>,
    buf: Vec<BufferedOutput>,
}

impl<'a> RightAlignWriter<'a> {
    fn finish(mut self) -> io::Result<()> {
        for _ in 0..self.to_fill {
            try!(write!(self.w, "{}", self.fill));
        }
        for out in self.buf {
            match out {
                BufferedOutput::Data(ref buf) => try!(self.w.write_all(buf)),
                BufferedOutput::Style(ref style) => try!(self.w.set_style(style)),
            }
        }
        Ok(())
    }
}

impl<'a> io::Write for RightAlignWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.to_fill = self.to_fill.saturating_sub(buf.len());

        let mut pushed = false;
        if let Some(&mut BufferedOutput::Data(ref mut data)) = self.buf.last_mut() {
            data.extend_from_slice(buf);
            pushed = true;
        };

        if !pushed {
            self.buf.push(BufferedOutput::Data(buf.to_owned()));
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> encode::Write for RightAlignWriter<'a> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.buf.push(BufferedOutput::Style(style.clone()));
        Ok(())
    }
}

enum Chunk {
    Text(String),
    Formatted {
        chunk: FormattedChunk,
        params: Parameters,
    },
    Error(String),
}

impl Chunk {
    fn encode(&self,
              w: &mut encode::Write,
              level: LogLevel,
              target: &str,
              location: &Location,
              args: &fmt::Arguments)
              -> io::Result<()> {
        match *self {
            Chunk::Text(ref s) => w.write_all(s.as_bytes()),
            Chunk::Formatted { ref chunk, ref params } => {
                // fast path for no width requirements
                if params.min_width.is_none() && params.max_width.is_none() {
                    return chunk.encode(w, level, target, location, args);
                }

                let w = MaxWidthWriter {
                    remaining: params.max_width.unwrap_or(usize::max_value()),
                    w: w,
                };

                match params.align {
                    Alignment::Left => {
                        let mut w = LeftAlignWriter {
                            to_fill: params.min_width.unwrap_or(0),
                            fill: params.fill,
                            w: w,
                        };
                        try!(chunk.encode(&mut w, level, target, location, args));
                        w.finish()
                    }
                    Alignment::Right => {
                        let mut w = RightAlignWriter {
                            to_fill: params.min_width.unwrap_or(0),
                            fill: params.fill,
                            w: w,
                            buf: vec![],
                        };
                        try!(chunk.encode(&mut w, level, target, location, args));
                        w.finish()
                    }
                }
            }
            Chunk::Error(ref s) => write!(w, "{{ERROR: {}}}", s),
        }
    }
}

impl<'a> From<Piece<'a>> for Chunk {
    fn from(piece: Piece<'a>) -> Chunk {
        match piece {
            Piece::Text(text) => Chunk::Text(text.to_owned()),
            Piece::Argument { formatter, parameters } => {
                match formatter.name {
                    "d" |
                    "date" => {
                        let mut format = String::new();
                        for piece in &formatter.arg {
                            match *piece {
                                Piece::Text(text) => format.push_str(text),
                                Piece::Argument { .. } => {
                                    format.push_str("{ERROR: unexpected formatter}");
                                }
                                Piece::Error(ref err) => {
                                    format.push_str("{ERROR: ");
                                    format.push_str(err);
                                    format.push('}');
                                }
                            }
                        }
                        if format.is_empty() {
                            format.push_str("%+");
                        }
                        Chunk::Formatted {
                            chunk: FormattedChunk::Time(format),
                            params: parameters,
                        }
                    }
                    "h" |
                    "highlight" => {
                        let chunks = formatter.arg.into_iter().map(From::from).collect();
                        Chunk::Formatted {
                            chunk: FormattedChunk::Highlight(chunks),
                            params: parameters,
                        }
                    }
                    "l" |
                    "level" => no_args(&formatter.arg, parameters, FormattedChunk::Level),
                    "m" |
                    "message" => no_args(&formatter.arg, parameters, FormattedChunk::Message),
                    "M" |
                    "module" => no_args(&formatter.arg, parameters, FormattedChunk::Module),
                    "f" |
                    "file" => no_args(&formatter.arg, parameters, FormattedChunk::File),
                    "L" |
                    "line" => no_args(&formatter.arg, parameters, FormattedChunk::Line),
                    "T" |
                    "thread" => no_args(&formatter.arg, parameters, FormattedChunk::Thread),
                    "t" |
                    "target" => no_args(&formatter.arg, parameters, FormattedChunk::Target),
                    "n" => no_args(&formatter.arg, parameters, FormattedChunk::Newline),
                    "" => {
                        let chunks = formatter.arg.into_iter().map(From::from).collect();
                        Chunk::Formatted {
                            chunk: FormattedChunk::Align(chunks),
                            params: parameters,
                        }
                    }
                    name => Chunk::Error(format!("unknown formatter `{}`", name)),
                }
            }
            Piece::Error(err) => Chunk::Error(err),
        }
    }
}

enum FormattedChunk {
    Time(String),
    Level,
    Message,
    Module,
    File,
    Line,
    Thread,
    Target,
    Newline,
    Align(Vec<Chunk>),
    Highlight(Vec<Chunk>),
}

impl FormattedChunk {
    fn encode(&self,
              w: &mut encode::Write,
              level: LogLevel,
              target: &str,
              location: &Location,
              args: &fmt::Arguments)
              -> io::Result<()> {
        match *self {
            FormattedChunk::Time(ref fmt) => write!(w, "{}", UTC::now().format(fmt)),
            FormattedChunk::Level => write!(w, "{}", level),
            FormattedChunk::Message => w.write_fmt(*args),
            FormattedChunk::Module => w.write_all(location.module_path.as_bytes()),
            FormattedChunk::File => w.write_all(location.file.as_bytes()),
            FormattedChunk::Line => write!(w, "{}", location.line),
            FormattedChunk::Thread => {
                w.write_all(thread::current().name().unwrap_or("<unnamed>").as_bytes())
            }
            FormattedChunk::Target => w.write_all(target.as_bytes()),
            FormattedChunk::Newline => w.write_all(NEWLINE.as_bytes()),
            FormattedChunk::Align(ref chunks) => {
                for chunk in chunks {
                    try!(chunk.encode(w, level, target, location, args));
                }
                Ok(())
            }
            FormattedChunk::Highlight(ref chunks) => {
                match level {
                    LogLevel::Error => {
                        try!(w.set_style(Style::new().text(Color::Red).intense(true)));
                    }
                    LogLevel::Warn => try!(w.set_style(Style::new().text(Color::Red))),
                    LogLevel::Info => try!(w.set_style(Style::new().text(Color::Blue))),
                    _ => {}
                }
                for chunk in chunks {
                    try!(chunk.encode(w, level, target, location, args));
                }
                match level {
                    LogLevel::Error |
                    LogLevel::Warn |
                    LogLevel::Info => try!(w.set_style(&Style::new())),
                    _ => {}
                }
                Ok(())
            }
        }
    }
}

/// An `Encode`r configured via a format string.
pub struct PatternEncoder {
    chunks: Vec<Chunk>,
    pattern: String,
}

impl fmt::Debug for PatternEncoder {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("PatternEncoder")
           .field("pattern", &self.pattern)
           .finish()
    }
}

/// Returns a `PatternEncoder` using the default pattern of `{d} {l} {t} - {m}{n}`.
impl Default for PatternEncoder {
    fn default() -> PatternEncoder {
        PatternEncoder::new("{d} {l} {t} - {m}{n}")
    }
}

impl Encode for PatternEncoder {
    fn encode(&self, w: &mut encode::Write, record: &LogRecord) -> io::Result<()> {
        let location = Location {
            module_path: record.location().module_path(),
            file: record.location().file(),
            line: record.location().line(),
        };
        self.append_inner(w, record.level(), record.target(), &location, record.args())
    }
}

fn no_args(arg: &[Piece], params: Parameters, chunk: FormattedChunk) -> Chunk {
    if arg.is_empty() {
        Chunk::Formatted {
            chunk: chunk,
            params: params,
        }
    } else {
        Chunk::Error("unexpected arguments".to_owned())
    }
}

impl PatternEncoder {
    /// Creates a `PatternEncoder` from a pattern string.
    ///
    /// The pattern string syntax is documented in the `pattern` module.
    pub fn new(pattern: &str) -> PatternEncoder {
        PatternEncoder {
            chunks: Parser::new(pattern).map(From::from).collect(),
            pattern: pattern.to_owned(),
        }
    }

    fn append_inner(&self,
                    w: &mut encode::Write,
                    level: LogLevel,
                    target: &str,
                    location: &Location,
                    args: &fmt::Arguments)
                    -> io::Result<()> {
        for chunk in &self.chunks {
            try!(chunk.encode(w, level, target, location, args));
        }
        Ok(())
    }
}

struct Location<'a> {
    module_path: &'a str,
    file: &'a str,
    line: u32,
}

/// A deserializer for the `PatternEncoder`.
///
/// The `pattern` key is required and specifies the pattern for the encoder.
pub struct PatternEncoderDeserializer;

impl Deserialize for PatternEncoderDeserializer {
    type Trait = Encode;

    fn deserialize(&self,
                   config: Value,
                   _: &Deserializers)
                   -> Result<Box<Encode>, Box<error::Error>> {
        let config = try!(config.deserialize_into::<PatternEncoderConfig>());
        let encoder = match config.pattern {
            Some(pattern) => PatternEncoder::new(&pattern),
            None => PatternEncoder::default(),
        };
        Ok(Box::new(encoder))
    }
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use std::thread;
    use log::LogLevel;

    use super::{PatternEncoder, Location, Chunk};
    use encode::writer::SimpleWriter;

    static LOCATION: Location<'static> = Location {
        module_path: "path",
        file: "file",
        line: 132,
    };

    fn error_free(encoder: &PatternEncoder) -> bool {
        encoder.chunks.iter().all(|c| {
            match *c {
                Chunk::Error(_) => false,
                _ => true,
            }
        })
    }

    #[test]
    fn invalid_formatter() {
        assert!(!error_free(&PatternEncoder::new("{x}")));
    }

    #[test]
    fn unclosed_delimiter() {
        assert!(!error_free(&PatternEncoder::new("{d(%Y-%m-%d)")));
    }

    #[test]
    fn log() {
        let pw = PatternEncoder::new("{l} {m} at {M} in {f}:{L}");
        let mut buf = vec![];
        pw.append_inner(&mut SimpleWriter::new(&mut buf),
                        LogLevel::Debug,
                        "target",
                        &LOCATION,
                        &format_args!("the message"))
          .unwrap();

        assert_eq!(buf, &b"DEBUG the message at path in file:132"[..]);
    }

    #[test]
    fn unnamed_thread() {
        thread::spawn(|| {
            let pw = PatternEncoder::new("{T}");
            let mut buf = vec![];
            pw.append_inner(&mut SimpleWriter::new(&mut buf),
                            LogLevel::Debug,
                            "target",
                            &LOCATION,
                            &format_args!("message"))
              .unwrap();
            assert_eq!(buf, b"<unnamed>");
        })
            .join()
            .unwrap();
    }

    #[test]
    fn named_thread() {
        thread::Builder::new()
            .name("foobar".to_string())
            .spawn(|| {
                let pw = PatternEncoder::new("{T}");
                let mut buf = vec![];
                pw.append_inner(&mut SimpleWriter::new(&mut buf),
                                LogLevel::Debug,
                                "target",
                                &LOCATION,
                                &format_args!("message"))
                  .unwrap();
                assert_eq!(buf, b"foobar");
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn default_okay() {
        assert!(error_free(&PatternEncoder::default()));
    }

    #[test]
    fn left_align() {
        let pw = PatternEncoder::new("{m:~<5.6}");

        let mut buf = vec![];
        pw.append_inner(&mut SimpleWriter::new(&mut buf),
                        LogLevel::Debug,
                        "",
                        &LOCATION,
                        &format_args!("foo"))
          .unwrap();
        assert_eq!(buf, b"foo~~");

        buf.clear();
        pw.append_inner(&mut SimpleWriter::new(&mut buf),
                        LogLevel::Debug,
                        "",
                        &LOCATION,
                        &format_args!("foobar!"))
          .unwrap();
        assert_eq!(buf, b"foobar");
    }

    #[test]
    fn right_align() {
        let pw = PatternEncoder::new("{m:~>5.6}");

        let mut buf = vec![];
        pw.append_inner(&mut SimpleWriter::new(&mut buf),
                        LogLevel::Debug,
                        "",
                        &LOCATION,
                        &format_args!("foo"))
          .unwrap();
        assert_eq!(buf, b"~~foo");

        buf.clear();
        pw.append_inner(&mut SimpleWriter::new(&mut buf),
                        LogLevel::Debug,
                        "",
                        &LOCATION,
                        &format_args!("foobar!"))
          .unwrap();
        assert_eq!(buf, b"foobar");
    }

    #[test]
    fn left_align_formatter() {
        let pw = PatternEncoder::new("{({l} {m}):15}");

        let mut buf = vec![];
        pw.append_inner(&mut SimpleWriter::new(&mut buf),
                        LogLevel::Info,
                        "",
                        &LOCATION,
                        &format_args!("foobar!"))
          .unwrap();
        assert_eq!(buf, b"INFO foobar!   ");
    }

    #[test]
    fn right_align_formatter() {
        let pw = PatternEncoder::new("{({l} {m}):>15}");

        let mut buf = vec![];
        pw.append_inner(&mut SimpleWriter::new(&mut buf),
                        LogLevel::Info,
                        "",
                        &LOCATION,
                        &format_args!("foobar!"))
          .unwrap();
        assert_eq!(buf, b"   INFO foobar!");
    }

    #[test]
    fn custom_date_format() {
        assert!(error_free(&PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {m}{n}")));
    }
}
