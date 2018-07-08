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
//! * `l``, level` - The log level.
//! * `L`, `line` - The line that the log message came from, or `???` if not
//!     provided.
//! * `m`, `message` - The log message.
//! * `M`, `module` - The module that the log message came from, or `???` if not
//!     provided.
//! * `n` - A platform-specific newline.
//! * `t`, `target` - The target of the log message.
//! * `T`, `thread` - The name of the current thread.
//! * `I`, `thread_id` - The ID of the current thread.
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
use log::{Level, Record};
use log_mdc;
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::io;
use std::thread;
use thread_id;

use encode::pattern::parser::{Alignment, Parameters, Parser, Piece};
use encode::{self, Color, Encode, Style, NEWLINE};
#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};

mod parser;

/// The pattern encoder's configuration.
#[cfg(feature = "file")]
#[derive(Deserialize)]
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
    w: &'a mut encode::Write,
}

impl<'a> io::Write for MaxWidthWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut remaining = self.remaining;
        let mut end = buf.len();
        for (idx, _) in buf.iter()
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

enum Chunk {
    Text(String),
    Formatted {
        chunk: FormattedChunk,
        params: Parameters,
    },
    Error(String),
}

impl Chunk {
    fn encode(&self, w: &mut encode::Write, record: &Record) -> io::Result<()> {
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
                        w: w,
                    };
                    chunk.encode(&mut w, record)
                }
                (Some(min_width), None, Alignment::Left) => {
                    let mut w = LeftAlignWriter {
                        to_fill: min_width,
                        fill: params.fill,
                        w: w,
                    };
                    chunk.encode(&mut w, record)?;
                    w.finish()
                }
                (Some(min_width), None, Alignment::Right) => {
                    let mut w = RightAlignWriter {
                        to_fill: min_width,
                        fill: params.fill,
                        w: w,
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
                            w: w,
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
                            w: w,
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

                    let format = match formatter.args.get(0) {
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
                            if arg.len() != 1 {
                                return Chunk::Error("invalid timezone".to_owned());
                            }
                            match arg[0] {
                                Piece::Text(ref z) if *z == "utc" => Timezone::Utc,
                                Piece::Text(ref z) if *z == "local" => Timezone::Local,
                                Piece::Text(ref z) => {
                                    return Chunk::Error(format!("invalid timezone `{}`", z));
                                }
                                _ => return Chunk::Error("invalid timezone".to_owned()),
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
                "l" | "level" => no_args(&formatter.args, parameters, FormattedChunk::Level),
                "m" | "message" => no_args(&formatter.args, parameters, FormattedChunk::Message),
                "M" | "module" => no_args(&formatter.args, parameters, FormattedChunk::Module),
                "n" => no_args(&formatter.args, parameters, FormattedChunk::Newline),
                "f" | "file" => no_args(&formatter.args, parameters, FormattedChunk::File),
                "L" | "line" => no_args(&formatter.args, parameters, FormattedChunk::Line),
                "T" | "thread" => no_args(&formatter.args, parameters, FormattedChunk::Thread),
                "I" | "thread_id" => no_args(&formatter.args, parameters, FormattedChunk::ThreadId),
                "t" | "target" => no_args(&formatter.args, parameters, FormattedChunk::Target),
                "X" | "mdc" => {
                    if formatter.args.len() > 2 {
                        return Chunk::Error("expected at most two arguments".to_owned());
                    }

                    let key = match formatter.args.get(0) {
                        Some(arg) => {
                            if arg.len() != 1 {
                                return Chunk::Error("invalid MDC key".to_owned());
                            }
                            match arg[0] {
                                Piece::Text(key) => key.to_owned(),
                                Piece::Error(ref e) => return Chunk::Error(e.clone()),
                                _ => return Chunk::Error("invalid MDC key".to_owned()),
                            }
                        }
                        None => return Chunk::Error("missing MDC key".to_owned()),
                    };

                    let default = match formatter.args.get(1) {
                        Some(arg) => {
                            if arg.len() != 1 {
                                return Chunk::Error("invalid MDC default".to_owned());
                            }
                            match arg[0] {
                                Piece::Text(key) => key.to_owned(),
                                Piece::Error(ref e) => return Chunk::Error(e.clone()),
                                _ => return Chunk::Error("invalid MDC default".to_owned()),
                            }
                        }
                        None => "".to_owned(),
                    };

                    Chunk::Formatted {
                        chunk: FormattedChunk::Mdc(key, default),
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
        Chunk::Formatted {
            chunk: chunk,
            params: params,
        }
    } else {
        Chunk::Error("unexpected arguments".to_owned())
    }
}

enum Timezone {
    Utc,
    Local,
}

enum FormattedChunk {
    Time(String, Timezone),
    Level,
    Message,
    Module,
    File,
    Line,
    Thread,
    ThreadId,
    Target,
    Newline,
    Align(Vec<Chunk>),
    Highlight(Vec<Chunk>),
    Mdc(String, String),
}

impl FormattedChunk {
    fn encode(&self, w: &mut encode::Write, record: &Record) -> io::Result<()> {
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
            FormattedChunk::ThreadId => {
                w.write_all(thread_id::get().to_string().as_bytes())
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
                    Level::Warn => w.set_style(Style::new().text(Color::Red))?,
                    Level::Info => w.set_style(Style::new().text(Color::Blue))?,
                    _ => {}
                }
                for chunk in chunks {
                    chunk.encode(w, record)?;
                }
                match record.level() {
                    Level::Error | Level::Warn | Level::Info => w.set_style(&Style::new())?,
                    _ => {}
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
    fn encode(
        &self,
        w: &mut encode::Write,
        record: &Record,
    ) -> Result<(), Box<Error + Sync + Send>> {
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
#[cfg(feature = "file")]
pub struct PatternEncoderDeserializer;

#[cfg(feature = "file")]
impl Deserialize for PatternEncoderDeserializer {
    type Trait = Encode;

    type Config = PatternEncoderConfig;

    fn deserialize(
        &self,
        config: PatternEncoderConfig,
        _: &Deserializers,
    ) -> Result<Box<Encode>, Box<Error + Sync + Send>> {
        let encoder = match config.pattern {
            Some(pattern) => PatternEncoder::new(&pattern),
            None => PatternEncoder::default(),
        };
        Ok(Box::new(encoder))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "simple_writer")]
    use std::thread;
    #[cfg(feature = "simple_writer")]
    use log::{Level, Record};
    #[cfg(feature = "simple_writer")]
    use log_mdc;
    #[cfg(feature = "simple_writer")]
    use thread_id;

    use super::{Chunk, PatternEncoder};
    #[cfg(feature = "simple_writer")]
    use encode::Encode;
    #[cfg(feature = "simple_writer")]
    use encode::writer::simple::SimpleWriter;

    fn error_free(encoder: &PatternEncoder) -> bool {
        encoder.chunks.iter().all(|c| match *c {
            Chunk::Error(_) => false,
            _ => true,
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
    #[cfg(feature = "simple_writer")]
    fn log() {
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
        ).unwrap();

        assert_eq!(buf, &b"DEBUG the message at path in file:132"[..]);
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn unnamed_thread() {
        thread::spawn(|| {
            let pw = PatternEncoder::new("{T}");
            let mut buf = vec![];
            pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
                .unwrap();
            assert_eq!(buf, b"unnamed");
        }).join()
            .unwrap();
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn named_thread() {
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
    fn thread_id() {
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
    fn default_okay() {
        assert!(error_free(&PatternEncoder::default()));
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn left_align() {
        let pw = PatternEncoder::new("{m:~<5.6}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foo")).build(),
        ).unwrap();
        assert_eq!(buf, b"foo~~");

        buf.clear();
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foobar!")).build(),
        ).unwrap();
        assert_eq!(buf, b"foobar");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn right_align() {
        let pw = PatternEncoder::new("{m:~>5.6}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foo")).build(),
        ).unwrap();
        assert_eq!(buf, b"~~foo");

        buf.clear();
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foobar!")).build(),
        ).unwrap();
        assert_eq!(buf, b"foobar");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn left_align_formatter() {
        let pw = PatternEncoder::new("{({l} {m}):15}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder()
                .level(Level::Info)
                .args(format_args!("foobar!"))
                .build(),
        ).unwrap();
        assert_eq!(buf, b"INFO foobar!   ");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn right_align_formatter() {
        let pw = PatternEncoder::new("{({l} {m}):>15}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder()
                .level(Level::Info)
                .args(format_args!("foobar!"))
                .build(),
        ).unwrap();
        assert_eq!(buf, b"   INFO foobar!");
    }

    #[test]
    fn custom_date_format() {
        assert!(error_free(&PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {m}{n}"
        )));
    }

    #[test]
    fn timezones() {
        assert!(error_free(&PatternEncoder::new("{d(%+)(utc)}")));
        assert!(error_free(&PatternEncoder::new("{d(%+)(local)}")));
        assert!(!error_free(&PatternEncoder::new("{d(%+)(foo)}")));
    }

    #[test]
    fn unescaped_parens() {
        assert!(!error_free(&PatternEncoder::new("(hi)")));
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn escaped_chars() {
        let pw = PatternEncoder::new("{{{m}(())}}");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().args(format_args!("foobar!")).build(),
        ).unwrap();
        assert_eq!(buf, b"{foobar!()}");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn quote_braces_with_backslash() {
        let pw = PatternEncoder::new(r"\{\({l}\)\}\\");

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().level(Level::Info).build(),
        ).unwrap();
        assert_eq!(buf, br"{(INFO)}\");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn mdc() {
        let pw = PatternEncoder::new("{X(user_id)}");
        log_mdc::insert("user_id", "mdc value");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"mdc value");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn mdc_missing_default() {
        let pw = PatternEncoder::new("{X(user_id)}");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn mdc_missing_custom() {
        let pw = PatternEncoder::new("{X(user_id)(missing value)}");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"missing value");
    }
}
