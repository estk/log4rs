//! The console appender.

use std::io::{self, Write, Stdout, StdoutLock};
use std::fmt;
use std::error::Error;
use log::LogRecord;

use append::Append;
use encode::{self, Encode, Style};
use encode::pattern::PatternEncoder;
use encode::writer::{SimpleWriter, ConsoleWriter, ConsoleWriterLock};
use file::{Deserialize, Deserializers};
use file::raw::Encoder;

include!("serde.rs");

enum Writer {
    Tty(ConsoleWriter),
    Raw(Stdout),
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
    Raw(SimpleWriter<StdoutLock<'a>>),
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

/// An appender which logs to stdout.
pub struct ConsoleAppender {
    stdout: Writer,
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
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>> {
        let mut stdout = self.stdout.lock();
        try!(self.encoder.encode(&mut stdout, record));
        try!(stdout.flush());
        Ok(())
    }
}

impl ConsoleAppender {
    /// Creates a new `ConsoleAppender` builder.
    pub fn builder() -> ConsoleAppenderBuilder {
        ConsoleAppenderBuilder { encoder: None }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    encoder: Option<Box<Encode>>,
}

impl ConsoleAppenderBuilder {
    /// Sets the output encoder for the `ConsoleAppender`.
    pub fn encoder(mut self, encoder: Box<Encode>) -> ConsoleAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// Consumes the `ConsoleAppenderBuilder`, producing a `ConsoleAppender`.
    pub fn build(self) -> ConsoleAppender {
        let stdout = match ConsoleWriter::stdout() {
            Some(stdout) => Writer::Tty(stdout),
            None => Writer::Raw(io::stdout()),
        };
        ConsoleAppender {
            stdout: stdout,
            encoder: self.encoder.unwrap_or_else(|| Box::new(PatternEncoder::default())),
        }
    }
}

/// A deserializer for the `ConsoleAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: console
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
/// ```
pub struct ConsoleAppenderDeserializer;

impl Deserialize for ConsoleAppenderDeserializer {
    type Trait = Append;

    type Config = ConsoleAppenderConfig;

    fn deserialize(&self,
                   config: ConsoleAppenderConfig,
                   deserializers: &Deserializers)
                   -> Result<Box<Append>, Box<Error>> {
        let mut appender = ConsoleAppender::builder();
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(try!(deserializers.deserialize("encoder",
                                                                       &encoder.kind,
                                                                       encoder.config)));
        }
        Ok(Box::new(appender.build()))
    }
}
