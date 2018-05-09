//! The file appender.
//!
//! Requires the `file_appender` feature.

use antidote::Mutex;
use log::Record;
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use append::Append;
use encode::Encode;
#[cfg(feature = "file")]
use encode::EncoderConfig;
use encode::pattern::PatternEncoder;
use encode::writer::simple::SimpleWriter;
#[cfg(feature = "file")]
use file::{Deserialize, Deserializers};
use record::ExtendedRecord;

/// The file appender's configuration.
#[cfg(feature = "file")]
#[derive(Deserialize)]
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
    encoder: Box<Encode>,
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
    fn append(&self, record: &ExtendedRecord) -> Result<(), Box<Error + Sync + Send>> {
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
    encoder: Option<Box<Encode>>,
    append: bool,
}

impl FileAppenderBuilder {
    /// Sets the output encoder for the `FileAppender`.
    pub fn encoder(mut self, encoder: Box<Encode>) -> FileAppenderBuilder {
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
    pub fn build<P: AsRef<Path>>(self, path: P) -> io::Result<FileAppender> {
        let path = path.as_ref().to_owned();
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
            path: path,
            file: Mutex::new(SimpleWriter(BufWriter::with_capacity(1024, file))),
            encoder: self.encoder
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
#[cfg(feature = "file")]
pub struct FileAppenderDeserializer;

#[cfg(feature = "file")]
impl Deserialize for FileAppenderDeserializer {
    type Trait = Append;

    type Config = FileAppenderConfig;

    fn deserialize(
        &self,
        config: FileAppenderConfig,
        deserializers: &Deserializers,
    ) -> Result<Box<Append>, Box<Error + Sync + Send>> {
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
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn create_directories() {
        let tempdir = TempDir::new("create_directories").unwrap();

        FileAppender::builder()
            .build(tempdir.path().join("foo").join("bar").join("baz.log"))
            .unwrap();
    }

    #[test]
    fn append_false() {
        let tempdir = TempDir::new("append_false").unwrap();
        FileAppender::builder()
            .append(false)
            .build(tempdir.path().join("foo.log"))
            .unwrap();
    }
}
