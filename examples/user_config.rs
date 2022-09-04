use log::{error, info, trace};
use log4rs::config::Appender;
use log4rs::filter::threshold::ThresholdFilterConfig;
use log4rs::{
    append::{console::ConsoleAppender, LocalOrUserAppender},
    config::runtime::{AppenderBuilder, IntoAppender},
    encode::{EncoderConfig, IntoEncode},
    filter::{IntoFilter, LocalOrUserFilter},
};

#[derive(Clone, Debug, serde::Deserialize)]
struct MyConfig {
    encoder: EncoderConfig,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(tag = "kind")]
enum UserAppender {
    #[serde(rename = "my_t")]
    T(MyConfig),
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
                let r = c.encoder.into_encode().map_err(|e| {
                    log4rs::config::DeserializingConfigError::Appender(name.clone(), e)
                })?;
                appender = appender.encoder(r)
            }
        }
        Ok(build.build(name, Box::new(appender.build())))
    }
}

#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum UserFilter {
    ///
    #[serde(rename = "user_threshold")]
    ThresholdFilter(ThresholdFilterConfig),
}

impl IntoFilter for UserFilter {
    fn into_filter(self) -> anyhow::Result<Box<dyn log4rs::filter::Filter>> {
        match self {
            UserFilter::ThresholdFilter(t) => t.into_filter(),
        }
    }
}

type MyAppender = LocalOrUserAppender<UserAppender>;
type MyFilter = LocalOrUserFilter<UserFilter>;

fn main() {
    let config_str = r#"
appenders:
    stdout:
        kind: my_t
        encoder:
            kind: pattern
            pattern: "{d(%+)(utc)} [{f}:{L}] {h({l})} {M}:{m}{n}"
        filters: 
            - kind: user_threshold
              level: info

root:
    level: info
    appenders:
        - stdout
"#;
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config::<MyAppender, MyFilter>(config).unwrap();

    info!("Goes to console");
    error!("Goes to console");
    trace!("Doesn't go to console as it is filtered out");
}
