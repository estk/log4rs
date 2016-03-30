//! The console appender.

use std::io::{self, Write, Stdout};
use std::fmt;
use std::error::Error;
use log::LogRecord;
use serde_value::Value;

use append::{Append, SimpleWriter};
use encode::Encode;
use encode::pattern::PatternEncoder;
use file::{Deserialize, Deserializers};
use file::raw::Encoder;

/// An appender which logs to stdout.
pub struct ConsoleAppender {
    stdout: Stdout,
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
        let mut stdout = SimpleWriter(self.stdout.lock());
        try!(self.encoder.encode(&mut stdout, record));
        try!(stdout.flush());
        Ok(())
    }
}

impl ConsoleAppender {
    /// Creates a new `ConsoleAppender` builder.
    pub fn builder() -> ConsoleAppenderBuilder {
        ConsoleAppenderBuilder { encoder: Box::new(PatternEncoder::default()) }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    encoder: Box<Encode>,
}

impl ConsoleAppenderBuilder {
    /// Sets the output encoder for the `ConsoleAppender`.
    pub fn encoder(mut self, encoder: Box<Encode>) -> ConsoleAppenderBuilder {
        self.encoder = encoder;
        self
    }

    /// Consumes the `ConsoleAppenderBuilder`, producing a `ConsoleAppender`.
    pub fn build(self) -> ConsoleAppender {
        ConsoleAppender {
            stdout: io::stdout(),
            encoder: self.encoder,
        }
    }
}

/// A deserializer for the `ConsoleAppender`.
///
/// The `encoder` key is optional and specifies an `Encoder` to be used for
/// output.
pub struct ConsoleAppenderDeserializer;

impl Deserialize for ConsoleAppenderDeserializer {
    type Trait = Append;

    fn deserialize(&self,
                   config: Value,
                   deserializers: &Deserializers)
                   -> Result<Box<Append>, Box<Error>> {
        let config = try!(config.deserialize_into::<ConsoleAppenderConfig>());
        let mut appender = ConsoleAppender::builder();
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(try!(deserializers.deserialize("encoder",
                                                                       &encoder.kind,
                                                                       encoder.config)));
        }
        Ok(Box::new(appender.build()))
    }
}

include!("console_serde.rs");
