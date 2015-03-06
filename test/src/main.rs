#![feature(std_misc, old_io)]

#[macro_use]
extern crate log;
extern crate log4rs;

use std::default::Default;
use std::old_io::timer::sleep;
use std::time::Duration;

fn main() {
    log4rs::init_file("log.toml", Default::default()).unwrap();

    loop {
        sleep(Duration::seconds(1));
        warn!("main");
        error!("error main");
        a::foo();
    }
}

mod a {
    pub fn foo() {
        info!("a");
    }
}
