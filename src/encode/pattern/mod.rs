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
//! format_spec := [ [ fill ] align ] [left_truncate] [ min_width ] [ '.' max_width ]
//! fill := character
//! align := '<' | '>'
//! min_width := number
//! max_width := number
//! left_truncate := '-'
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
//! * `K`, `key_value` - A value from a [log::kv][log_kv] structured logging
//!     record attributes. The first argument specifies the key, and the second
//!     argument specifies the default value if the key is not present in the
//!     log record's attributes. The second argument is optional, and defaults
//!     to the empty string. This formatter requires the `log_kv` feature to be
//!     enabled.
//!     * `{K(user_id)}` - `123e4567-e89b-12d3-a456-426655440000`
//!     * `{K(nonexistent_key)(no mapping)}` - `no mapping`
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
//! Truncation will cut the right end of the contents, unless left truncation
//! is specified (with a minus sign). Left/right truncation and left/right
//! alignment are specified independently.
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
//! The pattern `{({l} {m}):-15.15}` will behave as above, except the truncation
//! will be from the left. For example, at `DEBUG` level, and a message of
//! `hello, world!`, the output will be: `G hello, world!`
//!
//! [MDC]: https://crates.io/crates/log-mdc
//! [log_kv]: https://docs.rs/log/latest/log/kv/index.html

use chrono::{Local, Utc};
use derive_more::Debug;
use log::{Level, Record};
use std::{default::Default, io, mem, process, thread};
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

use crate::encode::{
    self,
    pattern::parser::{Alignment, Parameters, Parser, Piece},
    Color, Encode, Style, NEWLINE,
};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

use self::parser::Formatter;

mod parser;

#[cfg(not(target_family = "wasm"))]
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
            } => match (params.min_width, params.max_width) {
                (None, None) => chunk.encode(w, record),
                _ => {
                    let mut w = StringBasedWriter::new(w, params);
                    chunk.encode(&mut w, record)?;
                    w.chunk_end()
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
                "X" | "mdc" => match kv_parsing(&formatter) {
                    Err(e) => Chunk::Error(format!("MDC: {e}")),
                    Ok((key, default)) => Chunk::Formatted {
                        chunk: FormattedChunk::Mdc(key, default),
                        params: parameters,
                    },
                },
                #[cfg(feature = "log_kv")]
                "K" | "key_value" => match kv_parsing(&formatter) {
                    Err(e) => Chunk::Error(format!("key_value: {e}")),
                    Ok((key, default)) => Chunk::Formatted {
                        chunk: FormattedChunk::Kv(key, default),
                        params: parameters,
                    },
                },
                #[cfg(not(feature = "log_kv"))]
                "K" | "key_value" => Chunk::Error(
                    "The log_kv feature is required to parse the key_value argument".to_owned(),
                ),
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

enum StringOrStyle {
    String { glen: usize, s: String }, //glen means length in graphemes
    Style(Style),
}

struct StringBasedWriter<'writer, 'params> {
    buf: Vec<u8>,
    strings_and_styles: Vec<StringOrStyle>,
    w: &'writer mut dyn encode::Write,
    params: &'params Parameters,
}

impl encode::Write for StringBasedWriter<'_, '_> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        self.push_string();
        self.strings_and_styles
            .push(StringOrStyle::Style(style.clone()));
        Ok(())
    }
}

impl io::Write for StringBasedWriter<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'writer, 'params> StringBasedWriter<'writer, 'params> {
    fn new(w: &'writer mut dyn encode::Write, params: &'params Parameters) -> Self {
        StringBasedWriter {
            buf: Vec::new(),
            strings_and_styles: Vec::new(),
            w,
            params,
        }
    }

    fn push_string(&mut self) {
        if !self.buf.is_empty() {
            let old_buf = mem::take(&mut self.buf);
            let s = String::from_utf8_lossy(&old_buf[..]).into_owned();
            let glen = s.graphemes(true).count();
            self.strings_and_styles
                .push(StringOrStyle::String { glen, s });
        }
    }

    fn chunk_end(&mut self) -> io::Result<()> {
        self.push_string();
        let total_width = self.compute_width();
        let mut done = false;
        if let Some(max_width) = self.params.max_width {
            if total_width > max_width {
                if self.params.right_truncate {
                    self.output_right_truncate(max_width)?;
                } else {
                    self.output_left_truncate(total_width, max_width)?;
                }
                done = true;
            }
        }
        if let Some(min_width) = self.params.min_width {
            if total_width < min_width {
                if self.params.align == Alignment::Left {
                    self.output_everything()?;
                    self.output_padding(min_width - total_width)?;
                } else {
                    self.output_padding(min_width - total_width)?;
                    self.output_everything()?;
                }
                done = true;
            }
        }
        if !done {
            // between min and max length
            self.output_everything()?;
        }
        Ok(())
    }

    fn compute_width(&self) -> usize {
        let mut size = 0;
        for x in &self.strings_and_styles {
            if let StringOrStyle::String { glen, s: _ } = x {
                size += glen;
            }
        }
        size
    }

    fn output_left_truncate(&mut self, total_width: usize, max_width: usize) -> io::Result<()> {
        let mut to_cut = total_width - max_width;
        for x in &self.strings_and_styles {
            match x {
                StringOrStyle::String { glen, s } => {
                    if to_cut == 0 {
                        self.w.write_all(s.as_bytes())?;
                    } else if *glen <= to_cut {
                        to_cut -= glen;
                    } else {
                        let start = Self::boundary_or(s, to_cut, 0);
                        self.w.write_all(&s.as_bytes()[start..])?;
                        to_cut = 0;
                    }
                }
                StringOrStyle::Style(s) => self.w.set_style(s)?,
            }
        }
        Ok(())
    }

    fn boundary_or(s: &str, count: usize, or: usize) -> usize {
        let mut cursor = GraphemeCursor::new(0, s.len(), true);
        let mut start = 0;
        for _i in 0..count {
            let r = cursor.next_boundary(s, 0);
            if let Ok(Some(x)) = r {
                start = x;
            } else {
                // this should never happen, as we sanitize with to_utf8_lossy
                // but we don't assume so: we'll use the default, which will conservatively
                // output everything instead of trying to cut
                start = or;
                break;
            }
        }
        start
    }

    fn output_right_truncate(&mut self, mut max_width: usize) -> io::Result<()> {
        for x in &self.strings_and_styles {
            match x {
                StringOrStyle::String { glen, s } => {
                    if *glen <= max_width {
                        self.w.write_all(s.as_bytes())?;
                        max_width -= glen;
                    } else {
                        let end = Self::boundary_or(s, max_width, s.len());
                        self.w.write_all(&s.as_bytes()[0..end])?;
                        max_width = 0;
                    }
                    if max_width == 0 {
                        break;
                    }
                }
                StringOrStyle::Style(s) => self.w.set_style(s)?,
            }
        }
        Ok(())
    }

    fn output_everything(&mut self) -> io::Result<()> {
        for x in &self.strings_and_styles {
            match x {
                StringOrStyle::String { glen: _, s } => self.w.write_all(s.as_bytes())?,
                StringOrStyle::Style(s) => self.w.set_style(s)?,
            }
        }
        Ok(())
    }

    fn output_padding(&mut self, len: usize) -> io::Result<()> {
        for _i in 0..len {
            write!(self.w, "{}", self.params.fill)?;
        }
        Ok(())
    }
}

fn no_args(arg: &[Vec<Piece>], params: Parameters, chunk: FormattedChunk) -> Chunk {
    if arg.is_empty() {
        Chunk::Formatted { chunk, params }
    } else {
        Chunk::Error("unexpected arguments".to_owned())
    }
}

fn kv_parsing<'a>(formatter: &'a Formatter) -> Result<(String, String), &'a str> {
    if formatter.args.len() > 2 {
        return Err("expected at most two arguments");
    }

    let key = match formatter.args.first() {
        Some(arg) => {
            if let Some(arg) = arg.first() {
                match arg {
                    Piece::Text(key) => key.to_owned(),
                    Piece::Error(ref e) => return Err(e),
                    _ => return Err("invalid key"),
                }
            } else {
                return Err("invalid key");
            }
        }
        None => return Err("missing key"),
    };

    let default = match formatter.args.get(1) {
        Some(arg) => {
            if let Some(arg) = arg.first() {
                match arg {
                    Piece::Text(key) => key.to_owned(),
                    Piece::Error(ref e) => return Err(e),
                    _ => return Err("invalid default"),
                }
            } else {
                return Err("invalid default");
            }
        }
        None => "",
    };
    Ok((key.into(), default.into()))
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
    #[cfg(feature = "log_kv")]
    Kv(String, String),
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
            #[cfg(not(target_family = "wasm"))]
            FormattedChunk::ThreadId => w.write_all(thread_id::get().to_string().as_bytes()),
            #[cfg(target_family = "wasm")]
            FormattedChunk::ThreadId => w.write_all("0".as_bytes()),
            FormattedChunk::ProcessId => w.write_all(process::id().to_string().as_bytes()),
            #[cfg(not(target_family = "wasm"))]
            FormattedChunk::SystemThreadId => {
                TID.with(|tid| w.write_all(tid.to_string().as_bytes()))
            }
            #[cfg(target_family = "wasm")]
            FormattedChunk::SystemThreadId => w.write_all("0".as_bytes()),
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
            #[cfg(feature = "log_kv")]
            FormattedChunk::Kv(ref key, ref default) => {
                use log::kv::ToKey;
                if let Some(v) = record.key_values().get(key.to_key()) {
                    write!(w, "{v}")
                } else {
                    write!(w, "{default}")
                }
            }
        }
    }
}

/// An `Encode`r configured via a format string.
#[derive(Clone, Eq, Debug, PartialEq, Hash)]
pub struct PatternEncoder {
    #[debug(skip)]
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
    #[cfg(feature = "simple_writer")]
    use log::{Level, Record};
    #[cfg(feature = "simple_writer")]
    use std::process;
    #[cfg(feature = "simple_writer")]
    use std::thread;

    #[cfg(feature = "log_kv")]
    use super::Parser;
    use super::{Chunk, PatternEncoder};
    #[cfg(feature = "simple_writer")]
    use crate::encode::writer::simple::SimpleWriter;
    #[cfg(feature = "simple_writer")]
    use crate::encode::Encode;

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
        )
        .unwrap();

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
        })
        .join()
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
    fn thread_id_field() {
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
    fn process_id() {
        let pw = PatternEncoder::new("{P}");
        let mut buf = vec![];

        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, process::id().to_string().as_bytes());
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn system_thread_id() {
        let pw = PatternEncoder::new("{i}");
        let mut buf = vec![];

        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, thread_id::get().to_string().as_bytes());
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
    fn right_align() {
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

    #[cfg(feature = "simple_writer")]
    fn assert_info_message(pattern: &str, msg: &str, expected: &[u8]) {
        let pw = PatternEncoder::new(pattern);

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder()
                .level(Level::Info)
                .args(format_args!("{}", msg))
                .build(),
        )
        .unwrap();
        assert_eq!(buf, expected);
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn left_align_formatter() {
        assert_info_message("{({l} {m}):15}", "foobar!", b"INFO foobar!   ");
        assert_info_message("{({l} {m}):7}", "foobar!", b"INFO foobar!");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn right_truncate_formatter() {
        assert_info_message("{({l} {m}):7.7}", "foobar!", b"INFO fo");
        assert_info_message("{({l} {m}):12.12}", "foobar!", b"INFO foobar!");
        assert_info_message("{({l} {m}):7.14}", "foobar!", b"INFO foobar!");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn left_truncate_formatter() {
        assert_info_message("{({l} {m}):-9.9}", "foobar!", b"O foobar!");
        assert_info_message("{({l} {m}):-12.12}", "foobar!", b"INFO foobar!");
        assert_info_message("{({l} {m}):-7.14}", "foobar!", b"INFO foobar!");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn right_align_formatter() {
        assert_info_message("{({l} {m}):>15}", "foobar!", b"   INFO foobar!");
        assert_info_message("{({l} {m}):>12}", "foobar!", b"INFO foobar!");
        assert_info_message("{({l} {m}):>7}", "foobar!", b"INFO foobar!");
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn right_align_formatter_hard_unicode() {
        assert_info_message(
            "{({l} {m}):>15}",
            "\u{01f5}\u{0067}\u{0301}",
            "        INFO \u{01f5}\u{0067}\u{0301}".as_bytes(),
        );
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn zalgo_text() {
        let zalgo = "m\u{0301}\u{0302}o\u{0303}\u{0304}\u{0305}\u{0306}re testing l\u{113}ss \u{1F1F7}\u{1F1F8}\u{1F1EE}\u{1F1F4} CVE-2021-30860";
        assert_info_message(
            "{({l} {m}):10.10}",
            zalgo,
            "INFO m\u{0301}\u{0302}o\u{0303}\u{0304}\u{0305}\u{0306}re ".as_bytes(),
        );
        assert_info_message(
            "{({l} {m}):24.24}",
            zalgo,
            "INFO m\u{0301}\u{0302}o\u{0303}\u{0304}\u{0305}\u{0306}re testing l\u{113}ss \u{1F1F7}\u{1F1F8}".as_bytes(),
        );
        assert_info_message(
            "{({l} {m}):-24.24}",
            zalgo,
            "g l\u{113}ss \u{1F1F7}\u{1F1F8}\u{1F1EE}\u{1F1F4} CVE-2021-30860".as_bytes(),
        );
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
        )
        .unwrap();
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
        )
        .unwrap();
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

    #[test]
    #[cfg(all(feature = "simple_writer", feature = "log_kv"))]
    fn test_kv() {
        let pw = PatternEncoder::new("{K(user_id)}");
        let kv = [("user_id", "kv value")];

        let mut buf = vec![];
        pw.encode(
            &mut SimpleWriter(&mut buf),
            &Record::builder().key_values(&kv).build(),
        )
        .unwrap();

        assert_eq!(buf, b"kv value");
    }

    #[test]
    #[cfg(all(feature = "simple_writer", feature = "log_kv"))]
    fn test_kv_missing_default() {
        let pw = PatternEncoder::new("{K(user_id)}");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"");
    }

    #[test]
    #[cfg(all(feature = "simple_writer", feature = "log_kv"))]
    fn test_kv_missing_custom() {
        let pw = PatternEncoder::new("{K(user_id)(missing value)}");

        let mut buf = vec![];
        pw.encode(&mut SimpleWriter(&mut buf), &Record::builder().build())
            .unwrap();

        assert_eq!(buf, b"missing value");
    }

    #[test]
    #[cfg(feature = "log_kv")]
    fn test_kv_from_piece_to_chunk() {
        let tests = vec![
            (
                "[{K(user_id)(foobar)(test):<5.5}]",
                "expected at most two arguments",
            ),
            ("[{K({l user_id):<5.5}]", "expected '}'"),
            ("[{K({l} user_id):<5.5}]", "key_value: invalid key"),
            ("[{K:<5.5}]", "key_value: missing key"),
            ("[{K(user_id)({l):<5.5}]", "expected '}'"),
            ("[{K(user_id)({l}):<5.5}]", "key_value: invalid default"),
            ("[{K(user_id)():<5.5} {M}]", "key_value: invalid default"),
        ];

        for (pattern, error_msg) in tests {
            let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
            match chunks.get(1).unwrap() {
                Chunk::Error(err) => assert!(err.contains(error_msg)),
                _ => panic!(),
            }
        }
    }

    #[test]
    #[cfg(feature = "simple_writer")]
    fn debug_release() {
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
}
