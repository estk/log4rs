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
use pattern::PatternLayout;

/// An appender which logs to a file.
pub struct FileAppender {
    file: BufWriter<File>,
    pattern: PatternLayout,
}

impl Append for FileAppender {
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<Error>> {
        try!(self.pattern.append(&mut self.file, record));
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
            pattern: Default::default(),
            append: true,
        }
    }
}

/// A builder for `FileAppender`s.
pub struct FileAppenderBuilder {
    path: PathBuf,
    pattern: PatternLayout,
    append: bool,
}

impl FileAppenderBuilder {
    /// Sets the output pattern for the `FileAppender`.
    pub fn pattern(mut self, pattern: PatternLayout) -> FileAppenderBuilder {
        self.pattern = pattern;
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
            pattern: self.pattern
        })
    }
}

/// An appender which logs to stdout.
pub struct ConsoleAppender {
    stdout: Stdout,
    pattern: PatternLayout,
}

impl Append for ConsoleAppender {
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<Error>> {
        let mut stdout = self.stdout.lock();
        try!(self.pattern.append_console(&mut stdout, record));
        try!(stdout.flush());
        Ok(())
    }
}

impl ConsoleAppender {
    /// Creates a new `ConsoleAppender` builder.
    pub fn builder() -> ConsoleAppenderBuilder {
        ConsoleAppenderBuilder {
            pattern: Default::default(),
        }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    pattern: PatternLayout,
}

impl ConsoleAppenderBuilder {
    /// Sets the output pattern for the `ConsoleAppender`.
    pub fn pattern(mut self, pattern: PatternLayout) -> ConsoleAppenderBuilder {
        self.pattern = pattern;
        self
    }

    /// Consumes the `ConsoleAppenderBuilder`, producing a `ConsoleAppender`.
    pub fn build(self) -> ConsoleAppender {
        ConsoleAppender {
            stdout: io::stdout(),
            pattern: self.pattern,
        }
    }
}
