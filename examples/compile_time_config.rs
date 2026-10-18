use log::{error, info, trace};

fn main() {
    let config_str = include_str!("sample_config.yml");
    let config = serde_saphyr::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();

    info!("Goes to console, file and rolling file");
    error!("Goes to console, file and rolling file");
    trace!("Doesn't go to console as it is filtered out");
}
