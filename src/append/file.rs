//! The file appender.
//!
//! Requires the `file_appender` feature.

use log::Record;
use parking_lot::Mutex;
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use failure::Error;

#[cfg(feature = "config_parsing")]
use crate::config_parsing::{Deserialize, Deserializers};
#[cfg(feature = "config_parsing")]
use crate::encode::EncoderConfig;

use crate::{
    append::Append,
    encode::{pattern::PatternEncoder, writer::simple::SimpleWriter, Encode},
};

/// The file appender's configuration.
#[cfg(feature = "config_parsing")]
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileAppenderConfig {
    path: String,
    encoder: Option<EncoderConfig>,
    append: Option<bool>,
}

/// An appender which logs to a file.
pub struct FileAppender {
    path: PathBuf,
    file: Mutex<SimpleWriter<BufWriter<File>>>,
    encoder: Box<dyn Encode>,
}

impl fmt::Debug for FileAppender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("FileAppender")
            .field("file", &self.path)
            .field("encoder", &self.encoder)
            .finish()
    }
}

impl Append for FileAppender {
    fn append(&self, record: &Record) -> Result<(), Error> {
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
pub struct FileAppenderDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for FileAppenderDeserializer {
    type Trait = dyn Append;

    type Config = FileAppenderConfig;

    fn deserialize(
        &self,
        config: FileAppenderConfig,
        deserializers: &Deserializers,
    ) -> Result<Box<Self::Trait>, Error> {
        let mut appender = FileAppender::builder();
        if let Some(append) = config.append {
            appender = appender.append(append);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }
        Ok(Box::new(appender.build(&config.path)?))
    }
}

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
