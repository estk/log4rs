//! A simple pattern-based encoder.
//!
//! Requires the `pattern_encoder` feature.
//!
//! The pattern syntax is similar to Rust's string formatting syntax. It
//! consists of raw text interspersed with format arguments. The grammar is:
//!
//! ```not_rust
//! format_string := <text> [ format <text> ] *
//! format := '{' formatter [ ':' format_spec ] '}'
//! formatter := [ name ] [ '(' argument ')' ] *
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
//! # Special characters
//!
//! The `{`, `}`, `(`, `)`, and `\` characters are part of the pattern syntax;
//! they must be escaped to appear in output. Like with Rust's string
//! formatting syntax, type the character twice to escape it. That is, `{{`
//! will be rendered as `{` in output and `))` will be rendered as `)`.
//!
//! In addition, these characters may also be escaped by prefixing them with a
//! `\` character. That is, `\{` will be rendered as `{`.
//!
//! # Formatters
//!
//! A formatter inserts a dynamic portion of text into the pattern. It may be
//! text derived from a log event or from some other context like the current
//! time. Formatters may be passed arguments consisting of parenthesized format
//! strings.
//!
//! The following formatters are currently supported. Unless otherwise stated,
//! a formatter does not accept any argument.
//!
//! * `d`, `date` - The current time. By default, the ISO 8601 format is used.
//!     A custom format may be provided in the syntax accepted by `chrono`.
//!     The timezone defaults to local, but can be specified explicitly by
//!     passing a second argument of `utc` for UTC or `local` for local time.
//!     * `{d}` - `2016-03-20T14:22:20.644420340-08:00`
//!     * `{d(%Y-%m-%d %H:%M:%S)}` - `2016-03-20 14:22:20`
//!     * `{d(%Y-%m-%d %H:%M:%S %Z)(utc)}` - `2016-03-20 22:22:20 UTC`
//! * `f`, `file` - The source file that the log message came from, or `???` if
//!     not provided.
//! * `h`, `highlight` - Styles its argument according to the log level. The
//!     style is intense red for errors, red for warnings, blue for info, and
//!     the default style for all other levels.
//!     * `{h(the level is {l})}` -
//!         <code style="color: red; font-weight: bold">the level is ERROR</code>
//! * `D`, `debug` - Outputs its arguments ONLY in debug build.
//! * `R`, `release` - Outputs its arguments ONLY in release build.
//! * `l`, `level` - The log level.
//! * `L`, `line` - The line that the log message came from, or `???` if not
//!     provided.
//! * `m`, `message` - The log message.
//! * `M`, `module` - The module that the log message came from, or `???` if not
//!     provided.
//! * `P`, `pid` - The current process id.
//! * `i`, `tid` - The current system-wide unique thread ID.
//! * `n` - A platform-specific newline.
//! * `t`, `target` - The target of the log message.
//! * `T`, `thread` - The name of the current thread.
//! * `I`, `thread_id` - The pthread ID of the current thread.
//! * `X`, `mdc` - A value from the [MDC][MDC]. The first argument specifies
//!     the key, and the second argument specifies the default value if the
//!     key is not present in the MDC. The second argument is optional, and
//!     defaults to the empty string.
//!     * `{X(user_id)}` - `123e4567-e89b-12d3-a456-426655440000`
//!     * `{X(nonexistent_key)(no mapping)}` - `no mapping`
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
//!
//! [MDC]: https://crates.io/crates/log-mdc

use chrono::{Local, Utc};
use derivative::Derivative;
use log::{Level, Record};
use std::{default::Default, io, process, thread};

use crate::encode::{
    self,
    pattern::parser::{Alignment, Parameters, Parser, Piece},
    Color, Encode, Style, NEWLINE,
};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

mod parser;

thread_local!(
    /// Thread-locally cached thread ID.
    static TID: usize = thread_id::get()
);

/// The pattern encoder's configuration.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatternEncoderConfig {
    pattern: Option<String>,
}

fn is_char_boundary(b: u8) -> bool {
    b as i8 >= -0x40
}

fn char_starts(buf: &[u8]) -> usize {
    buf.iter().filter(|&&b| is_char_boundary(b)).count()
}

struct MaxWidthWriter<'a> {
    remaining: usize,
    w: &'a mut dyn encode::Write,
}

impl<'a> io::Write for MaxWidthWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut remaining = self.remaining;
        let mut end = buf.len();
        for (idx, _) in buf
            .iter()
            .enumerate()
            .filter(|&(_, &b)| is_char_boundary(b))
        {
            if remaining == 0 {
                end = idx;
                break;
            }
            remaining -= 1;
        }

        // we don't want to report EOF, so just act as a sink past this point
        if end == 0 {
            return Ok(buf.len());
        }

        let buf = &buf[..end];
        match self.w.write(buf) {
            Ok(len) => {
                if len == end {
                    self.remaining = remaining;
                } else {
                    self.remaining -= char_starts(&buf[..len]);
                }
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

struct LeftAlignWriter<W> {
    to_fill: usize,
    fill: char,
    w: W,
}

impl<W: encode::Write> LeftAlignWriter<W> {
    fn finish(mut self) -> io::Result<()> {
        for _ in 0..self.to_fill {
            write!(self.w, "{}", self.fill)?;
        }
        Ok(())
    }
}

impl<W: encode::Write> io::Write for LeftAlignWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.w.write(buf) {
            Ok(len) => {
                self.to_fill = self.to_fill.saturating_sub(char_starts(&buf[..len]));
                Ok(len)
            }
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

impl<W: encode::Write> encode::Write for LeftAlignWriter<W> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.w.set_style(style)
    }
}

enum BufferedOutput {
    Data(Vec<u8>),
    Style(Style),
}

struct RightAlignWriter<W> {
    to_fill: usize,
    fill: char,
    w: W,
    buf: Vec<BufferedOutput>,
}

impl<W: encode::Write> RightAlignWriter<W> {
    fn finish(mut self) -> io::Result<()> {
        for _ in 0..self.to_fill {
            write!(self.w, "{}", self.fill)?;
        }
        for out in self.buf {
            match out {
                BufferedOutput::Data(ref buf) => self.w.write_all(buf)?,
                BufferedOutput::Style(ref style) => self.w.set_style(style)?,
            }
        }
        Ok(())
    }
}

impl<W: encode::Write> io::Write for RightAlignWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.to_fill = self.to_fill.saturating_sub(char_starts(buf));

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

impl<W: encode::Write> encode::Write for RightAlignWriter<W> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.buf.push(BufferedOutput::Style(style.clone()));
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum Chunk {
    Text(String),
    Formatted {
        chunk: FormattedChunk,
        params: Parameters,
    },
    Error(String),
}

impl Chunk {
    fn encode(&self, w: &mut dyn encode::Write, record: &Record) -> io::Result<()> {
        match *self {
            Chunk::Text(ref s) => w.write_all(s.as_bytes()),
            Chunk::Formatted {
                ref chunk,
                ref params,
            } => match (params.min_width, params.max_width, params.align) {
                (None, None, _) => chunk.encode(w, record),
                (None, Some(max_width), _) => {
                    let mut w = MaxWidthWriter {
                        remaining: max_width,
                        w,
                    };
                    chunk.encode(&mut w, record)
                }
                (Some(min_width), None, Alignment::Left) => {
                    let mut w = LeftAlignWriter {
                        to_fill: min_width,
                        fill: params.fill,
                        w,
                    };
                    chunk.encode(&mut w, record)?;
                    w.finish()
                }
                (Some(min_width), None, Alignment::Right) => {
                    let mut w = RightAlignWriter {
                        to_fill: min_width,
                        fill: params.fill,
                        w,
                        buf: vec![],
                    };
                    chunk.encode(&mut w, record)?;
                    w.finish()
                }
                (Some(min_width), Some(max_width), Alignment::Left) => {
                    let mut w = LeftAlignWriter {
                        to_fill: min_width,
                        fill: params.fill,
                        w: MaxWidthWriter {
                            remaining: max_width,
                            w,
                        },
                    };
                    chunk.encode(&mut w, record)?;
                    w.finish()
                }
                (Some(min_width), Some(max_width), Alignment::Right) => {
                    let mut w = RightAlignWriter {
                        to_fill: min_width,
                        fill: params.fill,
                        w: MaxWidthWriter {
                            remaining: max_width,
                            w,
                        },
                        buf: vec![],
                    };
                    chunk.encode(&mut w, record)?;
                    w.finish()
                }
            },
            Chunk::Error(ref s) => write!(w, "{{ERROR: {}}}", s),
        }
    }
}

impl<'a> From<Piece<'a>> for Chunk {
    fn from(piece: Piece<'a>) -> Chunk {
        match piece {
            Piece::Text(text) => Chunk::Text(text.to_owned()),
            Piece::Argument {
                mut formatter,
                parameters,
            } => match formatter.name {
                "d" | "date" => {
                    if formatter.args.len() > 2 {
                        return Chunk::Error("expected at most two arguments".to_owned());
                    }

                    let format = match formatter.args.first() {
                        Some(arg) => {
                            let mut format = String::new();
                            for piece in arg {
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
                            format
                        }
                        None => "%+".to_owned(),
                    };

                    let timezone = match formatter.args.get(1) {
                        Some(arg) => {
                            if let Some(arg) = arg.first() {
                                match *arg {
                                    Piece::Text("utc") => Timezone::Utc,
                                    Piece::Text("local") => Timezone::Local,
                                    Piece::Text(z) => {
                                        return Chunk::Error(format!("invalid timezone `{}`", z));
                                    }
                                    _ => return Chunk::Error("invalid timezone".to_owned()),
                                }
                            } else {
                                return Chunk::Error("invalid timezone".to_owned());
                            }
                        }
                        None => Timezone::Local,
                    };

                    Chunk::Formatted {
                        chunk: FormattedChunk::Time(format, timezone),
                        params: parameters,
                    }
                }
                "h" | "highlight" => {
                    if formatter.args.len() != 1 {
                        return Chunk::Error("expected exactly one argument".to_owned());
                    }

                    let chunks = formatter
                        .args
                        .pop()
                        .unwrap()
                        .into_iter()
                        .map(From::from)
                        .collect();
                    Chunk::Formatted {
                        chunk: FormattedChunk::Highlight(chunks),
                        params: parameters,
                    }
                }
                "D" | "debug" => {
                    if formatter.args.len() != 1 {
                        return Chunk::Error("expected exactly one argument".to_owned());
                    }

                    let chunks = formatter
                        .args
                        .pop()
                        .unwrap()
                        .into_iter()
                        .map(From::from)
                        .collect();
                    Chunk::Formatted {
                        chunk: FormattedChunk::Debug(chunks),
                        params: parameters,
                    }
                }
                "R" | "release" => {
                    if formatter.args.len() != 1 {
                        return Chunk::Error("expected exactly one argument".to_owned());
                    }

                    let chunks = formatter
                        .args
                        .pop()
                        .unwrap()
                        .into_iter()
                        .map(From::from)
                        .collect();
                    Chunk::Formatted {
                        chunk: FormattedChunk::Release(chunks),
                        params: parameters,
                    }
                }
                "l" | "level" => no_args(&formatter.args, parameters, FormattedChunk::Level),
                "m" | "message" => no_args(&formatter.args, parameters, FormattedChunk::Message),
                "M" | "module" => no_args(&formatter.args, parameters, FormattedChunk::Module),
                "n" => no_args(&formatter.args, parameters, FormattedChunk::Newline),
                "f" | "file" => no_args(&formatter.args, parameters, FormattedChunk::File),
                "L" | "line" => no_args(&formatter.args, parameters, FormattedChunk::Line),
                "T" | "thread" => no_args(&formatter.args, parameters, FormattedChunk::Thread),
                "I" | "thread_id" => no_args(&formatter.args, parameters, FormattedChunk::ThreadId),
                "P" | "pid" => no_args(&formatter.args, parameters, FormattedChunk::ProcessId),
                "i" | "tid" => no_args(&formatter.args, parameters, FormattedChunk::SystemThreadId),
                "t" | "target" => no_args(&formatter.args, parameters, FormattedChunk::Target),
                "X" | "mdc" => {
                    if formatter.args.len() > 2 {
                        return Chunk::Error("expected at most two arguments".to_owned());
                    }

                    let key = match formatter.args.first() {
                        Some(arg) => {
                            if let Some(arg) = arg.first() {
                                match arg {
                                    Piece::Text(key) => key.to_owned(),
                                    Piece::Error(ref e) => return Chunk::Error(e.clone()),
                                    _ => return Chunk::Error("invalid MDC key".to_owned()),
                                }
                            } else {
                                return Chunk::Error("invalid MDC key".to_owned());
                            }
                        }
                        None => return Chunk::Error("missing MDC key".to_owned()),
                    };

                    let default = match formatter.args.get(1) {
                        Some(arg) => {
                            if let Some(arg) = arg.first() {
                                match arg {
                                    Piece::Text(key) => key.to_owned(),
                                    Piece::Error(ref e) => return Chunk::Error(e.clone()),
                                    _ => return Chunk::Error("invalid MDC default".to_owned()),
                                }
                            } else {
                                return Chunk::Error("invalid MDC default".to_owned());
                            }
                        }
                        None => "",
                    };

                    Chunk::Formatted {
                        chunk: FormattedChunk::Mdc(key.into(), default.into()),
                        params: parameters,
                    }
                }
                "" => {
                    if formatter.args.len() != 1 {
                        return Chunk::Error("expected exactly one argument".to_owned());
                    }

                    let chunks = formatter
                        .args
                        .pop()
                        .unwrap()
                        .into_iter()
                        .map(From::from)
                        .collect();
                    Chunk::Formatted {
                        chunk: FormattedChunk::Align(chunks),
                        params: parameters,
                    }
                }
                name => Chunk::Error(format!("unknown formatter `{}`", name)),
            },
            Piece::Error(err) => Chunk::Error(err),
        }
    }
}

fn no_args(arg: &[Vec<Piece>], params: Parameters, chunk: FormattedChunk) -> Chunk {
    if arg.is_empty() {
        Chunk::Formatted { chunk, params }
    } else {
        Chunk::Error("unexpected arguments".to_owned())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum Timezone {
    Utc,
    Local,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum FormattedChunk {
    Time(String, Timezone),
    Level,
    Message,
    Module,
    File,
    Line,
    Thread,
    ThreadId,
    ProcessId,
    SystemThreadId,
    Target,
    Newline,
    Align(Vec<Chunk>),
    Highlight(Vec<Chunk>),
    Debug(Vec<Chunk>),
    Release(Vec<Chunk>),
    Mdc(String, String),
}

impl FormattedChunk {
    fn encode(&self, w: &mut dyn encode::Write, record: &Record) -> io::Result<()> {
        match *self {
            FormattedChunk::Time(ref fmt, Timezone::Utc) => write!(w, "{}", Utc::now().format(fmt)),
            FormattedChunk::Time(ref fmt, Timezone::Local) => {
                write!(w, "{}", Local::now().format(fmt))
            }
            FormattedChunk::Level => write!(w, "{}", record.level()),
            FormattedChunk::Message => w.write_fmt(*record.args()),
            FormattedChunk::Module => w.write_all(record.module_path().unwrap_or("???").as_bytes()),
            FormattedChunk::File => w.write_all(record.file().unwrap_or("???").as_bytes()),
            FormattedChunk::Line => match record.line() {
                Some(line) => write!(w, "{}", line),
                None => w.write_all(b"???"),
            },
            FormattedChunk::Thread => {
                w.write_all(thread::current().name().unwrap_or("unnamed").as_bytes())
            }
            FormattedChunk::ThreadId => w.write_all(thread_id::get().to_string().as_bytes()),
            FormattedChunk::ProcessId => w.write_all(process::id().to_string().as_bytes()),
            FormattedChunk::SystemThreadId => {
                TID.with(|tid| w.write_all(tid.to_string().as_bytes()))
            }
            FormattedChunk::Target => w.write_all(record.target().as_bytes()),
            FormattedChunk::Newline => w.write_all(NEWLINE.as_bytes()),
            FormattedChunk::Align(ref chunks) => {
                for chunk in chunks {
                    chunk.encode(w, record)?;
                }
                Ok(())
            }
            FormattedChunk::Highlight(ref chunks) => {
                match record.level() {
                    Level::Error => {
                        w.set_style(Style::new().text(Color::Red).intense(true))?;
                    }
                    Level::Warn => w.set_style(Style::new().text(Color::Yellow))?,
                    Level::Info => w.set_style(Style::new().text(Color::Green))?,
                    Level::Trace => w.set_style(Style::new().text(Color::Cyan))?,
                    _ => {}
                }
                for chunk in chunks {
                    chunk.encode(w, record)?;
                }
                match record.level() {
                    Level::Error | Level::Warn | Level::Info | Level::Trace => {
                        w.set_style(&Style::new())?
                    }
                    _ => {}
                }
                Ok(())
            }
            FormattedChunk::Debug(ref chunks) => {
                if cfg!(debug_assertions) {
                    for chunk in chunks {
                        chunk.encode(w, record)?;
                    }
                }
                Ok(())
            }
            FormattedChunk::Release(ref chunks) => {
                if !cfg!(debug_assertions) {
                    for chunk in chunks {
                        chunk.encode(w, record)?;
                    }
                }
                Ok(())
            }
            FormattedChunk::Mdc(ref key, ref default) => {
                log_mdc::get(key, |v| write!(w, "{}", v.unwrap_or(default)))
            }
        }
    }
}

/// An `Encode`r configured via a format string.
#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct PatternEncoder {
    #[derivative(Debug = "ignore")]
    chunks: Vec<Chunk>,
    pattern: String,
}

/// Returns a `PatternEncoder` using the default pattern of `{d} {l} {t} - {m}{n}`.
impl Default for PatternEncoder {
    fn default() -> PatternEncoder {
        PatternEncoder::new("{d} {l} {t} - {m}{n}")
    }
}

impl Encode for PatternEncoder {
    fn encode(&self, w: &mut dyn encode::Write, record: &Record) -> anyhow::Result<()> {
        for chunk in &self.chunks {
            chunk.encode(w, record)?;
        }
        Ok(())
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
}

/// A deserializer for the `PatternEncoder`.
///
/// # Configuration
///
/// ```yaml
/// kind: pattern
///
/// # The pattern to follow when formatting logs. Defaults to
/// # "{d} {l} {t} - {m}{n}".
/// pattern: "{d} {l} {t} - {m}{n}"
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PatternEncoderDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for PatternEncoderDeserializer {
    type Trait = dyn Encode;

    type Config = PatternEncoderConfig;

    fn deserialize(
        &self,
        config: PatternEncoderConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Encode>> {
        let encoder = match config.pattern {
            Some(pattern) => PatternEncoder::new(&pattern),
            None => PatternEncoder::default(),
        };
        Ok(Box::new(encoder))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "config_parsing")]
    use crate::config::Deserializers;
    #[cfg(feature = "simple_writer")]
    use crate::encode::{writer::simple::SimpleWriter, Encode, Write as EncodeWrite};
    #[cfg(feature = "simple_writer")]
    use log::{Level, Record};
    #[cfg(feature = "simple_writer")]
    use std::{io::Write, process, thread};

    use super::*;

    fn error_free(encoder: &PatternEncoder) -> bool {
        encoder.chunks.iter().all(|c| match *c {
            Chunk::Error(_) => false,
            _ => true,
        })
    }

    #[test]
    fn test_invalid_formatter() {
        assert!(!error_free(&PatternEncoder::new("{x}")));
    }

    #[test]
    fn test_unclosed_delimiter() {
        assert!(!error_free(&PatternEncoder::new("{d(%Y-%m-%d)")));
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_log() {
        let pw = PatternEncoder::new("{l} {m} at {M} in {f}:{L}");
        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder()
                .level(Level::Debug)
                .args(format_args!("the message"))
                .module_path(Some("path"))
                .file(Some("file"))
                .line(Some(132))
                .build(),
        )
        .unwrap();

        assert_eq!(buf, &b"DEBUG the message at path in file:132"[..]);
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_unnamed_thread() {
        thread::spawn(|| {
            let pw = PatternEncoder::new("{T}");
            let mut buf = vec![];
            pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
                .unwrap();
            assert_eq!(buf, b"unnamed");
        })
        .join()
        .unwrap();
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_named_thread() {
        thread::Builder::new()
            .name("foobar".to_string())
            .spawn(|| {
                let pw = PatternEncoder::new("{T}");
                let mut buf = vec![];
                pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
                    .unwrap();
                assert_eq!(buf, b"foobar");
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_thread_id_field() {
        thread::spawn(|| {
            let pw = PatternEncoder::new("{I}");
            let mut buf = vec![];
            pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
                .unwrap();
            assert_eq!(buf, thread_id::get().to_string().as_bytes());
        })
        .join()
        .unwrap();
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_process_id() {
        let pw = PatternEncoder::new("{P}");
        let mut buf = vec![];

        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, process::id().to_string().as_bytes());
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_system_thread_id() {
        let pw = PatternEncoder::new("{i}");
        let mut buf = vec![];

        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, thread_id::get().to_string().as_bytes());
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_default_okay() {
        assert!(error_free(&PatternEncoder::default()));
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_left_align() {
        let pw = PatternEncoder::new("{m:~<5.6}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foo")).build(),
        )
        .unwrap();
        assert_eq!(buf, b"foo~~");

        buf.clear();
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foobar!")).build(),
        )
        .unwrap();
        assert_eq!(buf, b"foobar");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_right_align() {
        let pw = PatternEncoder::new("{m:~>5.6}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foo")).build(),
        )
        .unwrap();
        assert_eq!(buf, b"~~foo");

        buf.clear();
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foobar!")).build(),
        )
        .unwrap();
        assert_eq!(buf, b"foobar");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_left_align_formatter() {
        let pw = PatternEncoder::new("{({l} {m}):15}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder()
                .level(Level::Info)
                .args(format_args!("foobar!"))
                .build(),
        )
        .unwrap();
        assert_eq!(buf, b"INFO foobar!   ");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_right_align_formatter() {
        let pw = PatternEncoder::new("{({l} {m}):>15}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder()
                .level(Level::Info)
                .args(format_args!("foobar!"))
                .build(),
        )
        .unwrap();
        assert_eq!(buf, b"   INFO foobar!");
    }

    #[test]
    fn test_custom_date_format() {
        assert!(error_free(&PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {m}{n}"
        )));
    }

    #[test]
    fn test_timezones() {
        assert!(error_free(&PatternEncoder::new("{d(%+)(utc)}")));
        assert!(error_free(&PatternEncoder::new("{d(%+)(local)}")));
        assert!(!error_free(&PatternEncoder::new("{d(%+)(foo)}")));
    }

    #[test]
    fn test_unescaped_parens() {
        assert!(!error_free(&PatternEncoder::new("(hi)")));
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_escaped_chars() {
        let pw = PatternEncoder::new("{{{m}(())}}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foobar!")).build(),
        )
        .unwrap();
        assert_eq!(buf, b"{foobar!()}");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_quote_braces_with_backslash() {
        let pw = PatternEncoder::new(r"\{\({l}\)\}\\");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().level(Level::Info).build(),
        )
        .unwrap();
        assert_eq!(buf, br"{(INFO)}\");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_mdc() {
        let pw = PatternEncoder::new("{X(user_id)}");
        log_mdc::insert("user_id", "mdc value");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"mdc value");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_mdc_missing_default() {
        let pw = PatternEncoder::new("{X(user_id)}");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_mdc_missing_custom() {
        let pw = PatternEncoder::new("{X(user_id)(missing value)}");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"missing value");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_debug_release() {
        let debug_pat = "{D({l})}";
        let release_pat = "{R({l})}";

        let debug_encoder = PatternEncoder::new(debug_pat);
        let release_encoder = PatternEncoder::new(release_pat);
        let mut debug_buf = vec![];
        let mut release_buf = vec![];

        debug_encoder
            .encode(
                &mut SimpleWriter(&mut debug_buf),
                &Record::builder().level(Level::Info).build(),
            )
            .unwrap();
        release_encoder
            .encode(
                &mut SimpleWriter(&mut release_buf),
                &Record::builder().level(Level::Info).build(),
            )
            .unwrap();

        if cfg!(debug_assertions) {
            assert_eq!(debug_buf, b"INFO");
            assert!(release_buf.is_empty());
        } else {
            assert_eq!(release_buf, b"INFO");
            assert!(debug_buf.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_max_width_writer() {
        let mut buf = vec![];
        let mut w = SimpleWriter(&mut buf);

        let mut w = MaxWidthWriter {
            remaining: 2,
            w: &mut w,
        };

        let res = w.write(b"test write");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 2);
        assert_eq!(w.remaining, 0);
        assert!(w.flush().is_ok());
        assert!(w.set_style(&Style::new()).is_ok());
        assert_eq!(buf, b"te");

        let mut buf = vec![];
        let mut w = SimpleWriter(&mut buf);

        let mut w = MaxWidthWriter {
            remaining: 15,
            w: &mut w,
        };
        let res = w.write(b"test write");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 10);
        assert_eq!(w.remaining, 5);
        assert_eq!(buf, b"test write");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_left_align_writer() {
        let mut buf = vec![];
        let mut w = SimpleWriter(&mut buf);

        let mut w = LeftAlignWriter {
            to_fill: 4,
            fill: ' ',
            w: &mut w,
        };

        let res = w.write(b"test write");
        assert!(res.is_ok());
        assert!(w.flush().is_ok());
        assert!(w.set_style(&Style::new()).is_ok());
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_right_align_writer() {
        let mut write_buf = vec![];
        let buf = vec![BufferedOutput::Style(Style::new())];
        let mut w = SimpleWriter(&mut write_buf);

        let mut w = RightAlignWriter {
            to_fill: 4,
            fill: ' ',
            w: &mut w,
            buf,
        };

        let res = w.write(b"test write");
        assert!(res.is_ok());
        assert!(w.flush().is_ok());
        assert!(w.set_style(&Style::new()).is_ok());
        assert!(w.finish().is_ok());
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_cfg_deserializer() {
        let pattern_cfg = PatternEncoderConfig {
            pattern: Some("[{d(%Y-%m-%dT%H:%M:%S%.6f)} {h({l}):<5.5} {M}] {m}{n}".to_owned()),
        };

        let deserializer = PatternEncoderDeserializer;

        let res = deserializer.deserialize(pattern_cfg, &Deserializers::default());
        assert!(res.is_ok());

        let pattern_cfg = PatternEncoderConfig { pattern: None };

        let res = deserializer.deserialize(pattern_cfg, &Deserializers::default());
        assert!(res.is_ok());
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_chunk_no_min_width() {
        let mut buf = vec![];
        let pattern = "[{h({l}):<.5} {M}]";
        let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
        for chunk in chunks {
            assert!(chunk
                .encode(
                    &mut SimpleWriter(&mut buf),
                    &Record::builder()
                        .level(Level::Debug)
                        .args(format_args!("the message"))
                        .module_path(Some("path"))
                        .file(Some("file"))
                        .line(Some(132))
                        .build()
                )
                .is_ok())
        }
        assert!(!String::from_utf8(buf).unwrap().contains("ERROR"));
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_chunk_encode_err() {
        let mut buf = vec![];
        let pattern = "[{h({l):<.5}]";
        let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
        for chunk in chunks {
            assert!(chunk
                .encode(
                    &mut SimpleWriter(&mut buf),
                    &Record::builder()
                        .level(Level::Debug)
                        .args(format_args!("the message"))
                        .module_path(Some("path"))
                        .file(Some("file"))
                        .line(Some(132))
                        .build()
                )
                .is_ok())
        }
        assert!(String::from_utf8(buf).unwrap().contains("ERROR"));
    }

    #[test]
    fn test_from_piece_to_chunk() {
        // Test 3 args passed to date
        let pattern = "[{d(%Y-%m-%d %H:%M:%S %Z)(utc)(local)}]";
        let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
        match chunks.get(1).unwrap() {
            Chunk::Error(err) => assert_eq!(err, "expected at most two arguments"),
            _ => assert!(false),
        }

        // Test unexepected formatter
        let pattern = "[{d({l} %Y-%m-%d %H:%M:%S %Z)}]";
        let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
        match chunks.get(1).unwrap() {
            Chunk::Formatted { chunk, .. } => match chunk {
                FormattedChunk::Time(value, _tz) => {
                    assert_eq!(value, "{ERROR: unexpected formatter} %Y-%m-%d %H:%M:%S %Z")
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }

        let tests = vec![
            ("[{d(%Y-%m-%d %H:%M:%S %Z)(zulu)}]", "invalid timezone"),
            ("[{d(%Y-%m-%d %H:%M:%S %Z)({l})}]", "invalid timezone"),
            ("[{d(%Y-%m-%d %H:%M:%S %Z)()}]", "invalid timezone"),
            ("[{h({l})({M}):<5.5}]", "expected exactly one argument"),
            (
                "[{D({l})({M}):<5.5}{R({l})({M}):<5.5}]",
                "expected exactly one argument",
            ),
            (
                "[{X(user_id)(foobar)(test):<5.5}]",
                "expected at most two arguments",
            ),
            ("[{X({l user_id):<5.5}]", "expected '}'"),
            ("[{X({l} user_id):<5.5}]", "invalid MDC key"),
            ("[{X:<5.5}]", "missing MDC key"),
            ("[{X(user_id)({l):<5.5}]", "expected '}'"),
            ("[{X(user_id)({l}):<5.5}]", "invalid MDC default"),
            ("[{X(user_id)():<5.5} {M}]", "invalid MDC default"),
        ];

        for (pattern, error_msg) in tests {
            let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
            match chunks.get(1).unwrap() {
                Chunk::Error(err) => assert!(err.contains(error_msg)),
                _ => assert!(false),
            }
        }

        // Test expected 1 arg
        let pattern = "{({l} {m})()}";
        let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
        match chunks.get(0).unwrap() {
            Chunk::Error(err) => assert!(err.contains("expected exactly one argument")),
            _ => assert!(false),
        }

        // Test no_args
        let pattern = "{l()}";
        let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
        match chunks.get(0).unwrap() {
            Chunk::Error(err) => assert!(err.contains("unexpected arguments")),
            _ => assert!(false),
        }
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn test_encode_formatted_chunk() {
        // Each test gets a new buf and writer to allow for checking the
        // buffer and utilizing completely clean buffers.

        let record = Record::builder()
            .level(Level::Info)
            .args(format_args!("the message"))
            .module_path(Some("path"))
            .file(Some("file"))
            .line(None)
            .target("target")
            .build();

        // Limit the time tests to the year. Just need to verify that time can
        // be written. Don't need to be precise. This should limit potential
        // race condition failures.

        // Test UTC Time
        let mut write_buf = vec![];
        let mut w = SimpleWriter(&mut write_buf);
        let chunk = FormattedChunk::Time("%Y".to_owned(), Timezone::Utc);
        chunk.encode(&mut w, &record).unwrap();
        assert_eq!(write_buf, Utc::now().format("%Y").to_string().as_bytes());

        // Test Local Time
        let mut write_buf = vec![];
        let mut w = SimpleWriter(&mut write_buf);
        let chunk = FormattedChunk::Time("%Y".to_owned(), Timezone::Local);
        chunk.encode(&mut w, &record).unwrap();
        assert_eq!(write_buf, Local::now().format("%Y").to_string().as_bytes());

        // Test missing Line
        let mut write_buf = vec![];
        let mut w = SimpleWriter(&mut write_buf);
        let chunk = FormattedChunk::Line;
        chunk.encode(&mut w, &record).unwrap();
        assert_eq!(write_buf, b"???");

        // Test Target
        let mut write_buf = vec![];
        let mut w = SimpleWriter(&mut write_buf);
        let chunk = FormattedChunk::Target;
        chunk.encode(&mut w, &record).unwrap();
        assert_eq!(write_buf, b"target");

        // Test Newline
        let mut write_buf = vec![];
        let mut w = SimpleWriter(&mut write_buf);
        let chunk = FormattedChunk::Newline;
        chunk.encode(&mut w, &record).unwrap();
        assert_eq!(write_buf, NEWLINE.as_bytes());

        // Loop over to hit each possible styling
        for level in Level::iter() {
            let record = Record::builder()
                .level(level)
                .args(format_args!("the message"))
                .module_path(Some("path"))
                .file(Some("file"))
                .line(None)
                .target("target")
                .build();

            let mut write_buf = vec![];
            let mut w = SimpleWriter(&mut write_buf);
            let chunk = FormattedChunk::Highlight(vec![Chunk::Text("Text".to_owned())]);
            chunk.encode(&mut w, &record).unwrap();
            assert_eq!(write_buf, b"Text");
            // No style updates in the buffer to check for
        }
    }
}
