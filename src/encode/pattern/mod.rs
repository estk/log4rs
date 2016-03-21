//! A simple pattern-based encoder.
//!
//! The supported syntax is similar to that used by Rust's string formatting
//! infrastructure. It consists of text which will be output verbatim, with
//! formatting specifiers denoted by braces containing the configuration of the
//! formatter. This consists of a formatter name followed optionally by a
//! parenthesized argument.
//!
//! # Supported Formatters
//!
//! * `d`, `date` - The current time. By default, the ISO 8601 format is used.
//!     A custom format may be provided in the syntax accepted by `chrono` as
//!     an argument.
//! * `f`, `file` - The source file that the log message came from.
//! * `l``, level` - The log level.
//! * `L`, `line` - The line that the log message came from.
//! * `m`, `message` - The log message.
//! * `M`, `module` - The module that the log message came from.
//! * `T`, `thread` - The name of the thread that the log message came from.
//! * `t`, `target` - The target of the log message.
//!
//! # Examples
//!
//! The default pattern is `{d} {l} {t} - {m}` which produces output like
//! `2016-03-20T22:22:20.644420340+00:00 INFO module::path - this is a log
//! message`.
//!
//! The pattern `{d(%Y-%m-%d %H:%M:%S)}` will output the current time with a
//! custom format looking like `2016-03-20 22:22:20`.

use chrono::UTC;
use log::{LogRecord, LogLevel};
use serde_value::Value;
use std::default::Default;
use std::error;
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::Write;
use std::thread;

use encode::pattern::parser::{Parser, Piece};
use encode::{self, Encode};
use file::{Deserialize, Deserializers};
use ErrorInternals;

mod parser;

include!("serde.rs");

enum Chunk {
    Text(String),
    Time(String),
    Level,
    Message,
    Module,
    File,
    Line,
    Thread,
    Target,
    Error(String),
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

/// Returns a `PatternEncoder` using the default pattern of `{d} {l} {t} - {m}`.
impl Default for PatternEncoder {
    fn default() -> PatternEncoder {
        PatternEncoder::new("{d} {l} {t} - {m}")
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

fn empty_args(arg: &str) -> Option<Chunk> {
    if arg.is_empty() {
        None
    } else {
        Some(Chunk::Error("unexpected arguments".to_owned()))
    }
}

impl PatternEncoder {
    /// Creates a `PatternEncoder` from a pattern string.
    ///
    /// The pattern string syntax is documented in the `pattern` module.
    pub fn new(pattern: &str) -> PatternEncoder {
        let mut chunks = vec![];

        for piece in Parser::new(pattern) {
            let chunk = match piece {
                Piece::Text(text) => Chunk::Text(text.to_owned()),
                Piece::Argument { formatter, parameters } => {
                    match formatter.name {
                        "d" |
                        "date" => {
                            let format = if formatter.arg.is_empty() {
                                "%+".to_owned()
                            } else {
                                formatter.arg.to_owned()
                            };
                            Chunk::Time(format)
                        }
                        "l" |
                        "level" => empty_args(formatter.arg).unwrap_or(Chunk::Level),
                        "m" |
                        "message" => empty_args(formatter.arg).unwrap_or(Chunk::Message),
                        "M" |
                        "module" => empty_args(formatter.arg).unwrap_or(Chunk::Module),
                        "f" |
                        "file" => empty_args(formatter.arg).unwrap_or(Chunk::File),
                        "L" |
                        "line" => empty_args(formatter.arg).unwrap_or(Chunk::Line),
                        "T" |
                        "thread" => empty_args(formatter.arg).unwrap_or(Chunk::Thread),
                        "t" |
                        "target" => empty_args(formatter.arg).unwrap_or(Chunk::Target),
                        name => Chunk::Error(format!("unknown formatter `{}`", name)),
                    }
                }
                Piece::Error(err) => Chunk::Error(err),
            };
            chunks.push(chunk);
        }

        PatternEncoder {
            chunks: chunks,
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
            try!(match *chunk {
                Chunk::Text(ref text) => write!(w, "{}", text),
                Chunk::Time(ref fmt) => write!(w, "{}", UTC::now().format(fmt)),
                Chunk::Level => write!(w, "{}", level),
                Chunk::Message => write!(w, "{}", args),
                Chunk::Module => write!(w, "{}", location.module_path),
                Chunk::File => write!(w, "{}", location.file),
                Chunk::Line => write!(w, "{}", location.line),
                Chunk::Thread => write!(w, "{}", thread::current().name().unwrap_or("<unnamed>")),
                Chunk::Target => write!(w, "{}", target),
                Chunk::Error(ref msg) => write!(w, "{{ERROR: {}}}", msg),
            });
        }
        writeln!(w, "")
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
    use std::io::{self, Write};
    use log::LogLevel;

    use super::{PatternEncoder, Location, Chunk};
    use encode;

    struct SimpleWriter<W>(W);

    impl<W: Write> io::Write for SimpleWriter<W> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }

    impl<W: Write> encode::Write for SimpleWriter<W> {}

    fn error_free(encoder: &PatternEncoder) -> bool {
        encoder.chunks.iter().all(|c| match *c { Chunk::Error(_) => false, _ => true })
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

        static LOCATION: Location<'static> = Location {
            module_path: "mod path",
            file: "the file",
            line: 132,
        };
        let mut buf = SimpleWriter(vec![]);
        pw.append_inner(&mut buf,
                        LogLevel::Debug,
                        "target",
                        &LOCATION,
                        &format_args!("the message"))
          .unwrap();

        assert_eq!(buf.0, &b"DEBUG the message at mod path in the file:132\n"[..]);
    }

    #[test]
    fn unnamed_thread() {
        thread::spawn(|| {
            let pw = PatternEncoder::new("{T}");
            static LOCATION: Location<'static> = Location {
                module_path: "path",
                file: "file",
                line: 132,
            };
            let mut buf = SimpleWriter(vec![]);
            pw.append_inner(&mut buf,
                            LogLevel::Debug,
                            "target",
                            &LOCATION,
                            &format_args!("message"))
              .unwrap();
            assert_eq!(buf.0, b"<unnamed>\n");
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
                static LOCATION: Location<'static> = Location {
                    module_path: "path",
                    file: "file",
                    line: 132,
                };
                let mut buf = SimpleWriter(vec![]);
                pw.append_inner(&mut buf,
                                LogLevel::Debug,
                                "target",
                                &LOCATION,
                                &format_args!("message"))
                  .unwrap();
                assert_eq!(buf.0, b"foobar\n");
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn default_okay() {
        assert!(error_free(&PatternEncoder::default()));
    }
}
