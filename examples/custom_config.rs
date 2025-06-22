//! In this example, we will define the Appender, Encoder and Filter using the same code as in custom.rs.
//! However, unlike before, our logger will be initialized from a configuration file.

use derive_more::Debug;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    sync::Mutex,
};

use log4rs::{
    append::Append,
    config::{Deserialize, Deserializers},
    encode::{
        writer::{console::ConsoleWriter, simple::SimpleWriter},
        Encode, EncoderConfig,
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
            _ => panic!("Invalid log level"),
        };
        Ok(())
    }
    fn flush(&self) {}
}

/// The code above is same as `examples/custom.rs`.
/// Next, we'll define the custom deserializer to handle the configuration file.
/// To parse the configuration file, we need a struct that matches the structure
/// of the configuration, as well as a deserializer to generate the corresponding
/// config struct from the file.

#[derive(serde::Deserialize)]
struct MyAppenderConfig {
    file_name: String,
    encoder: EncoderConfig,
}
struct MyAppenderDeserializer;

#[derive(serde::Deserialize)]
struct MyEncoderConfig {
    prefix: String,
}
struct MyEncoderDeserializer;

#[derive(serde::Deserialize)]
struct MyFilterConfig {
    level: log::LevelFilter,
}
struct MyFilterDeserializer;

impl Deserialize for MyAppenderDeserializer {
    type Config = MyAppenderConfig;
    type Trait = dyn Append;

    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &log4rs::config::Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let file_name = config.file_name;
        let encoder_cfg = config.encoder;
        let encoder: Box<dyn Encode> = deserializers
            .deserialize(&encoder_cfg.kind, encoder_cfg.config)
            .unwrap();
        let appender = MyAppender::new(&file_name, encoder);
        Ok(Box::new(appender))
    }
}

impl Deserialize for MyEncoderDeserializer {
    type Config = MyEncoderConfig;
    type Trait = dyn Encode;

    fn deserialize(
        &self,
        config: Self::Config,
        _: &log4rs::config::Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let prefix = config.prefix;
        Ok(Box::new(MyEncoder::new(&prefix)))
    }
}

impl Deserialize for MyFilterDeserializer {
    type Config = MyFilterConfig;
    type Trait = dyn Filter;

    fn deserialize(
        &self,
        config: Self::Config,
        _: &log4rs::config::Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let level = config.level;
        Ok(Box::new(MyFilter::new(level)))
    }
}

fn init_logger() {
    // At last, we need register our deserializer to the default deserializers.
    let mut deserializers = Deserializers::default();
    deserializers.insert("my_appender", MyAppenderDeserializer);
    deserializers.insert("my_encoder", MyEncoderDeserializer);
    deserializers.insert("my_filter", MyFilterDeserializer);

    log4rs::init_file("examples/custom_config.yml", deserializers).unwrap();
}

fn main() {
    init_logger();

    log::trace!("This is a trace message");
    log::debug!("This is a debug message");
    log::info!("This is an info message");
    log::warn!("This is a warning message");
    log::error!("This is an error message");
}
