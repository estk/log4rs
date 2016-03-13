//! Simple pattern syntax for appender output formats.
//!
//! # Basic Specifiers
//!
//! * `%d` - The current time. By default, the ISO 8601 format is used. A
//!     custom format may be provided in the syntax accepted by `Tm::sprintf`,
//!     enclosed in `{}`s immediately after the specifier: `%d{%Y/%m/%d}`.
//! * `%f` - The source file that the log message came from.
//! * `%l` - The log level.
//! * `%L` - The line that the log message came from.
//! * `%m` - The log message.
//! * `%M` - The module that the log message came from.
//! * `%T` - The name of the thread that the log message came from.
//! * `%t` - The target of the log message.
//!

use std::default::Default;
use std::error;
use std::fmt;
use std::thread;
use std::io;
use std::io::Write;
use std::str;
use encoder::pattern::parser::{TimeFmt, Chunk, parse_pattern};
use nom;
use ErrorInternals;

use log::{LogRecord, LogLevel};
use time;
use encoder::Encode;

mod parser;

/// An error parsing a `PatternEncoder` pattern.
#[derive(Debug)]
pub struct Error(String);

impl ErrorInternals for Error {
    fn new(message: String) -> Error {
        Error(message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.0)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Error parsing a pattern"
    }
}

impl<'a> From<nom::Err<&'a [u8]>> for Error {
    fn from(nom_err: nom::Err<&'a [u8]>) -> Self {
        match nom_err {
            nom::Err::Position(_, token) => {
                Error(format!("Error parsing pattern at token \"{}\"",
                              str::from_utf8(token).unwrap()))
            }
            _ => Error("Could not parse pattern".into()),
        }
    }
}

struct PatternDebug<'a>(&'a [Chunk]);

impl<'a> fmt::Debug for PatternDebug<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(fmt.write_str("\""));
        for chunk in self.0 {
            try!(write!(fmt, "{}", chunk));
        }
        fmt.write_str("\"")
    }
}

/// A formatter object for `LogRecord`s.
pub struct PatternEncoder {
    pattern: Vec<Chunk>,
}

impl fmt::Debug for PatternEncoder {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("PatternEncoder")
           .field("pattern", &PatternDebug(&self.pattern))
           .finish()
    }
}

impl Default for PatternEncoder {
    /// Returns a `PatternEncoder` using the default pattern of `%d %l %t - %m`.
    fn default() -> PatternEncoder {
        PatternEncoder::new("%d %l %t - %m").unwrap()
    }
}

impl PatternEncoder {
    /// Creates a `PatternEncoder` from a pattern string.
    ///
    /// The pattern string syntax is documented in the `pattern` module.
    pub fn new(pattern: &str) -> Result<PatternEncoder, Error> {
        match parse_pattern(pattern.as_bytes()) {
            nom::IResult::Done(_, o) => Ok(PatternEncoder { pattern: o }),
            nom::IResult::Error(nom_err) => Err(Error::from(nom_err)),
            nom::IResult::Incomplete(error) => {
                // This is always a bug in the parser and should actually never happen.
                panic!("Parser returned an incomplete error: {:?}. Please report this bug at \
                        https://github.com/sfackler/log4rs",
                       error)
            }
        }
    }

    fn append_inner(&self,
                    w: &mut Write,
                    level: LogLevel,
                    target: &str,
                    location: &Location,
                    args: &fmt::Arguments)
                    -> io::Result<()> {
        for chunk in &self.pattern {
            try!(match *chunk {
                Chunk::Text(ref text) => write!(w, "{}", text),
                Chunk::Time(TimeFmt::Str(ref fmt)) => {
                    time::now()
                        .strftime(&**fmt)
                        .map(|time| write!(w, "{}", time))
                        .unwrap_or(Ok(()))
                }
                Chunk::Time(TimeFmt::Rfc3339) => write!(w, "{}", time::now().rfc3339()),
                Chunk::Level => write!(w, "{}", level),
                Chunk::Message => write!(w, "{}", args),
                Chunk::Module => write!(w, "{}", location.module_path),
                Chunk::File => write!(w, "{}", location.file),
                Chunk::Line => write!(w, "{}", location.line),
                Chunk::Thread => write!(w, "{}", thread::current().name().unwrap_or("<unnamed>")),
                Chunk::Target => write!(w, "{}", target),
            });
        }
        writeln!(w, "")
    }
}

impl Encode for PatternEncoder {
    fn encode(&mut self, w: &mut Write, record: &LogRecord) -> io::Result<()> {
        let location = Location {
            module_path: record.location().module_path(),
            file: record.location().file(),
            line: record.location().line(),
        };
        self.append_inner(w, record.level(), record.target(), &location, record.args())
    }
}

struct Location<'a> {
    module_path: &'a str,
    file: &'a str,
    line: u32,
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use std::thread;

    use log::LogLevel;

    use super::{PatternEncoder, Location, PatternDebug};
    use encoder::pattern::parser::{TimeFmt, Chunk};

    #[test]
    fn test_parse() {
        let expected = [Chunk::Text("hi".to_string()),
                        Chunk::Time(TimeFmt::Str("%Y-%m-%d".to_string())),
                        Chunk::Time(TimeFmt::Rfc3339),
                        Chunk::Level,
                        Chunk::Message,
                        Chunk::Module,
                        Chunk::File,
                        Chunk::Line,
                        Chunk::Thread,
                        Chunk::Target,
                        Chunk::Text("%".to_string())];
        let raw = "hi%d{%Y-%m-%d}%d%l%m%M%f%L%T%t%%";
        let actual = PatternEncoder::new(raw).unwrap().pattern;
        assert_eq!(actual, expected);
        assert_eq!(format!("{:?}", PatternDebug(&actual)), format!("{:?}", raw));
    }

    #[test]
    fn test_invalid_date_format() {
        assert!(PatternEncoder::new("%d{%q}").is_err());
    }

    #[test]
    fn test_invalid_formatter() {
        assert!(PatternEncoder::new("%x").is_err());
    }

    #[test]
    fn test_unclosed_delimiter() {
        assert!(PatternEncoder::new("%d{%Y-%m-%d").is_err());
    }

    #[test]
    fn test_log() {
        let pw = PatternEncoder::new("%l %m at %M in %f:%L").unwrap();

        static LOCATION: Location<'static> = Location {
            module_path: "mod path",
            file: "the file",
            line: 132,
        };
        let mut buf = vec![];
        pw.append_inner(&mut buf,
                        LogLevel::Debug,
                        "target",
                        &LOCATION,
                        &format_args!("the message"))
          .unwrap();

        assert_eq!(buf, &b"DEBUG the message at mod path in the file:132\n"[..]);
    }

    #[test]
    fn test_unnamed_thread() {
        thread::spawn(|| {
            let pw = PatternEncoder::new("%T").unwrap();
            static LOCATION: Location<'static> = Location {
                module_path: "path",
                file: "file",
                line: 132,
            };
            let mut buf = vec![];
            pw.append_inner(&mut buf,
                            LogLevel::Debug,
                            "target",
                            &LOCATION,
                            &format_args!("message"))
              .unwrap();
            assert_eq!(buf, b"<unnamed>\n");
        })
            .join()
            .unwrap();
    }

    #[test]
    fn test_named_thread() {
        thread::Builder::new()
            .name("foobar".to_string())
            .spawn(|| {
                let pw = PatternEncoder::new("%T").unwrap();
                static LOCATION: Location<'static> = Location {
                    module_path: "path",
                    file: "file",
                    line: 132,
                };
                let mut buf = vec![];
                pw.append_inner(&mut buf,
                                LogLevel::Debug,
                                "target",
                                &LOCATION,
                                &format_args!("message"))
                  .unwrap();
                assert_eq!(buf, b"foobar\n");
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn test_default_okay() {
        let _: PatternEncoder = Default::default();
    }
}
