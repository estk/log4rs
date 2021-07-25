use log::{error, info, trace};
use serde_yaml;
use log4rs;

fn main() {
    let config_str = include_str!("sample_config.yml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();

    info!("Goes to console");
    error!("Goes to console");
    trace!("Doesn't go to console as it is filtered out");
}
