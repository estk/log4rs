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

use derivative::Derivative;
use log::Record;
use parking_lot::Mutex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

#[cfg(feature = "config_parsing")]
use serde_value::Value;
#[cfg(feature = "config_parsing")]
use std::collections::BTreeMap;

use crate::{
    append::Append,
    encode::{self, pattern::PatternEncoder, Encode},
};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};
#[cfg(feature = "config_parsing")]
use crate::encode::EncoderConfig;

pub mod policy;

/// Configuration for the rolling file appender.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollingFileAppenderConfig {
    path: String,
    append: Option<bool>,
    encoder: Option<EncoderConfig>,
    policy: Policy,
}

#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct Policy {
    kind: String,
    config: Value,
}

#[cfg(feature = "config_parsing")]
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
            kind,
            config: Value::Map(map),
        })
    }
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct LogFile<'a> {
    writer: &'a mut Option<LogWriter>,
    path: &'a Path,
    len: u64,
}

#[allow(clippy::len_without_is_empty)]
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
    #[deprecated(since = "0.9.1", note = "Please use the len_estimate function instead")]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns an estimate of the log file's current size.
    ///
    /// This is calculated by taking the size of the log file when it is opened
    /// and adding the number of bytes written. It may be inaccurate if any
    /// writes have failed or if another process has modified the file
    /// concurrently.
    pub fn len_estimate(&self) -> u64 {
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
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RollingFileAppender {
    #[derivative(Debug = "ignore")]
    writer: Mutex<Option<LogWriter>>,
    path: PathBuf,
    append: bool,
    encoder: Box<dyn Encode>,
    policy: Box<dyn policy::Policy>,
}

impl Append for RollingFileAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        // TODO(eas): Perhaps this is better as a concurrent queue?
        let mut writer = self.writer.lock();

        let is_pre_process = self.policy.is_pre_process();
        let log_writer = self.get_writer(&mut writer)?;

        if is_pre_process {
            let len = log_writer.len;

            let mut file = LogFile {
                writer: &mut writer,
                path: &self.path,
                len,
            };

            // TODO(eas): Idea: make this optionally return a future, and if so, we initialize a queue for
            // data that comes in while we are processing the file rotation.

            self.policy.process(&mut file)?;

            let log_writer_new = self.get_writer(&mut writer)?;
            self.encoder.encode(log_writer_new, record)?;
            log_writer_new.flush()?;
        } else {
            self.encoder.encode(log_writer, record)?;
            log_writer.flush()?;
            let len = log_writer.len;

            let mut file = LogFile {
                writer: &mut writer,
                path: &self.path,
                len,
            };

            self.policy.process(&mut file)?;
        }

        Ok(())
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
                len,
            });
        }

        // :( unwrap
        Ok(writer.as_mut().unwrap())
    }
}

/// A builder for the `RollingFileAppender`.
pub struct RollingFileAppenderBuilder {
    append: bool,
    encoder: Option<Box<dyn Encode>>,
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
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> RollingFileAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// Constructs a `RollingFileAppender`.
    /// The path argument can contain environment variables of the form $ENV{name_here},
    /// where 'name_here' will be the name of the environment variable that
    /// will be resolved. Note that if the variable fails to resolve,
    /// $ENV{name_here} will NOT be replaced in the path.
    pub fn build<P>(
        self,
        path: P,
        policy: Box<dyn policy::Policy>,
    ) -> io::Result<RollingFileAppender>
    where
        P: AsRef<Path>,
    {
        let path = super::env_util::expand_env_vars(path.as_ref().to_string_lossy());
        let appender = RollingFileAppender {
            writer: Mutex::new(None),
            path: path.as_ref().into(),
            append: self.append,
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::<PatternEncoder>::default()),
            policy,
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
/// # The path can contain environment variables of the form $ENV{name_here},
/// # where 'name_here' will be the name of the environment variable that
/// # will be resolved. Note that if the variable fails to resolve,
/// # $ENV{name_here} will NOT be replaced in the path.
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
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct RollingFileAppenderDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for RollingFileAppenderDeserializer {
    type Trait = dyn Append;

    type Config = RollingFileAppenderConfig;

    fn deserialize(
        &self,
        config: RollingFileAppenderConfig,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<dyn Append>> {
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
    use super::*;
    use crate::append::rolling_file::policy::Policy;

    use tempfile::NamedTempFile;

    #[cfg(feature = "config_parsing")]
    use serde_test::{assert_de_tokens, Token};

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_config_deserialize() {
        use super::*;
        use serde_value::Value;
        use std::collections::BTreeMap;

        let policy = Policy {
            kind: "compound".to_owned(),
            config: Value::Map(BTreeMap::new()),
        };

        assert_de_tokens(
            &policy,
            &[
                Token::Struct {
                    name: "Policy",
                    len: 1,
                },
                Token::Str("kind"),
                Token::Str("compound"),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &policy,
            &[
                Token::Struct {
                    name: "Policy",
                    len: 0,
                },
                Token::StructEnd,
            ],
        );
    }

    #[test]
    #[cfg(feature = "yaml_format")]
    fn test_deserialize_appenders() {
        use crate::config::{Deserializers, RawConfig};

        let dir = tempfile::tempdir().unwrap();

        let config = format!(
            "
appenders:
    foo:
        kind: rolling_file
        path: {0}/foo.log
        policy:
            trigger:
                kind: time
                interval: 2 minutes
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
        assert!(errors.is_empty());
    }

    #[derive(Debug)]
    struct NopPostPolicy;

    impl Policy for NopPostPolicy {
        fn process(&self, _: &mut LogFile) -> anyhow::Result<()> {
            Ok(())
        }
        fn is_pre_process(&self) -> bool {
            false
        }
    }

    #[derive(Debug)]
    struct NopPrePolicy;

    impl Policy for NopPrePolicy {
        fn process(&self, _: &mut LogFile) -> anyhow::Result<()> {
            Ok(())
        }
        fn is_pre_process(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_rolling_append() {
        use log::Level;

        let tmp_file = NamedTempFile::new().unwrap();
        let policies: Vec<Box<dyn Policy>> = vec![Box::new(NopPrePolicy), Box::new(NopPostPolicy)];
        let record = Record::builder()
            .level(Level::Debug)
            .target("target")
            .module_path(Some("module_path"))
            .file(Some("file"))
            .line(Some(100))
            .build();
        log_mdc::insert("foo", "bar");

        for policy in policies {
            let appender = RollingFileAppender::builder()
                .append(true)
                .encoder(Box::new(PatternEncoder::new("{m}{n}")))
                .build(&tmp_file.path(), policy)
                .unwrap();

            assert!(appender.append(&record).is_ok());

            // No-op method, but get the test coverage :)
            appender.flush();
        }
    }

    #[test]
    fn test_logfile() {
        let tmp_file = NamedTempFile::new().unwrap();
        let mut logfile = LogFile {
            writer: &mut None,
            path: tmp_file.path(),
            len: 0,
        };

        assert_eq!(logfile.path(), tmp_file.path());
        assert_eq!(logfile.len_estimate(), 0);

        // No actions to take here, the writer becomes inaccessible but theres
        // no getter to verify
        logfile.roll();
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_cfg_deserializer() {
        use super::*;
        use crate::config::Deserializers;
        use serde_value::Value;
        use std::collections::BTreeMap;

        let tmp_file = NamedTempFile::new().unwrap();

        let append_cfg = RollingFileAppenderConfig {
            path: tmp_file.path().to_str().unwrap().to_owned(),
            append: Some(true),
            encoder: Some(EncoderConfig {
                kind: "pattern".to_owned(),
                config: Value::Map(BTreeMap::new()),
            }),
            policy: Policy {
                kind: "compound".to_owned(),
                config: Value::Map({
                    let mut map = BTreeMap::new();
                    map.insert(
                        Value::String("trigger".to_owned()),
                        Value::Map({
                            let mut map = BTreeMap::new();
                            map.insert(
                                Value::String("kind".to_owned()),
                                Value::String("size".to_owned()),
                            );
                            map.insert(
                                Value::String("limit".to_owned()),
                                Value::String("1mb".to_owned()),
                            );
                            map
                        }),
                    );
                    map.insert(
                        Value::String("roller".to_owned()),
                        Value::Map({
                            let mut map = BTreeMap::new();
                            map.insert(
                                Value::String("kind".to_owned()),
                                Value::String("fixed_window".to_owned()),
                            );
                            map.insert(Value::String("base".to_owned()), Value::I32(1));
                            map.insert(Value::String("count".to_owned()), Value::I32(5));
                            map.insert(
                                Value::String("pattern".to_owned()),
                                Value::String("logs/test.{}.log".to_owned()),
                            );
                            map
                        }),
                    );
                    map
                }),
            },
        };

        let deserializer = RollingFileAppenderDeserializer;

        let res = deserializer.deserialize(append_cfg, &Deserializers::default());
        assert!(res.is_ok());
    }

    #[test]
    fn test_logwriter() {
        // Can't use named or unnamed temp file here because of opening
        // the file multiple times for reading
        let file = tempfile::tempdir().unwrap();
        let file_path = file.path().join("writer.log");
        let file = File::create(&file_path).unwrap();
        let buf_writer = BufWriter::new(file);
        let mut log_writer = LogWriter {
            file: buf_writer,
            len: 0,
        };

        let contents = fs::read_to_string(&file_path).unwrap();
        assert!(contents.is_empty());
        assert_eq!(log_writer.write(b"test").unwrap(), 4);
        assert!(log_writer.flush().is_ok());
        let contents = fs::read_to_string(file_path).unwrap();
        assert!(contents.contains("test"));
    }
}
