#[test]
#[cfg(all(feature = "config_parsing", feature = "json_format"))]
fn test_init_json_cfg() {
    use log4rs;
    use std::path::Path;

    assert!(log4rs::init_file(
        Path::new("./test_cfgs/test.json"),
        log4rs::config::Deserializers::default()
    )
    .is_ok());
}
