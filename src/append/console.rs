//! The console appender.
//!
//! Requires the `console_appender` feature.

#[cfg(feature = "dedup")]
use crate::append::dedup::*;
#[cfg(feature = "file")]
use crate::encode::EncoderConfig;
#[cfg(feature = "file")]
use crate::file::{Deserialize, Deserializers};
use crate::{
    append::Append,
    encode::{
        self,
        pattern::PatternEncoder,
        writer::{
            console::{ConsoleWriter, ConsoleWriterLock},
            simple::SimpleWriter,
        },
        Encode, Style,
    },
    priv_io::{StdWriter, StdWriterLock},
};
use log::Record;

#[cfg(feature = "dedup")]
use parking_lot::Mutex;
#[cfg(feature = "file")]
use serde_derive::Deserialize;
use std::{
    error::Error,
    fmt,
    io::{self, Write},
};
/// The console appender's configuration.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsoleAppenderConfig {
    target: Option<ConfigTarget>,
    encoder: Option<EncoderConfig>,
    #[cfg(feature = "dedup")]
    dedup: Option<bool>,
}

#[cfg(feature = "file")]
#[derive(Deserialize)]
enum ConfigTarget {
    #[serde(rename = "stdout")]
    Stdout,
    #[serde(rename = "stderr")]
    Stderr,
}

enum Writer {
    Tty(ConsoleWriter),
    Raw(StdWriter),
}

impl Writer {
    fn lock(&self) -> WriterLock {
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
    encoder: Box<dyn Encode>,
    #[cfg(feature = "dedup")]
    deduper: Option<Mutex<DeDuper>>,
}

impl fmt::Debug for ConsoleAppender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ConsoleAppender")
            .field("encoder", &self.encoder)
            .finish()
    }
}

impl Append for ConsoleAppender {
    fn append(&self, record: &Record) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut writer = self.writer.lock();
        #[cfg(feature = "dedup")]
        let _ = {
            if let Some(dd) = &self.deduper {
                if dd.lock().dedup(&mut writer, &*self.encoder, record)? == DedupResult::Skip {
                    return Ok(());
                }
            }
        };
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
            #[cfg(feature = "dedup")]
            dedup: false,
        }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,

    target: Target,
    #[cfg(feature = "dedup")]
    dedup: bool,
}

impl ConsoleAppenderBuilder {
    /// Sets the output encoder for the `ConsoleAppender`.
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> ConsoleAppenderBuilder {
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
    /// Determines if the appender will reject and count duplicate messages.
    ///
    /// Defaults to `false`.
    #[cfg(feature = "dedup")]
    pub fn dedup(mut self, dedup: bool) -> ConsoleAppenderBuilder {
        self.dedup = dedup;
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
        #[cfg(feature = "dedup")]
        let deduper = {
            if self.dedup {
                Some(Mutex::new(DeDuper::default()))
            } else {
                None
            }
        };

        ConsoleAppender {
            writer,
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
            #[cfg(feature = "dedup")]
            deduper,
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
    type Trait = dyn Append;

    type Config = ConsoleAppenderConfig;

    fn deserialize(
        &self,
        config: ConsoleAppenderConfig,
        deserializers: &Deserializers,
    ) -> Result<Box<dyn Append>, Box<dyn Error + Sync + Send>> {
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
        #[cfg(feature = "dedup")]
        let _ = {
            if let Some(dedup) = config.dedup {
                appender = appender.dedup(dedup);
            }
        };
        Ok(Box::new(appender.build()))
    }
}
