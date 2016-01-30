//! A set of common appenders

use std::convert::AsRef;
use std::default::Default;
use std::io;
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufWriter, Stdout};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use log::LogRecord;

use Append;
use encoder::Encoder;

/// An appender which logs to a file.
pub struct FileAppender {
    file: BufWriter<File>,
    encoder: Encoder,
}

impl Append for FileAppender {
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<Error>> {
        try!(self.encoder.append(&mut self.file, record));
        try!(self.file.flush());
        Ok(())
    }
}

impl FileAppender {
    /// Creates a new `FileAppender` builder for an appender which will log to
    /// a file at the provided path.
    pub fn builder<P: AsRef<Path>>(path: P) -> FileAppenderBuilder {
        FileAppenderBuilder {
            path: path.as_ref().to_path_buf(),
            encoder: Default::default(),
            append: true,
        }
    }
}

/// A builder for `FileAppender`s.
pub struct FileAppenderBuilder {
    path: PathBuf,
    encoder: Encoder,
    append: bool,
}

impl FileAppenderBuilder {
    /// Sets the output encoder for the `FileAppender`.
    pub fn encoder(mut self, encoder: Encoder) -> FileAppenderBuilder {
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
            file: BufWriter::with_capacity(1024, file),
            encoder: self.encoder,
        })
    }
}

/// An appender which logs to stdout.
pub struct ConsoleAppender {
    stdout: Stdout,
    encoder: Encoder,
}

impl Append for ConsoleAppender {
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<Error>> {
        let mut stdout = self.stdout.lock();
        try!(self.encoder.append(&mut stdout, record));
        try!(stdout.flush());
        Ok(())
    }
}

impl ConsoleAppender {
    /// Creates a new `ConsoleAppender` builder.
    pub fn builder() -> ConsoleAppenderBuilder {
        ConsoleAppenderBuilder { encoder: Default::default() }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    encoder: Encoder,
}

impl ConsoleAppenderBuilder {
    /// Sets the output encoder for the `ConsoleAppender`.
    pub fn encoder(mut self, encoder: Encoder) -> ConsoleAppenderBuilder {
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
