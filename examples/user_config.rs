use log::{error, info, trace};
use log4rs::{
    append::{console::ConsoleAppender, LocalOrUserAppender},
    config::runtime::{AppenderBuilder, IntoAppender},
    encode::{EncoderConfig, IntoEncode},
    filter::LocalFilter,
};

use log4rs::config::Appender;

#[derive(Clone, Debug, serde::Deserialize, Default)]
struct MyConfig {
    encoder: EncoderConfig,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(tag = "kind")]
enum UserAppender {
    #[serde(rename = "my_t")]
    T(MyConfig),
}

impl Default for UserAppender {
    fn default() -> Self {
        UserAppender::T(Default::default())
    }
}

impl IntoAppender for UserAppender {
    fn into_appender(
        self,
        build: AppenderBuilder,
        name: String,
    ) -> Result<Appender, log4rs::config::DeserializingConfigError> {
        let mut appender = ConsoleAppender::builder();
        match self {
            UserAppender::T(c) => {
                let r = c.encoder.into_encode();
                appender = appender.encoder(r)
            }
        }
        Ok(build.build(name, Box::new(appender.build())))
    }
}

type MyAppender = LocalOrUserAppender<UserAppender>;

fn main() {
    let config_str = r#"
appenders:
    stdout:
        kind: my_t
        encoder:
            kind: pattern
            pattern: "{d(%+)(utc)} [{f}:{L}] {h({l})} {M}:{m}{n}"

root:
    level: info
    appenders:
        - stdout
"#;
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config::<MyAppender, LocalFilter>(config).unwrap();

    info!("Goes to console");
    error!("Goes to console");
    trace!("Doesn't go to console as it is filtered out");
}
