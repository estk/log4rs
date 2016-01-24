//! There are two encoders available: pattern and JSON.
//!
//! # Basic pattern syntax specifiers
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
//! Supported JSON fields
//! * message
//! * level
//! * source
//! * thread
//! * module
//! * line
//! * target
//! * timestamp

extern crate serde_json;

use std::default::Default;
use std::error;
use std::fmt;
use std::thread;
use std::io;
use std::io::Write;
use std::str;
use std::collections::BTreeMap;
use parser::{TimeFmt, Chunk, parse_pattern};
use nom;
use ErrorInternals;

use log::{LogRecord, LogLevel};
use time;

/// An error parsing a `Encoder` pattern.
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
pub enum Encoder {
    PatternLayout{pattern: Vec<Chunk>},
    Json{fields: Vec<JsonField>}
}

/// JSON field for Json encoder settings
#[derive(Debug)]
pub enum JsonField {
    /// The log message.
    Message,
    /// The name of the thread that the log message came from.
    Thread,
    /// The current time.
    Timestamp,
    /// The source file that the log message came from.
    Source,
    /// The log level.
    Level,
    /// The module that the log message came from.
    Module,
    /// The target of the log message.
    Target,
    /// The line that the log message came from.
    Line
}

fn parse_field(field: &str) -> Result<JsonField, Error> {
    match field {
        "message" => Ok(JsonField::Message),
        "level" => Ok(JsonField::Level),
        "source" => Ok(JsonField::Source),
        "thread" => Ok(JsonField::Thread),
        "module" => Ok(JsonField::Module),
        "line" => Ok(JsonField::Line),
        "target" => Ok(JsonField::Target),
        "timestamp" => Ok(JsonField::Timestamp),
        _ => Err(Error(format!("Unknown json field {:?}", field))),
    }
}


impl Default for Encoder {
    /// Returns a `Encoder` using the default pattern of `%d %l %t - %m`.
    fn default() -> Encoder {
        Encoder::pattern("%d %l %t - %m").unwrap()
    }
}

impl Encoder {
    /// Creates pattern `Encoder` from a pattern string.
    ///
    /// The pattern string syntax is documented in the `encoder` module.
    pub fn pattern(pattern: &str) -> Result<Encoder, Error> {
        match parse_pattern(pattern.as_bytes()) {
            nom::IResult::Done(_, o) => Ok(Encoder::PatternLayout { pattern: o }),
            nom::IResult::Error(nom_err) => Err(Error::from(nom_err)),
            nom::IResult::Incomplete(error) => {
                // This is always a bug in the parser and should actually never happen.
                panic!("Parser returned an incomplete error: {:?}. Please report this bug at \
                        https://github.com/sfackler/log4rs",
                       error)
            }
        }
    }

    /// Creates a JSON `Encoder` from a fields string iterator.
    ///
    /// The field names are documented in the `encoder` module.
    pub fn json<'a,I>(names: I) -> Result<Encoder, Error> where I: Iterator<Item=&'a str> {
        let mut fields = Vec::new();
        for name in names {
            fields.push(try!(parse_field(name)));
        }
        Ok(Encoder::Json{fields: fields})
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
        match self {
            &Encoder::PatternLayout{ ref pattern } =>
                for chunk in pattern.iter() {
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
                },
            &Encoder::Json{ref fields} => {
                let mut json = BTreeMap::new();
                for field in fields.iter() {
                    match *field {
                        JsonField::Message => {
                            json.insert("message".to_string(), serde_json::value::Value::String(format!("{}", args)));
                        }
                        JsonField::Level => {
                            json.insert("level".to_string(), serde_json::value::Value::String(format!("{}", level)));
                        }
                        JsonField::Line => {
                            json.insert("line".to_string(), serde_json::value::Value::String(format!("{}", location.line)));
                        }
                        JsonField::Module => {
                            json.insert("module".to_string(), serde_json::value::Value::String(format!("{}", location.module_path)));
                        }
                        JsonField::Source => {
                            json.insert("source".to_string(), serde_json::value::Value::String(format!("{}", location.file)));
                        }
                        JsonField::Target => {
                            json.insert("target".to_string(), serde_json::value::Value::String(format!("{}", target)));
                        }
                        JsonField::Timestamp => {
                            json.insert("timestamp".to_string(), serde_json::value::Value::String(format!("{}", time::now().rfc3339())));
                        }
                        JsonField::Thread => {
                            json.insert("thread".to_string(), serde_json::value::Value::String(format!("{}", thread::current().name().unwrap_or("<unnamed>"))));
                        }
                    }
                }
                let object = serde_json::value::Value::Object(json);
                serde_json::ser::to_writer(w, &object);
            }
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

    use super::{Encoder, Location};
    use parser::{TimeFmt, Chunk};

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
        match Encoder::pattern("hi%d{%Y-%m-%d}%d%l%m%M%f%L%T%t%%").unwrap() {
            Encoder::PatternLayout{pattern} =>
                assert_eq!(pattern, expected),
            _ =>
                assert!(false)
        }
    }

    #[test]
    fn test_invalid_date_format() {
        assert!(Encoder::pattern("%d{%q}").is_err());
    }

    #[test]
    fn test_invalid_formatter() {
        assert!(Encoder::pattern("%x").is_err());
    }

    #[test]
    fn test_unclosed_delimiter() {
        assert!(Encoder::pattern("%d{%Y-%m-%d").is_err());
    }

    #[test]
    fn test_log() {
        let pw = Encoder::pattern("%l %m at %M in %f:%L").unwrap();

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
            let pw = Encoder::pattern("%T").unwrap();
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
                let pw = Encoder::pattern("%T").unwrap();
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
        let _: Encoder = Default::default();
    }
}
