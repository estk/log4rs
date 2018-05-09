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
//!
//! Requires the `rolling_file_appender` feature.

use antidote::Mutex;
use log::Record;
#[cfg(feature = "file")]
use serde;
#[cfg(feature = "file")]
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
#[cfg(feature = "file")]
use serde_value::Value;

use append::Append;
use encode::{self, Encode};
#[cfg(feature = "file")]
use encode::EncoderConfig;
use encode::pattern::PatternEncoder;
#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};
use record::ExtendedRecord;

pub mod policy;

/// Configuration for the rolling file appender.
#[cfg(feature = "file")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollingFileAppenderConfig {
    path: String,
    append: Option<bool>,
    encoder: Option<EncoderConfig>,
    policy: Policy,
}

#[cfg(feature = "file")]
struct Policy {
    kind: String,
    config: Value,
}

#[cfg(feature = "file")]
impl<'de> serde::Deserialize<'de> for Policy {
    fn deserialize<D>(d: D) -> Result<Policy, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => kind.deserialize_into().map_err(|e| e.to_error())?,
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
    fn append(&self, record: &ExtendedRecord) -> Result<(), Box<Error + Sync + Send>> {
        let mut writer = self.writer.lock();

        let len = {
            let writer = self.get_writer(&mut writer)?;
            self.encoder.encode(writer, record)?;
            writer.flush()?;
            writer.len
        };

        let mut file = LogFile {
            writer: &mut writer,
            path: &self.path,
            len: len,
        };

        self.policy.process(&mut file)
    }

    fn flush(&self) {}
}

impl RollingFileAppender {
    /// Creates a new `RollingFileAppenderBuilder`.
    pub fn builder() -> RollingFileAppenderBuilder {
        RollingFileAppenderBuilder {
            append: true,
            encoder: None,
        }
    }

    fn get_writer<'a>(&self, writer: &'a mut Option<LogWriter>) -> io::Result<&'a mut LogWriter> {
        if writer.is_none() {
            let file = OpenOptions::new()
                .write(true)
                .append(self.append)
                .truncate(!self.append)
                .create(true)
                .open(&self.path)?;
            let len = if self.append {
                file.metadata()?.len()
            } else {
                0
            };
            *writer = Some(LogWriter {
                file: BufWriter::with_capacity(1024, file),
                len: len,
            });
        }

        // :( unwrap
        Ok(writer.as_mut().unwrap())
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
    pub fn build<P>(self, path: P, policy: Box<policy::Policy>) -> io::Result<RollingFileAppender>
    where
        P: AsRef<Path>,
    {
        let appender = RollingFileAppender {
            writer: Mutex::new(None),
            path: path.as_ref().to_owned(),
            append: self.append,
            encoder: self.encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
            policy: policy,
        };

        if let Some(parent) = appender.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // open the log file immediately
        appender.get_writer(&mut appender.writer.lock())?;

        Ok(appender)
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
#[cfg(feature = "file")]
pub struct RollingFileAppenderDeserializer;

#[cfg(feature = "file")]
impl Deserialize for RollingFileAppenderDeserializer {
    type Trait = Append;

    type Config = RollingFileAppenderConfig;

    fn deserialize(
        &self,
        config: RollingFileAppenderConfig,
        deserializers: &Deserializers,
    ) -> Result<Box<Append>, Box<Error + Sync + Send>> {
        let mut builder = RollingFileAppender::builder();
        if let Some(append) = config.append {
            builder = builder.append(append);
        }
        if let Some(encoder) = config.encoder {
            let encoder = deserializers.deserialize(&encoder.kind, encoder.config)?;
            builder = builder.encoder(encoder);
        }

        let policy = deserializers.deserialize(&config.policy.kind, config.policy.config)?;
        let appender = builder.build(config.path, policy)?;
        Ok(Box::new(appender))
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;
    use std::io::{Read, Write};
    use std::fs::File;
    use tempdir::TempDir;

    use append::rolling_file::policy::Policy;
    use super::*;

    #[test]
    #[cfg(feature = "yaml_format")]
    fn deserialize() {
        use file::{Deserializers, RawConfig};

        let dir = TempDir::new("deserialize").unwrap();

        let config = format!(
            "
appenders:
  foo:
    kind: rolling_file
    path: {0}/foo.log
    policy:
      trigger:
        kind: size
        limit: 1024
      roller:
        kind: delete
  bar:
    kind: rolling_file
    path: {0}/foo.log
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 5 mb
      roller:
        kind: fixed_window
        pattern: '{0}/foo.log.{{}}'
        base: 1
        count: 5
",
            dir.path().display()
        );

        let config = ::serde_yaml::from_str::<RawConfig>(&config).unwrap();
        let errors = config.appenders_lossy(&Deserializers::new()).1;
        println!("{:?}", errors);
        assert!(errors.is_empty());
    }

    #[derive(Debug)]
    struct NopPolicy;

    impl Policy for NopPolicy {
        fn process(&self, _: &mut LogFile) -> Result<(), Box<Error + Sync + Send>> {
            Ok(())
        }
    }

    #[test]
    fn append() {
        let dir = TempDir::new("rolling-file-append").unwrap();
        let path = dir.path().join("append.log");
        RollingFileAppender::builder()
            .append(true)
            .build(&path, Box::new(NopPolicy))
            .unwrap();
        assert!(path.exists());
        File::create(&path).unwrap().write_all(b"hello").unwrap();

        RollingFileAppender::builder()
            .append(true)
            .build(&path, Box::new(NopPolicy))
            .unwrap();
        let mut contents = vec![];
        File::open(&path)
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"hello");
    }

    #[test]
    fn truncate() {
        let dir = TempDir::new("rolling-file-truncate").unwrap();
        let path = dir.path().join("truncate.log");
        RollingFileAppender::builder()
            .append(false)
            .build(&path, Box::new(NopPolicy))
            .unwrap();
        assert!(path.exists());
        File::create(&path).unwrap().write_all(b"hello").unwrap();

        RollingFileAppender::builder()
            .append(false)
            .build(&path, Box::new(NopPolicy))
            .unwrap();
        let mut contents = vec![];
        File::open(&path)
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"");
    }
}
