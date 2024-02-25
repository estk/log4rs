#[test]
#[cfg(all(feature = "config_parsing", feature = "yaml_format"))]
fn test_malformed_appenders() {
    use std::fs;

    let config_str = fs::read_to_string("test_cfgs/malformed_appender.yml").unwrap();
    let cfg = ::serde_yaml::from_str::<log4rs::config::RawConfig>(&config_str);

    assert!(cfg.is_ok());
    let cfg = cfg.unwrap();

    let res = log4rs::config::create_raw_config(cfg);
    assert!(res.is_err());
}
