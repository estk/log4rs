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
use pattern::parser::{TimeFmt, Chunk, parse_pattern};
use nom;
use ErrorInternals;

use log::{LogRecord, LogLevel};
use time;

mod parser;

/// An error parsing a `PatternLayout` pattern.
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

/// A formatter object for `LogRecord`s.
#[derive(Debug)]
pub struct PatternLayout {
    pattern: Vec<Chunk>,
}

impl Default for PatternLayout {
    /// Returns a `PatternLayout` using the default pattern of `%d %l %t - %m`.
    fn default() -> PatternLayout {
        PatternLayout::new("%d %l %t - %m").unwrap()
    }
}

impl PatternLayout {
    /// Creates a `PatternLayout` from a pattern string.
    ///
    /// The pattern string syntax is documented in the `pattern` module.
    pub fn new(pattern: &str) -> Result<PatternLayout, Error> {
        match parse_pattern(pattern.as_bytes()) {
            nom::IResult::Done(_, o) => Ok(PatternLayout { pattern: o }),
            nom::IResult::Error(nom_err) => Err(Error::from(nom_err)),
            nom::IResult::Incomplete(error) => {
                // This is always a bug in the parser and should actually never happen.
                panic!("Parser returned an incomplete error: {:?}. Please report this bug at \
                        https://github.com/sfackler/log4rs",
                       error)
            }
        }
    }

    /// Writes the specified `LogRecord` to the specified `Write`r according
    /// to its pattern.
    pub fn append<W>(&self, w: &mut W, record: &LogRecord) -> io::Result<()>
        where W: Write
    {
        let location = Location {
            module_path: record.location().module_path(),
            file: record.location().file(),
            line: record.location().line(),
        };
        self.append_inner(w, record.level(), record.target(), &location, record.args())
    }

    fn append_inner<W>(&self,
                       w: &mut W,
                       level: LogLevel,
                       target: &str,
                       location: &Location,
                       args: &fmt::Arguments)
                       -> io::Result<()>
        where W: Write
    {
        for chunk in self.pattern.iter() {
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

    use super::{PatternLayout, Location};
    use pattern::parser::{TimeFmt, Chunk};

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
        let actual = PatternLayout::new("hi%d{%Y-%m-%d}%d%l%m%M%f%L%T%t%%").unwrap().pattern;
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_invalid_date_format() {
        assert!(PatternLayout::new("%d{%q}").is_err());
    }

    #[test]
    fn test_invalid_formatter() {
        assert!(PatternLayout::new("%x").is_err());
    }

    #[test]
    fn test_unclosed_delimiter() {
        assert!(PatternLayout::new("%d{%Y-%m-%d").is_err());
    }

    #[test]
    fn test_log() {
        let pw = PatternLayout::new("%l %m at %M in %f:%L").unwrap();

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
            let pw = PatternLayout::new("%T").unwrap();
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
                let pw = PatternLayout::new("%T").unwrap();
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
        let _: PatternLayout = Default::default();
    }
}
