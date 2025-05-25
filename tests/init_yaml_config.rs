#[test]
#[cfg(all(feature = "config_parsing", feature = "yaml_format"))]
fn test_init_yaml_cfg() {
    use log4rs;
    use std::path::Path;

    assert!(log4rs::init_file(
        Path::new("./test_cfgs/test.yml"),
        log4rs::config::Deserializers::default()
    )
    .is_ok());
}
