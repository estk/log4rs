//! Simple pattern syntax for logger output formats.
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
//! * `%t` - The name of the thread that the log message came from.
//!

use std::borrow::ToOwned;
use std::default::Default;
use std::thread::Thread;

use log::LogRecord;
use toml::{self, Value};
use time;

#[derive(Show)]
#[cfg_attr(test, derive(PartialEq))]
enum TimeFmt {
    Rfc3339,
    Str(String),
}

#[derive(Show)]
#[cfg_attr(test, derive(PartialEq))]
enum Chunk {
    Text(String),
    Time(TimeFmt),
    Level,
    Message,
    Module,
    File,
    Line,
    Thread,
}

/// A formatter object for `LogRecord`s.
#[derive(Show)]
pub struct PatternLogger {
    pattern: Vec<Chunk>,
}

impl Default for PatternLogger {
    /// Returns a `PatternLogger` using the default pattern of `%d %l %M - %m`.
    fn default() -> PatternLogger {
        PatternLogger::from_pattern("%d %l %M - %m").unwrap()
    }
}

impl PatternLogger {
    /// Creates a `PatternLogger` from the provided configuration table.
    ///
    /// If a `pattern` key is present, the pattern is parsed from it. If no
    /// such key is present, the default pattern is used.
    pub fn from_config(config: &toml::Table) -> Result<PatternLogger, String> {
        match config.get("pattern") {
            Some(&Value::String(ref p)) => PatternLogger::from_pattern(&**p),
            Some(_) => Err("`pattern` must be a string".to_owned()),
            None => Ok(Default::default()),
        }
    }

    /// Creates a `PatternLogger` from a pattern string.
    ///
    /// The pattern string syntax is documented in the `pattern` module.
    pub fn from_pattern(pattern: &str) -> Result<PatternLogger, String> {
        let mut parsed = vec![];
        let mut next_text = String::new();
        let mut it = pattern.chars().peekable();

        while let Some(ch) = it.next() {
            if ch == '%' {
                let chunk = match it.next() {
                    Some('%') => {
                        next_text.push('%');
                        None
                    }
                    Some('d') => {
                        let fmt = match it.peek() {
                            Some(&'{') => {
                                it.next();
                                let mut fmt = String::new();
                                loop {
                                    match it.next() {
                                        Some('}') => break,
                                        Some(c) => fmt.push(c),
                                        None => {
                                            return Err("Unterminated time format".to_owned());
                                        }
                                    }
                                }
                                if let Err(err) = time::now().strftime(&*fmt) {
                                    return Err(err.to_string());
                                }
                                TimeFmt::Str(fmt)
                            }
                            _ => TimeFmt::Rfc3339,
                        };
                        Some(Chunk::Time(fmt))
                    }
                    Some('l') => Some(Chunk::Level),
                    Some('m') => Some(Chunk::Message),
                    Some('M') => Some(Chunk::Module),
                    Some('f') => Some(Chunk::File),
                    Some('L') => Some(Chunk::Line),
                    Some('t') => Some(Chunk::Thread),
                    Some(ch) => return Err(format!("Invalid formatter `%{}`", ch)),
                    None => return Err("Unexpected end of pattern".to_owned()),
                };

                if let Some(chunk) = chunk {
                    if !next_text.is_empty() {
                        parsed.push(Chunk::Text(next_text));
                        next_text = String::new();
                    }
                    parsed.push(chunk);
                }
            } else {
                next_text.push(ch);
            }
        }

        if !next_text.is_empty() {
            parsed.push(Chunk::Text(next_text));
        }

        Ok(PatternLogger {
            pattern: parsed,
        })
    }

    /// Writes the specified `LogRecord` to the specified `Writer` according
    /// to its pattern.
    pub fn log<W>(&self, w: &mut W, record: &LogRecord) where W: Writer {
        for chunk in self.pattern.iter() {
            let _ = match *chunk {
                Chunk::Text(ref text) => write!(w, "{}", text),
                Chunk::Time(TimeFmt::Str(ref fmt)) => {
                    time::now().strftime(&**fmt).map(|time| write!(w, "{}", time))
                        .unwrap_or(Ok(()))
                }
                Chunk::Time(TimeFmt::Rfc3339) => write!(w, "{}", time::now().rfc3339()),
                Chunk::Level => write!(w, "{}", record.level()),
                Chunk::Message => write!(w, "{}", record.args()),
                Chunk::Module => write!(w, "{}", record.location().module_path),
                Chunk::File => write!(w, "{}", record.location().file),
                Chunk::Line => write!(w, "{}", record.location().line),
                Chunk::Thread => {
                    write!(w, "{}", Thread::current().name().unwrap_or("<unnamed thread>"))
                }
            };
        }
        let _ = writeln!(w, "");
    }
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use std::thread::{self, Thread};

    use log::{LogRecord, LogLocation, LogLevel};

    use super::{Chunk, TimeFmt, PatternLogger};

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
                        Chunk::Text("%".to_string())];
        let actual = PatternLogger::from_pattern("hi%d{%Y-%m-%d}%d%l%m%M%f%L%t%%").unwrap().pattern;
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_invalid_date_format() {
        assert!(PatternLogger::from_pattern("%d{%q}").is_err());
    }

    #[test]
    fn test_log() {
        let pw = PatternLogger::from_pattern("%l %m at %M in %f:%L").unwrap();

        static LOCATION: LogLocation = LogLocation {
            module_path: "mod path",
            file: "the file",
            line: 132,
        };
        let mut buf = vec![];
        let args = format_args!("the message");
        let record = LogRecord::new(LogLevel::Debug, &LOCATION, args);
        pw.log(&mut buf, &record);

        assert_eq!(b"DEBUG the message at mod path in the file:132\n", buf);
    }

    #[test]
    fn test_unnamed_thread() {
        Thread::scoped(|| {
            let pw = PatternLogger::from_pattern("%t").unwrap();
            static LOCATION: LogLocation = LogLocation {
                module_path: "path",
                file: "file",
                line: 132,
            };
            let mut buf = vec![];
            let args = format_args!("message");
            let record = LogRecord::new(LogLevel::Debug, &LOCATION, args);
            pw.log(&mut buf, &record);
            assert_eq!(b"<unnamed thread>\n", buf);
        }).join().ok().unwrap();
    }

    #[test]
    fn test_named_thread() {
        thread::Builder::new().name("foobar".to_string()).scoped(|| {
            let pw = PatternLogger::from_pattern("%t").unwrap();
            static LOCATION: LogLocation = LogLocation {
                module_path: "path",
                file: "file",
                line: 132,
            };
            let mut buf = vec![];
            let args = format_args!("message");
            let record = LogRecord::new(LogLevel::Debug, &LOCATION, args);
            pw.log(&mut buf, &record);
            assert_eq!(b"foobar\n", buf);
        }).join().ok().unwrap();
    }

    #[test]
    fn test_default_okay() {
        let _: PatternLogger = Default::default();
    }
}
