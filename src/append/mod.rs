//! Appenders

use log::{Log, Record};
#[cfg(feature = "file")]
use serde::{de, Deserialize, Deserializer};
#[cfg(feature = "file")]
use serde_value::Value;
#[cfg(feature = "file")]
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[cfg(feature = "file")]
use file::Deserializable;
#[cfg(feature = "file")]
use filter::FilterConfig;

#[cfg(feature = "console_appender")]
pub mod console;
#[cfg(feature = "file_appender")]
pub mod file;
#[cfg(feature = "rolling_file_appender")]
pub mod rolling_file;

/// A trait implemented by log4rs appenders.
///
/// Appenders take a log record and processes them, for example, by writing it
/// to a file or the console.
pub trait Append: fmt::Debug + downcast_rs::DowncastSync + Send + Sync + 'static {
    /// Processes the provided `Record`.
    fn append(&self, record: &Record) -> Result<(), Box<dyn Error + Sync + Send>>;

    /// Flushes all in-flight records.
    fn flush(&self);
}

impl_downcast!(sync Append);

#[cfg(feature = "file")]
impl Deserializable for dyn Append {
    fn name() -> &'static str {
        "appender"
    }
}

impl<T: Log + fmt::Debug + 'static> Append for T {
    fn append(&self, record: &Record) -> Result<(), Box<dyn Error + Sync + Send>> {
        self.log(record);
        Ok(())
    }

    fn flush(&self) {
        Log::flush(self)
    }
}

/// Configuration for an appender.
#[cfg(feature = "file")]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AppenderConfig {
    /// The appender kind.
    pub kind: String,
    /// The filters attached to the appender.
    pub filters: Vec<FilterConfig>,
    /// The appender configuration.
    pub config: Value,
}

#[cfg(feature = "file")]
impl<'de> Deserialize<'de> for AppenderConfig {
    fn deserialize<D>(d: D) -> Result<AppenderConfig, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => kind.deserialize_into().map_err(|e| e.into_error())?,
            None => return Err(de::Error::missing_field("kind")),
        };

        let filters = match map.remove(&Value::String("filters".to_owned())) {
            Some(filters) => filters.deserialize_into().map_err(|e| e.into_error())?,
            None => vec![],
        };

        Ok(AppenderConfig {
            kind,
            filters,
            config: Value::Map(map),
        })
    }
}

#[cfg(test)]
mod test {
    use super::Append;
    use append::{console::ConsoleAppender, file::FileAppender};
    use std::path::Path;
    use tempdir::TempDir;

    fn sample_appends(tempdir: &Path) -> Vec<Box<dyn Append>> {
        vec![
            Box::new(
                FileAppender::builder()
                    .build(tempdir.join("file.log"))
                    .unwrap(),
            ),
            Box::new(ConsoleAppender::builder().build()),
        ]
    }

    #[test]
    fn downcast_to_concrete_appender() {
        let tempdir = TempDir::new("downcast").unwrap();
        let appends = sample_appends(tempdir.path());

        let file_append = appends[0]
            .downcast_ref::<FileAppender>()
            .expect("should be FileAppender");
        assert_eq!(file_append.path(), tempdir.path().join("file.log"));

        assert!(appends[1].is::<ConsoleAppender>());
    }
}
