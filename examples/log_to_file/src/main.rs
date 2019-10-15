// cargo install cargo-edit
// cargo add log4rs log

use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;

fn main() {
    // You probably want to pass this as a parameter.
    let level = log::LevelFilter::Info;
    // You probably want to pass this as a parameter.
    let file_path = "/tmp/foo.log";

    // Logging to stderr
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path)
        .unwrap();

    // Log Trace output to file, and the user-specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    let _handle = log4rs::init_config(config).unwrap();

    error!("Goes to stderr and file");
    warn!("Goes to stderr and file");
    info!("Goes to stderr and file");
    debug!("Goes to file only");
    trace!("Goes to file only");
}
