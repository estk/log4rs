//! The file appender.
//!
//! Requires the `file_appender` feature.

use chrono::prelude::Local;
use derive_more::Debug;
use log::Record;
use parking_lot::Mutex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

const TIME_PREFIX: &str = "$TIME{";
const TIME_PREFIX_LEN: usize = TIME_PREFIX.len();
const TIME_SUFFIX: char = '}';
const TIME_SUFFIX_LEN: usize = 1;
const MAX_REPLACEMENTS: usize = 5;

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
    header: Option<String>,
}

/// An appender which logs to a file.
#[derive(Debug)]
pub struct FileAppender {
    #[allow(dead_code)] // reason = "debug purposes only"
    path: PathBuf,
    #[debug(skip)]
    file: Mutex<SimpleWriter<BufWriter<File>>>,
    encoder: Box<dyn Encode>,
    #[allow(dead_code)] // reason = "debug purposes only"
    header: Option<String>,
}

impl Append for FileAppender {
    fn append(&self, record: &Record<'_>) -> anyhow::Result<()> {
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
            header: None,
        }
    }
}

/// A builder for `FileAppender`s.
pub struct FileAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    append: bool,
    header: Option<String>,
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

    /// Sets the header text to write at the beginning of new log files.
    ///
    /// The header supports templating with `$ENV{var}` for environment variables
    /// and `$TIME{format}` for timestamps using chrono formatting.
    /// A newline is automatically appended after the header.
    ///
    /// The header is only written when a new empty file is created or when
    /// truncating an existing file.
    pub fn header(mut self, header: impl Into<String>) -> FileAppenderBuilder {
        self.header = Some(header.into());
        self
    }

    /// Consumes the `FileAppenderBuilder`, producing a `FileAppender`.
    /// The path argument can contain special patterns that will be resolved:
    ///
    /// - `$ENV{name_here}`: This pattern will be replaced by `name_here`.
    ///   where 'name_here' will be the name of the environment variable that
    ///   will be resolved. Note that if the variable fails to resolve,
    ///   $ENV{name_here} will NOT be replaced in the path.
    /// - `$TIME{chrono_format}`: This pattern will be replaced by `chrono_format`.
    ///   where 'chrono_format' will be date/time format from chrono crate. Note
    ///   that if the chrono_format fails to resolve, $TIME{chrono_format} will
    ///   NOT be replaced in the path.
    pub fn build<P: AsRef<Path>>(self, path: P) -> io::Result<FileAppender> {
        let path_cow = path.as_ref().to_string_lossy();
        // Expand environment variables in the path
        let expanded_env_path: PathBuf = expand_env_vars(path_cow).as_ref().into();
        // Apply the date/time format to the path
        let final_path = self.date_time_format(expanded_env_path);

        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if file exists and is empty before opening
        let file_was_empty = if final_path.exists() {
            fs::metadata(&final_path)?.len() == 0
        } else {
            true
        };

        let file = OpenOptions::new()
            .write(true)
            .append(self.append)
            .truncate(!self.append)
            .create(true)
            .open(&final_path)?;

        let mut writer = SimpleWriter(BufWriter::with_capacity(1024, file));

        // Write header if file was empty or being truncated
        if file_was_empty || !self.append {
            if let Some(ref header) = self.header {
                super::header_util::write_header(&mut writer, header)?;
            }
        }

        Ok(FileAppender {
            path: final_path,
            file: Mutex::new(writer),
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::<PatternEncoder>::default()),
            header: self.header,
        })
    }

    fn date_time_format(&self, path: PathBuf) -> PathBuf {
        let mut replacements = 0;
        let mut date_time_path = path.to_str().unwrap().to_string();
        // Locate the start and end of the placeholder
        while let Some(start) = date_time_path.find(TIME_PREFIX) {
            if replacements >= MAX_REPLACEMENTS {
                break;
            }
            if let Some(end) = date_time_path[start..].find(TIME_SUFFIX) {
                let end = start + end;
                // Extract the date format string
                let date_format = &date_time_path[start + TIME_PREFIX_LEN..end];

                // Get the current date and time
                let now = Local::now();

                // Format the current date and time
                let formatted_date = now.format(date_format).to_string();

                // replacing the placeholder with the formatted date
                date_time_path.replace_range(start..end + TIME_SUFFIX_LEN, &formatted_date);
                replacements += 1;
            } else {
                // If there's no closing brace, we leave the placeholder as is
                break;
            }
        }
        PathBuf::from(date_time_path)
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
///
/// # Optional header text to write at the beginning of new log files.
/// # The header supports $ENV{var} and $TIME{format} templating.
/// # A newline is automatically appended. Only written when a new empty
/// # file is created or when truncating.
/// header: "=== Log started at $TIME{%Y-%m-%d %H:%M:%S} ==="
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
        if let Some(header) = config.header {
            appender = appender.header(header);
        }
        Ok(Box::new(appender.build(&config.path)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_directories() {
        let tempdir = tempfile::tempdir().unwrap();

        FileAppender::builder()
            .build(tempdir.path().join("foo").join("bar").join("baz.log"))
            .unwrap();
    }

    #[test]
    fn append_false() {
        let tempdir = tempfile::tempdir().unwrap();
        FileAppender::builder()
            .append(false)
            .build(tempdir.path().join("foo.log"))
            .unwrap();
    }

    #[test]
    fn test_date_time_format_with_valid_format() {
        let current_time = Local::now().format("%Y-%m-%d").to_string();
        let tempdir = tempfile::tempdir().unwrap();
        let builder = FileAppender::builder()
            .build(
                tempdir
                    .path()
                    .join("foo")
                    .join("bar")
                    .join("logs/log-$TIME{%Y-%m-%d}.log"),
            )
            .unwrap();
        let expected_path = tempdir
            .path()
            .join(format!("foo/bar/logs/log-{}.log", current_time));
        assert_eq!(builder.path, expected_path);
    }

    #[test]
    fn test_date_time_format_with_invalid_format() {
        let tempdir = tempfile::tempdir().unwrap();
        let builder = FileAppender::builder()
            .build(
                tempdir
                    .path()
                    .join("foo")
                    .join("bar")
                    .join("logs/log-$TIME{INVALID}.log"),
            )
            .unwrap();
        let expected_path = tempdir.path().join("foo/bar/logs/log-INVALID.log");
        assert_eq!(builder.path, expected_path);
    }

    #[test]
    fn test_date_time_format_with_no_closing_brace() {
        let tempdir = tempfile::tempdir().unwrap();
        let current_time = Local::now().format("%Y-%m-%d").to_string();
        let builder = FileAppender::builder()
            .build(
                tempdir
                    .path()
                    .join("foo")
                    .join("bar")
                    .join("logs/log-$TIME{%Y-%m-%d}-$TIME{no_closing_brace.log"),
            )
            .unwrap();
        let expected_path = tempdir.path().join(format!(
            "foo/bar/logs/log-{}-$TIME{{no_closing_brace.log",
            current_time
        ));
        assert_eq!(builder.path, expected_path);
    }

    #[test]
    fn test_date_time_format_with_max_replacements() {
        let tempdir = tempfile::tempdir().unwrap();
        let current_time = Local::now().format("%Y-%m-%d").to_string();
        let builder = FileAppender::builder()
            .build(
                tempdir
                    .path()
                    .join("foo")
                    .join("bar")
                    .join("logs/log-$TIME{%Y-%m-%d}-$TIME{%Y-%m-%d}-$TIME{%Y-%m-%d}-$TIME{%Y-%m-%d}-$TIME{%Y-%m-%d}.log"),
            )
            .unwrap();
        let expected_path = tempdir.path().join(format!(
            "foo/bar/logs/log-{}-{}-{}-{}-{}.log",
            current_time, current_time, current_time, current_time, current_time
        ));
        assert_eq!(builder.path, expected_path);
    }

    #[test]
    fn test_date_time_format_over_max_replacements() {
        let tempdir = tempfile::tempdir().unwrap();
        let current_time = Local::now().format("%Y-%m-%d").to_string();

        // Build a path with more than MAX_REPLACEMENTS ($TIME{...}) placeholders
        let path_str = format!(
            "foo/bar/logs/log-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}.log",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}",
            "$TIME{%Y-%m-%d}"
        );
        let builder = FileAppender::builder()
            .build(tempdir.path().join(path_str.clone()))
            .unwrap();

        // Only MAX_REPLACEMENTS should be replaced, the rest should remain as $TIME{...}
        let mut expected_path = "foo/bar/logs/log".to_string();
        for i in 0..10 {
            if i < MAX_REPLACEMENTS {
                expected_path.push_str(&format!("-{}", current_time));
            } else {
                expected_path.push_str("-$TIME{%Y-%m-%d}");
            }
        }
        expected_path.push_str(".log");

        let expected_path = tempdir.path().join(expected_path);
        assert_eq!(builder.path, expected_path);
    }

    #[test]
    fn test_date_time_format_without_placeholder() {
        let tempdir = tempfile::tempdir().unwrap();
        let builder = FileAppender::builder()
            .build(tempdir.path().join("foo").join("bar").join("bar.log"))
            .unwrap();
        let expected_path = tempdir.path().join("foo/bar/bar.log");
        assert_eq!(builder.path, expected_path);
    }

    #[test]
    fn test_date_time_format_with_multiple_placeholders() {
        let current_time = Local::now().format("%Y-%m-%d").to_string();
        let tempdir = tempfile::tempdir().unwrap();
        let builder = FileAppender::builder()
            .build(
                tempdir
                    .path()
                    .join("foo")
                    .join("bar")
                    .join("logs-$TIME{%Y-%m-%d}/log-$TIME{%Y-%m-%d}.log"),
            )
            .unwrap();
        let expected_path = tempdir.path().join(format!(
            "foo/bar/logs-{}/log-{}.log",
            current_time, current_time
        ));
        assert_eq!(builder.path, expected_path);
    }

    #[test]
    fn test_header_on_new_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("header_test.log");

        FileAppender::builder()
            .header("=== TEST HEADER ===")
            .build(&path)
            .unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.starts_with("=== TEST HEADER ===\n"));
    }

    #[test]
    fn test_header_with_env_templating() {
        use std::env;

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("template_test.log");

        env::set_var("LOG4RS_TEST_VAR", "test_value");

        FileAppender::builder()
            .header("Header: $ENV{LOG4RS_TEST_VAR}")
            .build(&path)
            .unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.starts_with("Header: test_value\n"));
    }

    #[test]
    fn test_header_with_time_templating() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("time_test.log");
        let current_year = Local::now().format("%Y").to_string();

        FileAppender::builder()
            .header("Log started in $TIME{%Y}")
            .build(&path)
            .unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.starts_with(&format!("Log started in {}\n", current_year)));
    }

    #[test]
    fn test_no_header_on_existing_nonempty_file_append() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("existing.log");

        // Create file with content
        fs::write(&path, "existing content\n").unwrap();

        FileAppender::builder()
            .header("=== HEADER ===")
            .append(true)
            .build(&path)
            .unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(!contents.contains("=== HEADER ==="));
        assert_eq!(contents, "existing content\n");
    }

    #[test]
    fn test_header_on_existing_empty_file_append() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("empty.log");

        // Create empty file
        File::create(&path).unwrap();

        FileAppender::builder()
            .header("=== HEADER ===")
            .append(true)
            .build(&path)
            .unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.starts_with("=== HEADER ===\n"));
    }

    #[test]
    fn test_header_with_truncate() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("truncate.log");

        // Create file with content
        fs::write(&path, "old content\n").unwrap();

        FileAppender::builder()
            .header("=== NEW HEADER ===")
            .append(false)
            .build(&path)
            .unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.starts_with("=== NEW HEADER ===\n"));
        assert!(!contents.contains("old content"));
    }
}
