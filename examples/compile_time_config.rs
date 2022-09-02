use log::{error, info, trace};
use log4rs::{self, append::LocalAppender, filter::LocalFilter};


fn main() {
    let config_str = include_str!("sample_config.yml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config::<LocalAppender, LocalFilter>(config).unwrap();

    info!("Goes to console");
    error!("Goes to console");
    trace!("Doesn't go to console as it is filtered out");
}
