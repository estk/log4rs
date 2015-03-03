use std::default::Default;
use std::io;
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufWriter, Stdout};
use std::fs::{File, OpenOptions};
use std::path::{AsPath, PathBuf};
use log::LogRecord;

use Append;
use pattern::PatternLayout;

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
    pub fn builder<P: AsPath + ?Sized>(path: &P) -> FileAppenderBuilder {
        FileAppenderBuilder {
            path: path.as_path().to_path_buf(),
            pattern: Default::default(),
        }
    }
}

pub struct FileAppenderBuilder {
    path: PathBuf,
    pattern: PatternLayout,
}

impl FileAppenderBuilder {
    pub fn pattern(mut self, pattern: PatternLayout) -> FileAppenderBuilder {
        self.pattern = pattern;
        self
    }

    pub fn build(self) -> io::Result<FileAppender> {
        let file = try!(OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path));

        Ok(FileAppender {
            file: BufWriter::with_capacity(1024, file),
            pattern: self.pattern
        })
    }
}

pub struct ConsoleAppender {
    stdout: Stdout,
    pattern: PatternLayout,
}

impl Append for ConsoleAppender {
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<Error>> {
        let mut stdout = self.stdout.lock();
        try!(self.pattern.append(&mut stdout, record));
        try!(stdout.flush());
        Ok(())
    }
}

impl ConsoleAppender {
    pub fn builder() -> ConsoleAppenderBuilder {
        ConsoleAppenderBuilder {
            pattern: Default::default(),
        }
    }
}

pub struct ConsoleAppenderBuilder {
    pattern: PatternLayout,
}

impl ConsoleAppenderBuilder {
    pub fn pattern(mut self, pattern: PatternLayout) -> ConsoleAppenderBuilder {
        self.pattern = pattern;
        self
    }

    pub fn build(self) -> ConsoleAppender {
        ConsoleAppender {
            stdout: io::stdout(),
            pattern: self.pattern,
        }
    }
}
