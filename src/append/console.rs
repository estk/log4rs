//! The console appender.
//!
//! Requires the `console_appender` feature.

use std::io::{self, Write};
use std::fmt;
use std::error::Error;
use log::Record;

use append::Append;
use encode::{self, Encode, Style};
#[cfg(feature = "file")]
use encode::EncoderConfig;
use encode::pattern::PatternEncoder;
use encode::writer::simple::SimpleWriter;
use encode::writer::console::{ConsoleWriter, ConsoleWriterLock};
#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};
use priv_io::{StdWriter, StdWriterLock};
use record::ExtendedRecord;

/// The console appender's configuration.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsoleAppenderConfig {
    target: Option<ConfigTarget>,
    encoder: Option<EncoderConfig>,
}

#[cfg(feature = "file")]
#[derive(Deserialize)]
enum ConfigTarget {
    #[serde(rename = "stdout")] Stdout,
    #[serde(rename = "stderr")] Stderr,
}

enum Writer {
    Tty(ConsoleWriter),
    Raw(StdWriter),
}

impl Writer {
    fn lock<'a>(&'a self) -> WriterLock<'a> {
        match *self {
            Writer::Tty(ref w) => WriterLock::Tty(w.lock()),
            Writer::Raw(ref w) => WriterLock::Raw(SimpleWriter(w.lock())),
        }
    }
}

enum WriterLock<'a> {
    Tty(ConsoleWriterLock<'a>),
    Raw(SimpleWriter<StdWriterLock<'a>>),
}

impl<'a> io::Write for WriterLock<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            WriterLock::Tty(ref mut w) => w.write(buf),
            WriterLock::Raw(ref mut w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            WriterLock::Tty(ref mut w) => w.flush(),
            WriterLock::Raw(ref mut w) => w.flush(),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match *self {
            WriterLock::Tty(ref mut w) => w.write_all(buf),
            WriterLock::Raw(ref mut w) => w.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        match *self {
            WriterLock::Tty(ref mut w) => w.write_fmt(fmt),
            WriterLock::Raw(ref mut w) => w.write_fmt(fmt),
        }
    }
}

impl<'a> encode::Write for WriterLock<'a> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        match *self {
            WriterLock::Tty(ref mut w) => w.set_style(style),
            WriterLock::Raw(ref mut w) => w.set_style(style),
        }
    }
}

/// An appender which logs to standard out.
///
/// It supports output styling if standard out is a console buffer on Windows
/// or is a TTY on Unix.
pub struct ConsoleAppender {
    writer: Writer,
    encoder: Box<Encode>,
}

impl fmt::Debug for ConsoleAppender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ConsoleAppender")
            .field("encoder", &self.encoder)
            .finish()
    }
}

impl Append for ConsoleAppender {
    fn append(&self, record: &ExtendedRecord) -> Result<(), Box<Error + Sync + Send>> {
        let mut writer = self.writer.lock();
        self.encoder.encode(&mut writer, record)?;
        writer.flush()?;
        Ok(())
    }

    fn flush(&self) {}
}

impl ConsoleAppender {
    /// Creates a new `ConsoleAppender` builder.
    pub fn builder() -> ConsoleAppenderBuilder {
        ConsoleAppenderBuilder {
            encoder: None,
            target: Target::Stdout,
        }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    encoder: Option<Box<Encode>>,
    target: Target,
}

impl ConsoleAppenderBuilder {
    /// Sets the output encoder for the `ConsoleAppender`.
    pub fn encoder(mut self, encoder: Box<Encode>) -> ConsoleAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// Sets the output stream to log to.
    ///
    /// Defaults to `Target::Stdout`.
    pub fn target(mut self, target: Target) -> ConsoleAppenderBuilder {
        self.target = target;
        self
    }

    /// Consumes the `ConsoleAppenderBuilder`, producing a `ConsoleAppender`.
    pub fn build(self) -> ConsoleAppender {
        let writer = match self.target {
            Target::Stderr => match ConsoleWriter::stderr() {
                Some(writer) => Writer::Tty(writer),
                None => Writer::Raw(StdWriter::stderr()),
            },
            Target::Stdout => match ConsoleWriter::stdout() {
                Some(writer) => Writer::Tty(writer),
                None => Writer::Raw(StdWriter::stdout()),
            },
        };

        ConsoleAppender {
            writer: writer,
            encoder: self.encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
        }
    }
}

/// The stream to log to.
pub enum Target {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// A deserializer for the `ConsoleAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: console
///
/// # The output to write to. One of `stdout` or `stderr`. Defaults to `stdout`.
/// target: stdout
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
/// ```
#[cfg(feature = "file")]
pub struct ConsoleAppenderDeserializer;

#[cfg(feature = "file")]
impl Deserialize for ConsoleAppenderDeserializer {
    type Trait = Append;

    type Config = ConsoleAppenderConfig;

    fn deserialize(
        &self,
        config: ConsoleAppenderConfig,
        deserializers: &Deserializers,
    ) -> Result<Box<Append>, Box<Error + Sync + Send>> {
        let mut appender = ConsoleAppender::builder();
        if let Some(target) = config.target {
            let target = match target {
                ConfigTarget::Stdout => Target::Stdout,
                ConfigTarget::Stderr => Target::Stderr,
            };
            appender = appender.target(target);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }
        Ok(Box::new(appender.build()))
    }
}
