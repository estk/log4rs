use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};

use thiserror::Error;

use super::{init_config, Config, Deserializers, Handle, RawConfig};
use crate::handle_error;

/// Initializes the global logger as a log4rs logger configured via a file.
///
/// Configuration is read from a file located at the provided path on the
/// filesystem and components are created from the provided `Deserializers`.
///
/// Any nonfatal errors encountered when processing the configuration are
/// reported to stderr.
///
/// Requires the `file` feature (enabled by default).
pub fn init_file<P>(path: P, deserializers: Deserializers) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_path_buf();
    let format = Format::from_path(&path)?;
    let source = read_config(&path)?;
    // An Err here could come because mtime isn't available, so don't bail
    let modified = fs::metadata(&path).and_then(|m| m.modified()).ok();
    let config = format.parse(&source)?;

    let refresh_rate = config.refresh_rate();
    let config = deserialize(&config, &deserializers);

    match init_config(config) {
        Ok(handle) => {
            if let Some(refresh_rate) = refresh_rate {
                ConfigReloader::start(
                    path,
                    format,
                    refresh_rate,
                    source,
                    modified,
                    deserializers,
                    handle,
                );
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Loads a log4rs logger configuration from a file.
///
/// Unlike `init_file`, this function does not initialize the logger; it only
/// loads the `Config` and returns it.
pub fn load_config_file<P>(path: P, deserializers: Deserializers) -> anyhow::Result<Config>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let format = Format::from_path(&path)?;
    let source = read_config(&path)?;
    let config = format.parse(&source)?;

    Ok(deserialize(&config, &deserializers))
}

/// The various types of formatting errors that can be generated.
#[derive(Debug, Error)]
pub enum FormatError {
    /// The YAML feature flag was missing.
    #[error("the `yaml_format` feature is required for YAML support")]
    YamlFeatureFlagRequired,

    /// The JSON feature flag was missing.
    #[error("the `json_format` feature is required for JSON support")]
    JsonFeatureFlagRequired,

    /// The TOML feature flag was missing.
    #[error("the `toml_format` feature is required for TOML support")]
    TomlFeatureFlagRequired,

    /// An unsupported format was specified.
    #[error("unsupported file format `{0}`")]
    UnsupportedFormat(String),

    /// Log4rs could not determine the file format.
    #[error("unable to determine the file format")]
    UnknownFormat,
}

#[derive(Debug)]
enum Format {
    #[cfg(feature = "yaml_format")]
    Yaml,
    #[cfg(feature = "json_format")]
    Json,
    #[cfg(feature = "toml_format")]
    Toml,
}

impl Format {
    fn from_path(path: &Path) -> anyhow::Result<Format> {
        match path.extension().and_then(|s| s.to_str()) {
            #[cfg(feature = "yaml_format")]
            Some("yaml") | Some("yml") => Ok(Format::Yaml),
            #[cfg(not(feature = "yaml_format"))]
            Some("yaml") | Some("yml") => Err(FormatError::YamlFeatureFlagRequired.into()),

            #[cfg(feature = "json_format")]
            Some("json") => Ok(Format::Json),
            #[cfg(not(feature = "json_format"))]
            Some("json") => Err(FormatError::JsonFeatureFlagRequired.into()),

            #[cfg(feature = "toml_format")]
            Some("toml") => Ok(Format::Toml),
            #[cfg(not(feature = "toml_format"))]
            Some("toml") => Err(FormatError::TomlFeatureFlagRequired.into()),

            Some(f) => Err(FormatError::UnsupportedFormat(f.to_string()).into()),
            None => Err(FormatError::UnknownFormat.into()),
        }
    }

    #[allow(unused_variables)]
    fn parse(&self, source: &str) -> anyhow::Result<RawConfig> {
        match *self {
            #[cfg(feature = "yaml_format")]
            Format::Yaml => ::serde_yaml::from_str(source).map_err(Into::into),
            #[cfg(feature = "json_format")]
            Format::Json => ::serde_json::from_str(source).map_err(Into::into),
            #[cfg(feature = "toml_format")]
            Format::Toml => ::toml::from_str(source).map_err(Into::into),
        }
    }
}

fn read_config(path: &Path) -> anyhow::Result<String> {
    let s = fs::read_to_string(path)?;
    Ok(s)
}

fn deserialize(config: &RawConfig, deserializers: &Deserializers) -> Config {
    let (appenders, mut errors) = config.appenders_lossy(deserializers);
    errors.handle();

    let (config, mut errors) = Config::builder()
        .appenders(appenders)
        .loggers(config.loggers())
        .build_lossy(config.root());

    errors.handle();

    config
}

struct ConfigReloader {
    path: PathBuf,
    format: Format,
    source: String,
    modified: Option<SystemTime>,
    deserializers: Deserializers,
    handle: Handle,
}

impl ConfigReloader {
    fn start(
        path: PathBuf,
        format: Format,
        rate: Duration,
        source: String,
        modified: Option<SystemTime>,
        deserializers: Deserializers,
        handle: Handle,
    ) {
        let mut reloader = ConfigReloader {
            path,
            format,
            source,
            modified,
            deserializers,
            handle,
        };

        thread::Builder::new()
            .name("log4rs refresh".to_owned())
            .spawn(move || reloader.run(rate))
            .unwrap();
    }

    fn run(&mut self, mut rate: Duration) {
        loop {
            thread::sleep(rate);

            match self.run_once(rate) {
                Ok(Some(r)) => rate = r,
                Ok(None) => break,
                Err(e) => handle_error(&e),
            }
        }
    }

    fn run_once(&mut self, rate: Duration) -> anyhow::Result<Option<Duration>> {
        if let Some(last_modified) = self.modified {
            let modified = fs::metadata(&self.path).and_then(|m| m.modified())?;
            if last_modified == modified {
                return Ok(Some(rate));
            }

            self.modified = Some(modified);
        }

        let source = read_config(&self.path)?;

        if source == self.source {
            return Ok(Some(rate));
        }

        self.source = source;

        let config = self.format.parse(&self.source)?;
        let rate = config.refresh_rate();
        let config = deserialize(&config, &self.deserializers);

        self.handle.set_config(config);

        Ok(rate)
    }
}
