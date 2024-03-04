#[test]
#[cfg(all(feature = "config_parsing", feature = "toml_format"))]
fn test_init_toml_cfg() {
    use log4rs;
    use std::path::Path;

    assert!(log4rs::init_file(
        Path::new("./test_cfgs/test.toml"),
        log4rs::config::Deserializers::default()
    )
    .is_ok());
}
