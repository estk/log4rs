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
//! # Color Codes
//!
//! When logging to console, color codes can be used to display text in different colors.
//!
//! The basic syntax looks like this:
//! `%c:{color}(...)` , where {color} is one of the following:
//!
//! * `black`
//! * `red`
//! * `green`
//! * `yellow`
//! * `blue`
//! * `magenta`
//! * `cyan`
//! * `white`
//! * `boldBlack`
//! * `boldRed`
//! * `boldGreen`
//! * `boldYellow`
//! * `boldBlue`
//! * `boldMagenta`
//! * `boldCyan`
//! * `boldWhite`
//! * `highlight` - Colors the text depending on the log level
//!
//! **Example**:
//!
//! `%c:green(This text is green) %c:highlight(%d %l %t -) %m`


use std::borrow::ToOwned;
use std::default::Default;
use std::error;
use std::fmt;
use std::thread;
use std::io;
use std::io::Write;

use log::{LogRecord, LogLevel};
use time;
use term;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum TimeFmt {
    Rfc3339,
    Str(String),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum ColorFmt {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BoldBlack,
    BoldRed,
    BoldGreen,
    BoldYellow,
    BoldBlue,
    BoldMagenta,
    BoldCyan,
    BoldWhite,
    Highlight,
    DefaultColor,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum Chunk {
    Text(String),
    Time(TimeFmt),
    Color(ColorFmt),
    Level,
    Message,
    Module,
    File,
    Line,
    Thread,
    Target,
}

/// An error parsing a `PatternLayout` pattern.
#[derive(Debug)]
pub struct Error(String);

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

/// A formatter object for `LogRecord`s.
#[derive(Debug)]
pub struct PatternLayout {
    pattern: Vec<Chunk>,
}

impl Default for PatternLayout {
    /// Returns a `PatternLayout` using the default pattern of `%c:highlight(%d %l %t -) %m.
    fn default() -> PatternLayout {
        PatternLayout::new("%c:highlight(%d %l %t -) %m").unwrap()
    }
}

impl PatternLayout {
    /// Creates a `PatternLayout` from a pattern string.
    ///
    /// The pattern string syntax is documented in the `pattern` module.
    pub fn new(pattern: &str) -> Result<PatternLayout, Error> {
        let mut parsed = vec![];
        let mut next_text = String::new();
        let mut it = pattern.chars().peekable();

        // Indicates if there is a previously opened `(` that has not been closed yet.
        let mut pending_close_colorfmt = false;

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
                                            return Err(Error("Unterminated time format".to_owned()));
                                        }
                                    }
                                }
                                if let Err(err) = time::now().strftime(&*fmt) {
                                    return Err(Error(err.to_string()));
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
                    Some('T') => Some(Chunk::Thread),
                    Some('t') => Some(Chunk::Target),
                    Some('c') => {
                        match it.peek() {
                            Some(&':') => {
                                it.next();
                                let mut color_string = String::new();
                                loop {
                                    match it.next() {
                                        Some('(') => break,
                                        Some(c) => color_string.push(c),
                                        None => {
                                            return Err(Error("Did not find opening bracket for color format".to_owned()));
                                        }
                                    }
                                }

                                let color_fmt = match &*color_string {
                                    "black" => ColorFmt::Black,
                                    "red" => ColorFmt::Red,
                                    "green" => ColorFmt::Green,
                                    "yellow" => ColorFmt::Yellow,
                                    "blue" => ColorFmt::Blue,
                                    "magenta" => ColorFmt::Magenta,
                                    "cyan" => ColorFmt::Cyan,
                                    "white" => ColorFmt::White,
                                    "boldBlack" => ColorFmt::BoldBlack,
                                    "boldRed" => ColorFmt::BoldRed,
                                    "boldGreen" => ColorFmt::BoldGreen,
                                    "boldYellow" => ColorFmt::BoldYellow,
                                    "boldBlue" => ColorFmt::BoldBlue,
                                    "boldMagenta" => ColorFmt::BoldMagenta,
                                    "boldCyan" => ColorFmt::BoldCyan,
                                    "boldWhite" => ColorFmt::BoldWhite,
                                    "highlight" => ColorFmt::Highlight,
                                    _ => return Err(Error(format!("Unrecognized color string `{}`", color_string))),
                                };
                                pending_close_colorfmt = true;
                                Some(Chunk::Color(color_fmt))
                            }
                            _ => return Err(Error(format!("Invalid formatter `%c`"))),
                        }
                    }
                    Some(ch) => return Err(Error(format!("Invalid formatter `%{}`", ch))),
                    None => return Err(Error("Unexpected end of pattern".to_owned())),
                };

                if let Some(chunk) = chunk {
                    if !next_text.is_empty() {
                        parsed.push(Chunk::Text(next_text));
                        next_text = String::new();
                    }
                    parsed.push(chunk);
                }
            } else if ch == ')' {
                if pending_close_colorfmt {
                    if !next_text.is_empty() {
                        parsed.push(Chunk::Text(next_text));
                        next_text = String::new();
                    }
                    parsed.push(Chunk::Color(ColorFmt::DefaultColor));
                    pending_close_colorfmt = false;
                } else {
                    next_text.push(ch);
                }
            } else {
                next_text.push(ch);
            }
        }

        if pending_close_colorfmt {
            return Err(Error("Unexpected end of color format".to_owned()))
        }

        if !next_text.is_empty() {
            parsed.push(Chunk::Text(next_text));
        }

        Ok(PatternLayout {
            pattern: parsed,
        })
    }

    /// Writes the specified `LogRecord` to the specified `Write`r according
    /// to its pattern. This method should *not* be used for console `Write`rs.
    pub fn append<W>(&self, w: &mut W, record: &LogRecord) -> io::Result<()> where W: Write {
        let location = Location {
            module_path: record.location().module_path(),
            file: record.location().file(),
            line: record.location().line(),
        };
        self.append_inner(w, false, record.level(), record.target(), &location, record.args())
    }

    /// Writes the specified `LogRecord` to the specified `Write`r according
    /// to its pattern. This method should be used for console `Write`rs.
    pub fn append_console<W>(&self, w: &mut W, record: &LogRecord) -> io::Result<()> where W: Write {
        let location = Location {
            module_path: record.location().module_path(),
            file: record.location().file(),
            line: record.location().line(),
        };
        self.append_inner(w, true, record.level(), record.target(), &location, record.args())
    }

    fn append_inner<W>(&self,
                       w: &mut W,
                       console: bool,
                       level: LogLevel,
                       target: &str,
                       location: &Location,
                       args: &fmt::Arguments)
                       -> io::Result<()> where W: Write {
        for chunk in self.pattern.iter() {
            try!(match *chunk {
                Chunk::Text(ref text) => write!(w, "{}", text),
                Chunk::Time(TimeFmt::Str(ref fmt)) => {
                    time::now().strftime(&**fmt).map(|time| write!(w, "{}", time))
                        .unwrap_or(Ok(()))
                }
                Chunk::Time(TimeFmt::Rfc3339) => write!(w, "{}", time::now().rfc3339()),
                Chunk::Level => write!(w, "{}", level),
                Chunk::Message => write!(w, "{}", args),
                Chunk::Module => write!(w, "{}", location.module_path),
                Chunk::File => write!(w, "{}", location.file),
                Chunk::Line => write!(w, "{}", location.line),
                Chunk::Thread => {
                    write!(w, "{}", thread::current().name().unwrap_or("<unnamed>"))
                }
                Chunk::Target => write!(w, "{}", target),
                Chunk::Color(ref colorfmt) => {
                    // Only deal with colors when logging to console
                    if !console { continue }
                    let mut w = term::stdout().unwrap();
                    match *colorfmt {
                        ColorFmt::Black => w.fg(term::color::BLACK).map(|_| ()),
                        ColorFmt::Red => w.fg(term::color::RED).map(|_| ()),
                        ColorFmt::Green => w.fg(term::color::GREEN).map(|_| ()),
                        ColorFmt::Yellow => w.fg(term::color::YELLOW).map(|_| ()),
                        ColorFmt::Blue => w.fg(term::color::BLUE).map(|_| ()),
                        ColorFmt::Magenta => w.fg(term::color::MAGENTA).map(|_| ()),
                        ColorFmt::Cyan => w.fg(term::color::CYAN).map(|_| ()),
                        ColorFmt::White => w.fg(term::color::WHITE).map(|_| ()),
                        ColorFmt::BoldBlack => w.fg(term::color::BRIGHT_BLACK).map(|_| ()),
                        ColorFmt::BoldRed => w.fg(term::color::BRIGHT_RED).map(|_| ()),
                        ColorFmt::BoldGreen => w.fg(term::color::BRIGHT_GREEN).map(|_| ()),
                        ColorFmt::BoldYellow => w.fg(term::color::BRIGHT_YELLOW).map(|_| ()),
                        ColorFmt::BoldBlue => w.fg(term::color::BRIGHT_BLUE).map(|_| ()),
                        ColorFmt::BoldMagenta => w.fg(term::color::BRIGHT_MAGENTA).map(|_| ()),
                        ColorFmt::BoldCyan => w.fg(term::color::BRIGHT_CYAN).map(|_| ()),
                        ColorFmt::BoldWhite => w.fg(term::color::BRIGHT_WHITE).map(|_| ()),
                        ColorFmt::DefaultColor => w.reset().map(|_| ()),
                        ColorFmt::Highlight => {
                            match level {
                                LogLevel::Trace => w.fg(term::color::BLUE).map(|_| ()),
                                LogLevel::Debug => w.fg(term::color::CYAN).map(|_| ()),
                                LogLevel::Info => w.fg(term::color::GREEN).map(|_| ()),
                                LogLevel::Warn => w.fg(term::color::YELLOW).map(|_| ()),
                                LogLevel::Error => w.fg(term::color::RED).map(|_| ()),
                            }
                        }
                    }
                }
            });
        }
        // if console {
        //     let mut w = term::stdout().unwrap();
        //     w.reset().unwrap();
        // }
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

    use super::{Chunk, TimeFmt, PatternLayout, Location, ColorFmt};

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
    fn test_parse_with_colors() {
        let expected = [Chunk::Color(ColorFmt::Yellow),
                        Chunk::Text("hi".to_string()),
                        Chunk::Color(ColorFmt::DefaultColor),
                        Chunk::Time(TimeFmt::Str("%Y-%m-%d".to_string())),
                        Chunk::Color(ColorFmt::Highlight),
                        Chunk::Time(TimeFmt::Rfc3339),
                        Chunk::Color(ColorFmt::DefaultColor),
                        Chunk::Text("()".to_string()),
                        Chunk::Level,
                        ];
        let actual = PatternLayout::new("%c:yellow(hi)%d{%Y-%m-%d}%c:highlight(%d)()%l").unwrap().pattern;
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_invalid_date_format() {
        assert!(PatternLayout::new("%d{%q}").is_err());
    }

    #[test]
    fn test_invalid_color_formats() {
        assert!(PatternLayout::new("%c:darkBlack(Is this dark black?)").is_err());
        assert!(PatternLayout::new("%c:green(Oops no closing bracket").is_err());
        assert!(PatternLayout::new("%c(What color is this?)").is_err());
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
                        false,
                        LogLevel::Debug,
                        "target",
                        &LOCATION,
                        &format_args!("the message")).unwrap();

        assert_eq!(buf, &b"DEBUG the message at mod path in the file:132\n"[..]);
    }

    #[test]
    fn test_log_with_colors() {
        let pw = PatternLayout::new("%l %c:red(%m) at %M in %f:%L").unwrap();

        static LOCATION: Location<'static> = Location {
            module_path: "mod path",
            file: "the file",
            line: 132,
        };
        let mut buf = vec![];
        pw.append_inner(&mut buf,
                        true,
                        LogLevel::Debug,
                        "target",
                        &LOCATION,
                        &format_args!("the message")).unwrap();

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
                            false,
                            LogLevel::Debug,
                            "target",
                            &LOCATION,
                            &format_args!("message")).unwrap();
            assert_eq!(buf, b"<unnamed>\n");
        }).join().unwrap();
    }

    #[test]
    fn test_named_thread() {
        thread::Builder::new().name("foobar".to_string()).spawn(|| {
            let pw = PatternLayout::new("%T").unwrap();
            static LOCATION: Location<'static> = Location {
                module_path: "path",
                file: "file",
                line: 132,
            };
            let mut buf = vec![];
            pw.append_inner(&mut buf,
                            false,
                            LogLevel::Debug,
                            "target",
                            &LOCATION,
                            &format_args!("message")).unwrap();
            assert_eq!(buf, b"foobar\n");
        }).unwrap().join().unwrap();
    }

    #[test]
    fn test_default_okay() {
        let _: PatternLayout = Default::default();
    }
}
