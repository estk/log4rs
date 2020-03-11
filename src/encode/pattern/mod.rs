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
//! The `{`, `}`, `(`, `)` (all platform), and `\` (not windows)
//! characters are part of the pattern syntax;
//! they must be escaped to appear in output. Like with Rust's string
//! formatting syntax, type the character twice to escape it. That is, `{{`
//! will be rendered as `{` in output and `))` will be rendered as `)`.
//!
//! In addition, when not windows, these characters may also be escaped by prefixing them with a
//! `\` character. That is, `\{` will be rendered as `{`. windows use `\` as path seperator.
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
//! * `P`, `pid` - The current process id.
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

mod log_content;

///! A module for processing log path in configuration
pub mod log_path;

mod parser;

pub use log_content::PatternEncoder;

#[cfg(feature = "file")]
pub use log_content::PatternEncoderDeserializer;
