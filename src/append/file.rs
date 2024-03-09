//! The file appender.
//!
//! Requires the `file_appender` feature.

use derivative::Derivative;
use log::Record;
use parking_lot::Mutex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};
#[cfg(feature = "config_parsing")]
use crate::encode::EncoderConfig;

use crate::{
    append::{env_util::expand_env_vars, Append},
    encode::{pattern::PatternEncoder, writer::simple::SimpleWriter, Encode},
};

/// The file appender's configuration.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileAppenderConfig {
    path: String,
    encoder: Option<EncoderConfig>,
    append: Option<bool>,
}

/// An appender which logs to a file.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct FileAppender {
    path: PathBuf,
    #[derivative(Debug = "ignore")]
    file: Mutex<SimpleWriter<BufWriter<File>>>,
    encoder: Box<dyn Encode>,
}

impl Append for FileAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        let mut file = self.file.lock();
        self.encoder.encode(&mut *file, record)?;
        file.flush()?;
        Ok(())
    }

    fn flush(&self) {}
}

impl FileAppender {
    /// Creates a new `FileAppender` builder.
    pub fn builder() -> FileAppenderBuilder {
        FileAppenderBuilder {
            encoder: None,
            append: true,
        }
    }
}

/// A builder for `FileAppender`s.
pub struct FileAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    append: bool,
}

impl FileAppenderBuilder {
    /// Sets the output encoder for the `FileAppender`.
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> FileAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// Determines if the appender will append to or truncate the output file.
    ///
    /// Defaults to `true`.
    pub fn append(mut self, append: bool) -> FileAppenderBuilder {
        self.append = append;
        self
    }

    /// Consumes the `FileAppenderBuilder`, producing a `FileAppender`.
    /// The path argument can contain environment variables of the form $ENV{name_here},
    /// where 'name_here' will be the name of the environment variable that
    /// will be resolved. Note that if the variable fails to resolve,
    /// $ENV{name_here} will NOT be replaced in the path.
    pub fn build<P: AsRef<Path>>(self, path: P) -> io::Result<FileAppender> {
        let path_cow = path.as_ref().to_string_lossy();
        let path: PathBuf = expand_env_vars(path_cow).as_ref().into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .write(true)
            .append(self.append)
            .truncate(!self.append)
            .create(true)
            .open(&path)?;

        Ok(FileAppender {
            path,
            file: Mutex::new(SimpleWriter(BufWriter::with_capacity(1024, file))),
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::<PatternEncoder>::default()),
        })
    }
}

/// A deserializer for the `FileAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: file
///
/// # The path of the log file. Required.
/// # The path can contain environment variables of the form $ENV{name_here},
/// # where 'name_here' will be the name of the environment variable that
/// # will be resolved. Note that if the variable fails to resolve,
/// # $ENV{name_here} will NOT be replaced in the path.
/// path: log/foo.log
///
/// # Specifies if the appender should append to or truncate the log file if it
/// # already exists. Defaults to `true`.
/// append: true
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct FileAppenderDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for FileAppenderDeserializer {
    type Trait = dyn Append;

    type Config = FileAppenderConfig;

    fn deserialize(
        &self,
        config: FileAppenderConfig,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let mut appender = FileAppender::builder();
        if let Some(append) = config.append {
            appender = appender.append(append);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }
        Ok(Box::new(appender.build(&config.path)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_directories() {
        let tempdir = tempfile::tempdir().unwrap();

        FileAppender::builder()
            .build(tempdir.path().join("foo").join("bar").join("baz.log"))
            .unwrap();
    }

    #[test]
    fn test_append_trait() {
        use log::Level;

        let tempdir = tempfile::tempdir().unwrap();
        let appender = FileAppender::builder()
            .build(tempdir.path().join("foo.log"))
            .unwrap();

        log_mdc::insert("foo", "bar");
        let res = appender.append(
            &Record::builder()
                .level(Level::Debug)
                .target("target")
                .module_path(Some("module_path"))
                .file(Some("file"))
                .line(Some(100))
                .args(format_args!("{}", "message"))
                .build(),
        );
        assert!(res.is_ok());

        appender.flush();
    }

    #[test]
    fn test_appender_builder() {
        let append_choices = vec![true, false];
        let tempdir = tempfile::tempdir().unwrap();

        for do_append in append_choices {
            // No actionable test
            FileAppender::builder()
                .append(do_append)
                .build(tempdir.path().join("foo.log"))
                .unwrap();
        }
    }

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_config_deserializer() {
        use crate::config::Deserializers;
        use serde_value::Value;
        use std::collections::BTreeMap;

        let tempdir = tempfile::tempdir().unwrap();
        let file_cfg = FileAppenderConfig {
            path: tempdir.path().join("foo.log").to_str().unwrap().to_owned(),
            encoder: Some(EncoderConfig {
                kind: "pattern".to_owned(),
                config: Value::Map(BTreeMap::new()),
            }),
            append: Some(true),
        };

        let deserializer = FileAppenderDeserializer;

        let res = deserializer.deserialize(file_cfg, &Deserializers::default());
        assert!(res.is_ok());
    }
}
