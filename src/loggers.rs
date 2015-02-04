//! A set of basic log4rs loggers.

use std::borrow::ToOwned;
use std::old_io::{FileMode, FileAccess, LineBufferedWriter};
use std::old_io::fs::File;
use std::old_io::stdio;

use toml::{self, Value};
use log::LogRecord;

use {Log, MakeLogger};
use pattern::PatternLogger;

struct BasicLogger<W> where W: Writer+Send {
    w: W,
    p: PatternLogger,
}

impl<W> Log for BasicLogger<W> where W: Writer+Send {
    fn log(&mut self, record: &LogRecord) {
        self.p.log(&mut self.w, record)
    }
}

/// A logger maker returning a log4rs logger which logs to `stderr`.
///
/// The log output format may be customized via the `pattern` parameter. The
/// accepted syntax is documented in the `pattern` module.
///
/// The `default_loggers` function maps the string "console" to this logger
/// maker.
///
/// # Examples
///
/// ```toml
/// [[logger.console]]
/// kind = "console"
/// pattern = "%d [%l] %M - %m"
/// ```
#[derive(Debug)]
pub struct ConsoleLoggerMaker;

impl MakeLogger for ConsoleLoggerMaker {
    fn make_logger(&self, config: &toml::Table) -> Result<Box<Log>, String> {
        let pattern = try!(PatternLogger::from_config(config));

        Ok(Box::new(BasicLogger {
            w: stdio::stderr(),
            p: pattern,
        }) as Box<Log>)
    }
}

/// A logger maker returning a log4rs logger which logs to a specified file.
///
/// The file is specified by the `file` parameter. The log output format may be
/// customized via the `pattern` parameter. The accepted syntax is documented
/// in the `pattern` module.
///
/// The `default_loggers` function maps the string "file" to this logger maker.
///
/// # Examples
///
/// ```toml
/// [[logger.errors]]
/// kind = "file"
/// file = "log/errors.log"
/// pattern = "%d [%l] %M - %m"
/// ```
#[derive(Debug)]
pub struct FileLoggerMaker;

impl MakeLogger for FileLoggerMaker {
    fn make_logger(&self, config: &toml::Table) -> Result<Box<Log>, String> {
        let file = match config.get("file") {
            Some(&Value::String(ref file)) => Path::new(file),
            Some(_) => return Err("`file` must be a string".to_owned()),
            None => return Err("Missing required key `file`".to_owned()),
        };
        let file = match File::open_mode(&file, FileMode::Append, FileAccess::Write) {
            Ok(file) => LineBufferedWriter::new(file),
            Err(err) => return Err(err.to_string()),
        };
        let pattern = try!(PatternLogger::from_config(config));

        Ok(Box::new(BasicLogger {
            w: file,
            p: pattern,
        }) as Box<Log>)
    }
}
