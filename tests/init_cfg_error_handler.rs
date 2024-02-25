#[test]
#[cfg(all(feature = "config_parsing", feature = "yaml_format"))]
fn test_cfg_err_hdlr() {
    use std::{
        io::{self, Write},
        path::Path,
    };

    let cfg = log4rs::config::load_config_file(
        Path::new("./test_cfgs/test.yml"),
        log4rs::config::Deserializers::default(),
    );
    assert!(cfg.is_ok());
    let cfg = cfg.unwrap();

    let res = log4rs::config::init_config_with_err_handler(
        cfg,
        Box::new(|e: &anyhow::Error| {
            let _ = writeln!(io::stderr(), "log4rs: {}", e);
        }),
    );
    assert!(res.is_ok());
}
