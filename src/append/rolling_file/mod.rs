//! A rolling file appender.
//!
//! Logging directly to a file can be a dangerous proposition for long running
//! processes. You wouldn't want to start a server up and find out a couple
//! weeks later that the disk is filled with hundreds of gigabytes of logs! A
//! rolling file appender alleviates these issues by limiting the amount of log
//! data that's preserved.
//!
//! Like a normal file appender, a rolling file appender is configured with the
//! location of its log file and the encoder which formats log events written
//! to it. In addition, it holds a "policy" object which controls when a log
//! file is rolled over and how the old files are archived.
//!
//! For example, you may configure an appender to roll the log over once it
//! reaches 50 megabytes, and to preserve the last 10 log files.

use antidote::Mutex;
use log::LogRecord;
use serde;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufWriter};
use std::path::{Path, PathBuf};
use serde_value::Value;

use append::Append;
use encode::{self, Encode, EncoderConfig};
use encode::pattern::PatternEncoder;
use file::{Deserialize, Deserializers};

pub mod policy;

include!("config.rs");

struct Policy {
    kind: String,
    config: Value,
}

impl serde::Deserialize for Policy {
    fn deserialize<D>(d: &mut D) -> Result<Policy, D::Error>
        where D: serde::Deserializer
    {
        let mut map = try!(BTreeMap::<Value, Value>::deserialize(d));

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => try!(kind.deserialize_into().map_err(|e| e.to_error())),
            None => "compound".to_owned(),
        };

        Ok(Policy {
            kind: kind,
            config: Value::Map(map),
        })
    }
}

struct LogWriter {
    file: BufWriter<File>,
    len: u64,
}

impl io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf).map(|n| {
            self.len += n as u64;
            n
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl encode::Write for LogWriter {}

/// Information about the active log file.
pub struct LogFile<'a> {
    writer: &'a mut Option<LogWriter>,
    path: &'a Path,
    len: u64,
}

impl<'a> LogFile<'a> {
    /// Returns the path to the log file.
    pub fn path(&self) -> &Path {
        self.path
    }

    /// Returns an estimate of the log file's current size.
    ///
    /// This is calculated by taking the size of the log file when it is opened
    /// and adding the number of bytes written. It may be inaccurate if any
    /// writes have failed or if another process has modified the file
    /// concurrently.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Triggers the log file to roll over.
    ///
    /// A policy must call this method when it wishes to roll the log. The
    /// appender's handle to the file will be closed, which is necessary to
    /// move or delete the file on Windows.
    ///
    /// If this method is called, the log file must no longer be present on
    /// disk when the policy returns.
    pub fn roll(&mut self) {
        *self.writer = None;
    }
}

/// An appender which archives log files in a configurable strategy.
pub struct RollingFileAppender {
    writer: Mutex<Option<LogWriter>>,
    path: PathBuf,
    append: bool,
    encoder: Box<Encode>,
    policy: Box<policy::Policy>,
}

impl fmt::Debug for RollingFileAppender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("RollingFileAppender")
            .field("path", &self.path)
            .field("append", &self.append)
            .field("encoder", &self.encoder)
            .field("policy", &self.policy)
            .finish()
    }
}

impl Append for RollingFileAppender {
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>> {
        let mut writer = self.writer.lock();

        if writer.is_none() {
            if let Some(parent) = self.path.parent() {
                try!(fs::create_dir_all(parent));
            }

            let file = try!(OpenOptions::new()
                .write(true)
                .append(self.append)
                .truncate(!self.append)
                .create(true)
                .open(&self.path));
            let len = if self.append {
                try!(file.metadata()).len()
            } else {
                0
            };
            *writer = Some(LogWriter {
                file: BufWriter::with_capacity(1024, file),
                len: len,
            });
        }

        let len = {
            // :( unwrap
            let writer = writer.as_mut().unwrap();
            try!(self.encoder.encode(writer, record));
            try!(writer.flush());
            writer.len
        };

        let mut file = LogFile {
            writer: &mut writer,
            path: &self.path,
            len: len,
        };

        self.policy.process(&mut file)
    }
}

impl RollingFileAppender {
    /// Creates a new `RollingFileAppenderBuilder`.
    pub fn builder() -> RollingFileAppenderBuilder {
        RollingFileAppenderBuilder {
            append: true,
            encoder: None,
        }
    }
}

/// A builder for the `RollingFileAppender`.
pub struct RollingFileAppenderBuilder {
    append: bool,
    encoder: Option<Box<Encode>>,
}

impl RollingFileAppenderBuilder {
    /// Determines if the appender will append to or truncate the log file.
    ///
    /// Defaults to `true`.
    pub fn append(mut self, append: bool) -> RollingFileAppenderBuilder {
        self.append = append;
        self
    }

    /// Sets the encoder used by the appender.
    ///
    /// Defaults to a `PatternEncoder` with the default pattern.
    pub fn encoder(mut self, encoder: Box<Encode>) -> RollingFileAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// Constructs a `RollingFileAppender`.
    pub fn build<P>(self, path: P, policy: Box<policy::Policy>) -> RollingFileAppender
        where P: AsRef<Path>
    {
        RollingFileAppender {
            writer: Mutex::new(None),
            path: path.as_ref().to_owned(),
            append: self.append,
            encoder: self.encoder.unwrap_or_else(|| Box::new(PatternEncoder::default())),
            policy: policy,
        }
    }
}

/// A deserializer for the `RollingFileAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: rolling_file
///
/// # The path of the log file. Required.
/// path: log/foo.log
///
/// # Specifies if the appender should append to or truncate the log file if it
/// # already exists. Defaults to `true`.
/// append: true
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
///
/// # The policy which handles rotation of the log file. Required.
/// policy:
///   # Identifies which policy is to be used. If no kind is specified, it will
///   # default to "compound".
///   kind: compound
///
///   # The remainder of the configuration is passed along to the policy's
///   # deserializer, and will vary based on the kind of policy.
///   trigger:
///     kind: size
///     limit: 10 mb
///
///   roller:
///     kind: delete
/// ```
pub struct RollingFileAppenderDeserializer;

impl Deserialize for RollingFileAppenderDeserializer {
    type Trait = Append;

    type Config = RollingFileAppenderConfig;

    fn deserialize(&self,
                   config: RollingFileAppenderConfig,
                   deserializers: &Deserializers)
                   -> Result<Box<Append>, Box<Error>> {
        let mut builder = RollingFileAppender::builder();
        if let Some(append) = config.append {
            builder = builder.append(append);
        }
        if let Some(encoder) = config.encoder {
            let encoder = try!(deserializers.deserialize(&encoder.kind, encoder.config));
            builder = builder.encoder(encoder);
        }

        let policy = try!(deserializers.deserialize(&config.policy.kind, config.policy.config));
        Ok(Box::new(builder.build(config.path, policy)))
    }
}

#[cfg(test)]
#[cfg(feature = "yaml")]
mod test {
    use file::{Config, Deserializers, Format};

    #[test]
    fn deserialize() {
        let config = "
appenders:
  foo:
    kind: rolling_file
    path: foo.log
    policy:
      trigger:
        kind: size
        limit: 1024
      roller:
        kind: delete
  bar:
    kind: rolling_file
    path: foo.log
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 5 mb
      roller:
        kind: fixed_window
        pattern: 'foo.log.{}'
        base: 1
        count: 5
";

        let config = Config::parse(config, Format::Yaml, &Deserializers::default()).unwrap();
        println!("{:?}", config.errors());
        assert!(config.errors().is_empty());
    }
}
