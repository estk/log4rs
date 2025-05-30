//! In this example, to better showcase how customizations work,
//! we’ll intentionally include some rather “odd” features — don’t take them too seriously!
//! This logger will:
//! - Have a custom filter that excludes all log messages at the info level.
//! - Have a custom encoder that adds a prefix to each log entry.
//! - Print messages at the trace and debug levels to the console.
//! - Write messages at the warning and error levels to a file.

use derive_more::Debug;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    sync::Mutex,
};

use log4rs::{
    append::Append,
    config::{Appender, Root},
    encode::{
        writer::{console::ConsoleWriter, simple::SimpleWriter},
        Encode,
    },
    filter::{Filter, Response},
};

/// A custom filter that excludes log messages at a specific level.
#[derive(Debug)]
struct MyFilter {
    level: log::LevelFilter,
}

impl MyFilter {
    fn new(level: log::LevelFilter) -> Self {
        MyFilter { level }
    }
}

impl Filter for MyFilter {
    fn filter(&self, record: &log::Record) -> Response {
        // Exclude all log messages at the info level
        if record.level() == self.level {
            return Response::Reject;
        }
        Response::Accept
    }
}

/// A custom encoder that adds a prefix to each log entry.
#[derive(Debug)]
struct MyEncoder {
    prefix: String,
}

impl MyEncoder {
    fn new(prefix: &str) -> Self {
        MyEncoder {
            prefix: prefix.to_string(),
        }
    }
}

impl Encode for MyEncoder {
    fn encode(
        &self,
        w: &mut dyn log4rs::encode::Write,
        record: &log::Record,
    ) -> anyhow::Result<()> {
        // Write the prefix followed by the log message
        writeln!(
            w,
            "{}{} - {}",
            self.prefix,
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z"),
            record.args()
        )?;
        Ok(())
    }
}

/// A custom appender that writes to both console and file based on log level.
#[derive(Debug)]
struct MyAppender {
    #[debug(skip)]
    console_writer: ConsoleWriter,
    file_writer: Mutex<SimpleWriter<BufWriter<File>>>,
    encoder: Box<dyn Encode>,
}

impl MyAppender {
    fn new(file_name: &str, encoder: Box<dyn Encode>) -> Self {
        let console_writer = ConsoleWriter::stderr().unwrap();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)
            .expect("Failed to open log file");
        let file_writer = Mutex::new(SimpleWriter(BufWriter::new(file)));

        MyAppender {
            console_writer,
            file_writer,
            encoder,
        }
    }
}

impl Append for MyAppender {
    fn append(&self, record: &log::Record) -> anyhow::Result<()> {
        match record.level() {
            log::Level::Trace | log::Level::Debug => {
                let mut writer = self.console_writer.lock();
                self.encoder.encode(&mut writer, record)?;
                writer.flush()?;
            }
            log::Level::Warn | log::Level::Error => {
                let mut writer = self.file_writer.lock().unwrap();
                self.encoder.encode(&mut *writer, record)?;
                writer.flush()?;
            }
            _ => panic!("Unexpected log level"),
        };
        Ok(())
    }
    fn flush(&self) {}
}

fn init_logger() {
    let encoder = MyEncoder::new("[MyApp] ");
    let appender = MyAppender::new("log.txt", Box::new(encoder));
    let filter = MyFilter::new(log::LevelFilter::Info);

    let log_config = log4rs::config::Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(filter))
                .build("my_appender", Box::new(appender)),
        )
        .build(
            Root::builder()
                // Set the root logger to use the custom appender
                .appender("my_appender")
                .build(log::LevelFilter::Trace),
        )
        .unwrap();
    log4rs::init_config(log_config).unwrap();
}

fn main() {
    init_logger();

    log::trace!("This is a trace message");
    log::debug!("This is a debug message");
    log::info!("This is an info message");
    log::warn!("This is a warning message");
    log::error!("This is an error message");
}
