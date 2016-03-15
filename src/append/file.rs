//! The file appender.

use std::error::Error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use log::LogRecord;
use serde_value::Value;

use append::{Append, SimpleWriter};
use encode::Encode;
use encode::pattern::PatternEncoder;
use file::{Build, Builder};
use file::raw::Encoder;

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
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>> {
        let mut file = self.file.lock().unwrap_or_else(|e| e.into_inner());
        try!(self.encoder.encode(&mut *file, record));
        try!(file.flush());
        Ok(())
    }
}

impl FileAppender {
    /// Creates a new `FileAppender` builder for an appender which will log to
    /// a file at the provided path.
    pub fn builder<P: AsRef<Path>>(path: P) -> FileAppenderBuilder {
        FileAppenderBuilder {
            path: path.as_ref().to_path_buf(),
            encoder: Box::new(PatternEncoder::default()),
            append: true,
        }
    }
}

/// A builder for `FileAppender`s.
pub struct FileAppenderBuilder {
    path: PathBuf,
    encoder: Box<Encode>,
    append: bool,
}

impl FileAppenderBuilder {
    /// Sets the output encoder for the `FileAppender`.
    pub fn encoder(mut self, encoder: Box<Encode>) -> FileAppenderBuilder {
        self.encoder = encoder;
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
    pub fn build(self) -> io::Result<FileAppender> {
        let file = try!(OpenOptions::new()
                            .write(true)
                            .append(self.append)
                            .create(true)
                            .open(&self.path));

        Ok(FileAppender {
            path: self.path,
            file: Mutex::new(SimpleWriter(BufWriter::with_capacity(1024, file))),
            encoder: self.encoder,
        })
    }
}


/// A deserializer for the `FileAppender`.
///
/// The `path` key is required, and specifies the path to the log file. The
/// `pattern` key is optional and specifies a `PatternEncoder` pattern to be
/// used for output. The `append` key is optional and specifies whether the
/// output file should be truncated or appended to.
pub struct FileAppenderDeserializer;

impl Build for FileAppenderDeserializer {
    type Trait = Append;

    fn build(&self, config: Value, builder: &Builder) -> Result<Box<Append>, Box<Error>> {
        let config = try!(config.deserialize_into::<FileAppenderConfig>());
        let mut appender = FileAppender::builder(&config.path);
        if let Some(append) = config.append {
            appender = appender.append(append);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(try!(builder.build("encoder",
                                                           &encoder.kind,
                                                           encoder.config)));
        }
        Ok(Box::new(try!(appender.build())))
    }
}

include!("file_serde.rs");
