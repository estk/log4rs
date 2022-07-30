use std::default::Default;
use std::thread;
use std::time::Duration;

use log::{error, info, warn};
use log4rs;

fn main() {
    log4rs::init_file("log.yml", Default::default()).unwrap();

    loop {
        thread::sleep(Duration::from_secs(1));
        warn!("main");
        error!("error main");
        a::foo();
    }
}

mod a {
    pub fn foo() {
        use log::info;
        info!("a");
    }
}
