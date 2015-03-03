use std::default::Default;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
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
    fn append(&mut self, record: &LogRecord) {
        let _ = self.pattern.append(&mut self.file, record);
        let _ = self.file.flush();
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
