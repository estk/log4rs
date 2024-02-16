use log::{error, info};
use log4rs;
use serde_yaml;
use std::env;

fn main() {
    let config_str = include_str!("sample_config.yml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();

    let no_color = match env::var("NO_COLOR") {
        Ok(no_color) => no_color,
        Err(_) => "0".to_string(),
    };
    let clicolor_force = match env::var("CLICOLOR_FORCE") {
        Ok(clicolor_force) => clicolor_force,
        Err(_) => "0".to_string(),
    };
    let cli_color = match env::var("CLICOLOR") {
        Ok(cli_color) => cli_color,
        Err(_) => "0".to_string(),
    };
    info!("NO_COLOR: {}, CLICOLOR_FORCE: {}, CLICOLOR: {}", no_color, clicolor_force, cli_color);
    error!("NO_COLOR: {}, CLICOLOR_FORCE: {}, CLICOLOR: {}", no_color, clicolor_force, cli_color);
}
