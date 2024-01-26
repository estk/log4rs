use std::{default::Default, thread, time::Duration};

use log::{error, warn};
use log4rs;

fn main() {
    log4rs::init_file("examples/multi_logger.yml", Default::default()).unwrap();

    loop {
        thread::sleep(Duration::from_secs(1));
        warn!("main");
        error!("error main");
        a::foo();
    }
}

mod a {
    use log::info;

    pub fn foo() {
        info!("module a");
    }
}
