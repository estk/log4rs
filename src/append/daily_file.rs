//! The daily file appender.
//!
//! Requires the `daily_file_appender` feature.

use chrono::{Local, Date};
use derivative::Derivative;
use log::Record;
use parking_lot::Mutex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{PathBuf},
};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};
#[cfg(feature = "config_parsing")]
use crate::encode::EncoderConfig;

use crate::{
    append::Append,
    encode::{pattern::PatternEncoder, writer::simple::SimpleWriter, Encode},
};

/// The daily_file appender's configuration.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DailyFileAppenderConfig {
    pattern: String,
    encoder: Option<EncoderConfig>,
    append: Option<bool>,
}

/// Wrap file_name with it's name
pub struct FileWithName {
    /// file handle
    pub file_handle: SimpleWriter<BufWriter<File>>,
    /// file name
    pub name: String,
}

/// An appender which logs to a file.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct DailyFileAppender {
    pattern: String,
    path: PathBuf,
    #[derivative(Debug = "ignore")]
    file: Mutex<FileWithName>,
    encoder: Box<dyn Encode>,
}

impl Append for DailyFileAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        let mut file = self.file.lock();

        // check if day changes
        let today: Date<Local> = Local::today();
        let today_str = today.format("%Y-%m-%d").to_string();
        let filename = self.pattern.replace("{}", &today_str);
        if file.name != filename {
            let mut path_buf = PathBuf::new();
            path_buf.push(filename.clone());

            let path = super::env_util::expand_env_vars(path_buf);

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            // open new file
            let new_file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&filename)?;
            file.file_handle = SimpleWriter(BufWriter::with_capacity(1024, new_file))
        }

        self.encoder.encode(&mut file.file_handle, record)?;
        file.file_handle.flush()?;
        Ok(())
    }

    fn flush(&self) {}
}

impl DailyFileAppender {
    /// Creates a new `DailyFileAppender` builder.
    pub fn builder() -> DailyFileAppenderBuilder {
        DailyFileAppenderBuilder {
            encoder: None,
        }
    }
}

/// A builder for `DailyFileAppender`s.
pub struct DailyFileAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
}

impl DailyFileAppenderBuilder {
    /// Sets the output encoder for the `DilyFileAppender`.
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> DailyFileAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// Consumes the `DailyFileAppenderBuilder`, producing a `DailyFileAppender`.
    /// The path argument can contain environment variables of the form $ENV{name_here},
    /// where 'name_here' will be the name of the environment variable that
    /// will be resolved. Note that if the variable fails to resolve,
    /// $ENV{name_here} will NOT be replaced in the path.
    pub fn build(self, pattern: &str) -> io::Result<DailyFileAppender> {
        let today: Date<Local> = Local::today();
        let today_str = today.format("%Y-%m-%d").to_string();
        let filename = pattern.replace("{}", &today_str);

        let mut path_buf = PathBuf::new();
        path_buf.push(filename.clone());

        let path = super::env_util::expand_env_vars(path_buf);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&filename)?;

        Ok(DailyFileAppender {
            pattern: String::from(pattern),
            path: path,
            file: Mutex::new(FileWithName {
                file_handle: SimpleWriter(BufWriter::with_capacity(1024, file)),
                name: filename,
            }),
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
        })
    }
}

/// A deserializer for the `DailyFileAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: daily_file
///
/// # The path of the log file. Required.
/// # {} will be replaced by date YYYY.MM.DD
/// pattern: log/foo_{}_.log
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct DailyFileAppenderDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for DailyFileAppenderDeserializer {
    type Trait = dyn Append;

    type Config = DailyFileAppenderConfig;

    fn deserialize(
        &self,
        config: DailyFileAppenderConfig,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let mut appender = DailyFileAppender::builder();
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }
        Ok(Box::new(appender.build(&config.pattern)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_directories() {
        let tempdir = tempfile::tempdir().unwrap();
        let mut path_buf = PathBuf::new();
        path_buf.push(tempdir);
        path_buf.push("foo_{}_.log");

        DailyFileAppender::builder()
            .build(&path_buf.as_path().to_str().unwrap())
            .unwrap();
    }
}
