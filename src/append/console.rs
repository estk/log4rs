//! The console appender.
//!
//! Requires the `console_appender` feature.

use derivative::Derivative;
use log::Record;
use std::{
    fmt,
    io::{self, Write},
};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};
#[cfg(feature = "config_parsing")]
use crate::encode::EncoderConfig;
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

/// The console appender's configuration.
#[cfg(feature = "config_parsing")]
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsoleAppenderConfig {
    target: Option<ConfigTarget>,
    encoder: Option<EncoderConfig>,
    tty_only: Option<bool>,
}

#[cfg(feature = "config_parsing")]
#[derive(Debug, serde::Deserialize)]
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

    fn is_tty(&self) -> bool {
        // 1.40 compat
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Self::Tty(_) => true,
            _ => false,
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
#[derive(Derivative)]
#[derivative(Debug)]
pub struct ConsoleAppender {
    #[derivative(Debug = "ignore")]
    writer: Writer,
    encoder: Box<dyn Encode>,
    do_write: bool,
}

impl Append for ConsoleAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        if self.do_write {
            let mut writer = self.writer.lock();
            self.encoder.encode(&mut writer, record)?;
            writer.flush()?;
        }
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
            tty_only: false,
        }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    target: Target,
    tty_only: bool,
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

    /// Sets the output to log only when it's a TTY.
    ///
    /// Defaults to `false`.
    pub fn tty_only(mut self, tty_only: bool) -> ConsoleAppenderBuilder {
        self.tty_only = tty_only;
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

        let do_write = writer.is_tty() || !self.tty_only;

        ConsoleAppender {
            writer,
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::<PatternEncoder>::default()),
            do_write,
        }
    }
}

/// The stream to log to.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
/// # Set this boolean when the console appender must only write when the target is a TTY.
/// tty_only: false
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ConsoleAppenderDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for ConsoleAppenderDeserializer {
    type Trait = dyn Append;

    type Config = ConsoleAppenderConfig;

    fn deserialize(
        &self,
        config: ConsoleAppenderConfig,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<dyn Append>> {
        let mut appender = ConsoleAppender::builder();
        if let Some(target) = config.target {
            let target = match target {
                ConfigTarget::Stdout => Target::Stdout,
                ConfigTarget::Stderr => Target::Stderr,
            };
            appender = appender.target(target);
        }
        if let Some(tty_only) = config.tty_only {
            appender = appender.tty_only(tty_only);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }
        Ok(Box::new(appender.build()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::encode::Write;

    #[test]
    fn test_append() {
        use log::Level;

        // Build a std out appender
        let appender = ConsoleAppender::builder()
            .tty_only(false)
            .target(Target::Stdout)
            .encoder(Box::new(PatternEncoder::new("{m}{n}")))
            .build();

        assert!(appender
            .append(
                &Record::builder()
                    .level(Level::Debug)
                    .target("target")
                    .module_path(Some("module_path"))
                    .file(Some("file"))
                    .line(Some(100))
                    .args(format_args!("{}", "message"))
                    .build()
            )
            .is_ok());

        // No op, but test coverage :)
        appender.flush();
    }

    #[test]
    fn test_builder() {
        // Build a std out appender
        let _appender = ConsoleAppender::builder()
            .tty_only(false)
            .target(Target::Stdout)
            .encoder(Box::new(PatternEncoder::new("{m}{n}")))
            .build();

        // Build a std err appender
        let _appender = ConsoleAppender::builder()
            .tty_only(false)
            .target(Target::Stderr)
            .encoder(Box::new(PatternEncoder::new("{m}{n}")))
            .build();

        // Build a default encoder appender
        let _appender = ConsoleAppender::builder()
            .tty_only(true)
            .target(Target::Stderr)
            .build();
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_config_deser() {
        use crate::{config::Deserializers, encode::EncoderConfig};
        use serde_value::Value;
        use std::collections::BTreeMap;
        let deserializer = ConsoleAppenderDeserializer;

        let targets = vec![ConfigTarget::Stdout, ConfigTarget::Stderr];

        for target in targets {
            let console_cfg = ConsoleAppenderConfig {
                target: Some(target),
                encoder: Some(EncoderConfig {
                    kind: "pattern".to_owned(),
                    config: Value::Map(BTreeMap::new()),
                }),
                tty_only: Some(true),
            };
            assert!(deserializer
                .deserialize(console_cfg, &Deserializers::default())
                .is_ok());
        }
    }

    fn write_test(mut writer: WriterLock) {
        use std::io::Write;

        assert_eq!(writer.write(b"Write log\n").unwrap(), 10);
        assert!(writer.set_style(&Style::new()).is_ok());
        assert!(writer.write_all(b"Write All log\n").is_ok());
        assert!(writer.write_fmt(format_args!("{} \n", "normal")).is_ok());
        assert!(writer.flush().is_ok());
    }

    #[test]
    fn test_tty() {
        // Note that this fails in GitHub Actions and therefore does not
        // show as covered.
        let w = match ConsoleWriter::stdout() {
            Some(w) => w,
            None => return,
        };

        let tty = Writer::Tty(w);
        assert!(tty.is_tty());

        write_test(tty.lock());
    }

    #[test]
    fn test_raw() {
        let raw = Writer::Raw(StdWriter::stdout());
        assert!(!raw.is_tty());
        write_test(raw.lock());
    }
}
