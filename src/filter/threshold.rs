//! The threshold filter.
//!
//! Requires the `threshold_filter` feature.

use log::{LevelFilter, Record};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};
use crate::filter::{Filter, Response};

/// The threshold filter's configuration.
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
pub struct ThresholdFilterConfig {
    level: LevelFilter,
}

/// A filter that rejects all events at a level below a provided threshold.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ThresholdFilter {
    level: LevelFilter,
}

impl ThresholdFilter {
    /// Creates a new `ThresholdFilter` with the specified threshold.
    pub fn new(level: LevelFilter) -> ThresholdFilter {
        ThresholdFilter { level }
    }
}

impl Filter for ThresholdFilter {
    fn filter(&self, record: &Record) -> Response {
        if record.level() > self.level {
            Response::Reject
        } else {
            Response::Neutral
        }
    }
}

/// A deserializer for the `ThresholdFilter`.
///
/// # Configuration
///
/// ```yaml
/// kind: threshold
///
/// # The threshold log level to filter at. Required
/// level: warn
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ThresholdFilterDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for ThresholdFilterDeserializer {
    type Trait = dyn Filter;

    type Config = ThresholdFilterConfig;

    fn deserialize(
        &self,
        config: ThresholdFilterConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Filter>> {
        Ok(Box::new(ThresholdFilter::new(config.level)))
    }
}

#[cfg(test)]
mod test {
    use log::{Level, LevelFilter, Record};

    use super::*;

    #[cfg(feature = "config_parsing")]
    use crate::config::Deserializers;

    #[cfg(feature = "config_parsing")]
    use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_cfg_serde() {
        let filter_cfg = ThresholdFilterConfig {
            level: LevelFilter::Off,
        };

        let mut cfg = vec![
            Token::Struct {
                name: "ThresholdFilterConfig",
                len: 1,
            },
            Token::Str("level"),
            Token::Enum {
                name: "LevelFilter",
            },
            Token::Str("Off"),
            Token::Unit,
            Token::StructEnd,
        ];

        assert_de_tokens(&filter_cfg, &cfg);

        cfg[1] = Token::Str("leel");
        assert_de_tokens_error::<ThresholdFilterConfig>(&cfg, "missing field `level`");

        cfg[1] = Token::Str("level");
        cfg[3] = Token::Str("On");
        cfg.remove(4); // No Unit on this one as the Option is invalid
        assert_de_tokens_error::<ThresholdFilterConfig>(
            &cfg,
            "unknown variant `On`, expected one of `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`",
        );
    }

    #[test]
    fn test_filter_new() {
        assert_eq!(
            ThresholdFilter::new(LevelFilter::Info),
            ThresholdFilter {
                level: LevelFilter::Info
            }
        );
    }

    #[test]
    fn test_threshold() {
        let thres = ThresholdFilter::new(LevelFilter::Info);
        let debug_record = Record::builder()
            .level(Level::Debug)
            .args(format_args!("the message"))
            .module_path(Some("path"))
            .file(Some("file"))
            .line(Some(132))
            .build();

        assert_eq!(thres.filter(&debug_record), Response::Reject);

        let error_record = Record::builder()
            .level(Level::Error)
            .args(format_args!("the message"))
            .module_path(Some("path"))
            .file(Some("file"))
            .line(Some(132))
            .build();

        assert_eq!(thres.filter(&error_record), Response::Neutral);
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_cfg_deserializer() {
        let filter_cfg = ThresholdFilterConfig {
            level: LevelFilter::Off,
        };

        let deserializer = ThresholdFilterDeserializer;

        let res = deserializer.deserialize(filter_cfg, &Deserializers::default());
        assert!(res.is_ok());
    }
}
