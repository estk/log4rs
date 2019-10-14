#[macro_use]
extern crate log;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::json::JsonEncoder;

//appenders:
//    stdout:
//        kind: console
//        encoder:
//            kind: json
//root:
//    level: info
//    appenders:
//        - stdout
//The above YAML is the same as the programatically built logger below.

fn main() {
    let stdout: ConsoleAppender = ConsoleAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build();
    let log_config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(log_config).unwrap();

    info!("Info log!");
    warn!("Warn log with value {}", "test");
    error!("ERROR!");
}
