//! The file appender.
//!
//! Requires the `file_appender` feature.

use derivative::Derivative;
use log::Record;
use parking_lot::Mutex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

#[cfg(feature = "config_parsing")]
use crate::config::{raw::DeserializingConfigError, runtime::IntoAppender};
#[cfg(feature = "config_parsing")]
use crate::encode::{EncoderConfig, IntoEncode};

use crate::{
    append::Append,
    encode::{pattern::PatternEncoder, writer::simple::SimpleWriter, Encode},
};

/// The file appender's configuration.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileAppenderConfig {
    path: String,
    encoder: Option<EncoderConfig>,
    append: Option<bool>,
}

#[cfg(feature = "config_parsing")]
impl IntoAppender for FileAppenderConfig {
    fn into_appender(
        self,
        build: crate::config::runtime::AppenderBuilder,
        name: String,
    ) -> Result<crate::config::Appender, DeserializingConfigError> {
        let mut appender = FileAppender::builder();
        if let Some(encoder) = self.encoder {
            appender = appender.encoder(
                encoder
                    .into_encode()
                    .map_err(|e| DeserializingConfigError::Appender(name.clone(), e))?,
            );
        };
        if let Some(append) = self.append {
            appender = appender.append(append);
        };
        let n = name.clone();
        let appender = appender
            .build(self.path)
            .map_err(move |e| DeserializingConfigError::Appender(n, e.into()))?;
        Ok(build.build(name, Box::new(appender)))
    }
}

/// An appender which logs to a file.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct FileAppender {
    path: PathBuf,
    #[derivative(Debug = "ignore")]
    file: Mutex<SimpleWriter<BufWriter<File>>>,
    encoder: Box<dyn Encode>,
}

impl Append for FileAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        let mut file = self.file.lock();
        self.encoder.encode(&mut *file, record)?;
        file.flush()?;
        Ok(())
    }

    fn flush(&self) {}
}

impl FileAppender {
    /// Creates a new `FileAppender` builder.
    pub fn builder() -> FileAppenderBuilder {
        FileAppenderBuilder {
            encoder: None,
            append: true,
        }
    }
}

/// A builder for `FileAppender`s.
pub struct FileAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    append: bool,
}

impl FileAppenderBuilder {
    /// Sets the output encoder for the `FileAppender`.
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> FileAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// Determines if the appender will append to or truncate the output file.
    ///
    /// Defaults to `true`.
    pub fn append(mut self, append: bool) -> FileAppenderBuilder {
        self.append = append;
        self
    }

    /// Consumes the `FileAppenderBuilder`, producing a `FileAppender`.
    /// The path argument can contain environment variables of the form $ENV{name_here},
    /// where 'name_here' will be the name of the environment variable that
    /// will be resolved. Note that if the variable fails to resolve,
    /// $ENV{name_here} will NOT be replaced in the path.
    pub fn build<P: AsRef<Path>>(self, path: P) -> io::Result<FileAppender> {
        let path = super::env_util::expand_env_vars(path.as_ref().to_path_buf());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .write(true)
            .append(self.append)
            .truncate(!self.append)
            .create(true)
            .open(&path)?;

        Ok(FileAppender {
            path,
            file: Mutex::new(SimpleWriter(BufWriter::with_capacity(1024, file))),
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
        })
    }
}

/// A deserializer for the `FileAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: file
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
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct FileAppenderDeserializer;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_directories() {
        let tempdir = tempfile::tempdir().unwrap();

        FileAppender::builder()
            .build(tempdir.path().join("foo").join("bar").join("baz.log"))
            .unwrap();
    }

    #[test]
    fn append_false() {
        let tempdir = tempfile::tempdir().unwrap();
        FileAppender::builder()
            .append(false)
            .build(tempdir.path().join("foo.log"))
            .unwrap();
    }
}
