//! Appenders

use log::{Log, Record};
#[cfg(feature = "config_parsing")]
use serde::Deserialize;
use std::fmt;

#[cfg(all(feature = "config_parsing", feature = "console_appender"))]
use self::console::ConsoleAppenderConfig;
#[cfg(all(feature = "config_parsing", feature = "file_appender"))]
use self::file::FileAppenderConfig;
#[cfg(all(feature = "config_parsing", feature = "rolling_file_appender"))]
use self::rolling_file::RollingFileAppenderConfig;
#[cfg(feature = "config_parsing")]
use crate::config::raw::DeserializingConfigError;
#[cfg(feature = "config_parsing")]
use crate::config::runtime::AppenderBuilder;
#[cfg(feature = "config_parsing")]
use crate::config::runtime::IntoAppender;
#[cfg(feature = "config_parsing")]
use crate::config::Appender;
#[cfg(feature = "config_parsing")]
use crate::config::LocalOrUser;
#[cfg(feature = "config_parsing")]
use crate::filter::IntoFilter;

#[cfg(feature = "console_appender")]
pub mod console;
#[cfg(feature = "file_appender")]
pub mod file;
#[cfg(feature = "rolling_file_appender")]
pub mod rolling_file;

#[cfg(any(feature = "file_appender", feature = "rolling_file_appender"))]
mod env_util {
    const ENV_PREFIX: &str = "$ENV{";
    const ENV_PREFIX_LEN: usize = ENV_PREFIX.len();
    const ENV_SUFFIX: char = '}';
    const ENV_SUFFIX_LEN: usize = 1;

    fn is_env_var_start(c: char) -> bool {
        // Close replacement for old [\w]
        // Note that \w implied \d and '_' and non-ASCII letters/digits.
        c.is_alphanumeric() || c == '_'
    }

    fn is_env_var_part(c: char) -> bool {
        // Close replacement for old [\w\d_.]
        c.is_alphanumeric() || c == '_' || c == '.'
    }

    pub fn expand_env_vars(path: std::path::PathBuf) -> std::path::PathBuf {
        let path: String = path.to_string_lossy().into();
        let mut outpath: String = path.clone();
        for (match_start, _) in path.match_indices(ENV_PREFIX) {
            let env_name_start = match_start + ENV_PREFIX_LEN;
            let (_, tail) = path.split_at(env_name_start);
            let mut cs = tail.chars();
            // Check first character.
            if let Some(ch) = cs.next() {
                if is_env_var_start(ch) {
                    let mut env_name = String::new();
                    env_name.push(ch);
                    // Consume following characters.
                    let valid = loop {
                        match cs.next() {
                            Some(ch) if is_env_var_part(ch) => env_name.push(ch),
                            Some(ENV_SUFFIX) => break true,
                            _ => break false,
                        }
                    };
                    // Try replacing properly terminated env var.
                    if valid {
                        if let Ok(env_value) = std::env::var(&env_name) {
                            let match_end = env_name_start + env_name.len() + ENV_SUFFIX_LEN;
                            // This simply rewrites the entire outpath with all instances
                            // of this var replaced. Could be done more efficiently by building
                            // `outpath` as we go when processing `path`. Not critical.
                            outpath = outpath.replace(&path[match_start..match_end], &env_value);
                        }
                    }
                }
            }
        }
        outpath.into()
    }
}

/// A trait implemented by log4rs appenders.
///
/// Appenders take a log record and processes them, for example, by writing it
/// to a file or the console.
pub trait Append: fmt::Debug + Send + Sync + 'static {
    /// Processes the provided `Record`.
    fn append(&self, record: &Record) -> anyhow::Result<()>;

    /// Flushes all in-flight records.
    fn flush(&self);
}

impl<T: Log + fmt::Debug + 'static> Append for T {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        self.log(record);
        Ok(())
    }

    fn flush(&self) {
        Log::flush(self)
    }
}

/// Configuration for an appender.
#[cfg(feature = "config_parsing")]
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum LocalAppender {
    /// console_appender Config
    #[cfg(feature = "console_appender")]
    #[serde(rename = "console")]
    ConsoleAppender(ConsoleAppenderConfig),

    /// file_appender Config
    #[cfg(feature = "file_appender")]
    #[serde(rename = "file")]
    FileAppender(FileAppenderConfig),

    /// rolling_file_appender Config
    #[cfg(feature = "rolling_file_appender")]
    #[serde(rename = "rolling_file")]
    RollingFileAppender(RollingFileAppenderConfig),
}

#[cfg(feature = "config_parsing")]
#[allow(unused_variables)]
impl IntoAppender for LocalAppender {
    fn into_appender(
        self,
        build: AppenderBuilder,
        name: String,
    ) -> Result<Appender, DeserializingConfigError> {
        match self {
            #[cfg(feature = "console_appender")]
            LocalAppender::ConsoleAppender(c) => c.into_appender(build, name),
            #[cfg(feature = "file_appender")]
            LocalAppender::FileAppender(f) => f.into_appender(build, name),
            #[cfg(feature = "rolling_file_appender")]
            LocalAppender::RollingFileAppender(r) => r.into_appender(build, name),
        }
    }
}

/// LocalOrUserAppender configuration
#[cfg(feature = "config_parsing")]
pub type LocalOrUserAppender<T> = LocalOrUser<LocalAppender, T>;

/// Configuration for an appender.
#[cfg(feature = "config_parsing")]
#[derive(Deserialize, Debug, Clone)]
pub struct AppenderConfig<A, F>
where
    A: Clone + IntoAppender,
    F: Clone + IntoFilter,
{
    /// The filters attached to the appender.
    #[serde(default = "filters_default")]
    pub filters: Vec<F>,
    /// The appender configuration.
    #[serde(flatten)]
    pub appender: A,
}
#[cfg(feature = "config_parsing")]
fn filters_default<F>() -> Vec<F>
where
    F: Clone + IntoFilter,
{
    Vec::new()
}
#[cfg(feature = "config_parsing")]
impl<A, F> IntoAppender for AppenderConfig<A, F>
where
    A: Clone + IntoAppender,
    F: Clone + IntoFilter,
{
    fn into_appender(
        self,
        build: AppenderBuilder,
        name: String,
    ) -> Result<Appender, DeserializingConfigError> {
        self.appender.into_appender(build, name)
    }
}

#[cfg(test)]
mod test {
    #[cfg(any(feature = "file_appender", feature = "rolling_file_appender"))]
    use std::{
        env::{set_var, var},
        path::PathBuf,
    };

    #[test]
    #[cfg(any(feature = "file_appender", feature = "rolling_file_appender"))]
    fn expand_env_vars_tests() {
        set_var("HELLO_WORLD", "GOOD BYE");
        #[cfg(not(target_os = "windows"))]
        let test_cases = vec![
            ("$ENV{HOME}", PathBuf::from(var("HOME").unwrap())),
            (
                "$ENV{HELLO_WORLD}",
                PathBuf::from(var("HELLO_WORLD").unwrap()),
            ),
            (
                "$ENV{HOME}/test",
                PathBuf::from(format!("{}/test", var("HOME").unwrap())),
            ),
            (
                "/test/$ENV{HOME}",
                PathBuf::from(format!("/test/{}", var("HOME").unwrap())),
            ),
            (
                "/test/$ENV{HOME}/test",
                PathBuf::from(format!("/test/{}/test", var("HOME").unwrap())),
            ),
            (
                "/test$ENV{HOME}/test",
                PathBuf::from(format!("/test{}/test", var("HOME").unwrap())),
            ),
            (
                "test/$ENV{HOME}/test",
                PathBuf::from(format!("test/{}/test", var("HOME").unwrap())),
            ),
            (
                "/$ENV{HOME}/test/$ENV{USER}",
                PathBuf::from(format!(
                    "/{}/test/{}",
                    var("HOME").unwrap(),
                    var("USER").unwrap()
                )),
            ),
            (
                "$ENV{SHOULD_NOT_EXIST}",
                PathBuf::from("$ENV{SHOULD_NOT_EXIST}"),
            ),
            (
                "/$ENV{HOME}/test/$ENV{SHOULD_NOT_EXIST}",
                PathBuf::from(format!(
                    "/{}/test/$ENV{{SHOULD_NOT_EXIST}}",
                    var("HOME").unwrap()
                )),
            ),
            (
                "/unterminated/$ENV{USER",
                PathBuf::from("/unterminated/$ENV{USER"),
            ),
        ];

        #[cfg(target_os = "windows")]
        let test_cases = vec![
            ("$ENV{HOMEPATH}", PathBuf::from(var("HOMEPATH").unwrap())),
            (
                "$ENV{HELLO_WORLD}",
                PathBuf::from(var("HELLO_WORLD").unwrap()),
            ),
            (
                "$ENV{HOMEPATH}/test",
                PathBuf::from(format!("{}/test", var("HOMEPATH").unwrap())),
            ),
            (
                "/test/$ENV{USERNAME}",
                PathBuf::from(format!("/test/{}", var("USERNAME").unwrap())),
            ),
            (
                "/test/$ENV{USERNAME}/test",
                PathBuf::from(format!("/test/{}/test", var("USERNAME").unwrap())),
            ),
            (
                "/test$ENV{USERNAME}/test",
                PathBuf::from(format!("/test{}/test", var("USERNAME").unwrap())),
            ),
            (
                "test/$ENV{USERNAME}/test",
                PathBuf::from(format!("test/{}/test", var("USERNAME").unwrap())),
            ),
            (
                "$ENV{HOMEPATH}/test/$ENV{USERNAME}",
                PathBuf::from(format!(
                    "{}/test/{}",
                    var("HOMEPATH").unwrap(),
                    var("USERNAME").unwrap()
                )),
            ),
            (
                "$ENV{SHOULD_NOT_EXIST}",
                PathBuf::from("$ENV{SHOULD_NOT_EXIST}"),
            ),
            (
                "$ENV{HOMEPATH}/test/$ENV{SHOULD_NOT_EXIST}",
                PathBuf::from(format!(
                    "{}/test/$ENV{{SHOULD_NOT_EXIST}}",
                    var("HOMEPATH").unwrap()
                )),
            ),
            (
                "/unterminated/$ENV{USERNAME",
                PathBuf::from("/unterminated/$ENV{USERNAME"),
            ),
        ];

        for (input, expected) in test_cases {
            let res = super::env_util::expand_env_vars(input.into());
            assert_eq!(res, expected)
        }
    }
}
