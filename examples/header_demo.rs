//! Demonstrates the log file header feature.
//!
//! Run with: cargo run --example header_demo --features file_appender

use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use log::{info, LevelFilter};

fn main() {
    // Create a file appender with a header
    let header_text = "=== Log File Started at $TIME{%Y-%m-%d %H:%M:%S} ===\n\
                       === User: $ENV{USER} ===";

    let file_appender = FileAppender::builder()
        .header(header_text)
        .encoder(Box::new(PatternEncoder::new("{d} [{l}] {m}{n}")))
        .build("/tmp/log4rs_header_demo.log")
        .expect("Failed to create file appender");

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(Root::builder().appender("file").build(LevelFilter::Info))
        .expect("Failed to build config");

    log4rs::init_config(config).expect("Failed to initialize log4rs");

    // Write some log messages
    info!("This is the first log message");
    info!("This is the second log message");
    info!("This is the third log message");

    println!("\n✓ Logs written to /tmp/log4rs_header_demo.log");
    println!("\nFile contents:");
    println!("─────────────────────────────────────");
    let contents =
        std::fs::read_to_string("/tmp/log4rs_header_demo.log").expect("Failed to read log file");
    println!("{}", contents);
    println!("─────────────────────────────────────");
}
