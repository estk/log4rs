use log::SetLoggerError;
use std::error;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Read;
use std::thread;
use std::fmt;
use std::time::{Duration, SystemTime};

use {handle_error, init_config, Handle};
use file::{Deserializers, RawConfig};
#[cfg(feature = "xml_format")]
use file::RawConfigXml;
use config::Config;

/// Initializes the global logger as a log4rs logger configured via a file.
///
/// Configuration is read from a file located at the provided path on the
/// filesystem and components are created from the provided `Deserializers`.
///
/// Any nonfatal errors encountered when processing the configuration are
/// reported to stderr.
///
/// Requires the `file` feature (enabled by default).
pub fn init_file<P>(path: P, deserializers: Deserializers) -> Result<(), Error>
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

/// An error initializing the logging framework from a file.
#[derive(Debug)]
pub enum Error {
    /// An error from the log crate
    Log(SetLoggerError),
    /// A fatal error initializing the log4rs config.
    Log4rs(Box<error::Error + Sync + Send>),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Log(ref e) => fmt::Display::fmt(e, fmt),
            Error::Log4rs(ref e) => fmt::Display::fmt(e, fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Log(ref e) => error::Error::description(e),
            Error::Log4rs(ref e) => error::Error::description(&**e),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Log(ref e) => Some(e),
            Error::Log4rs(ref e) => Some(&**e),
        }
    }
}

impl From<SetLoggerError> for Error {
    fn from(t: SetLoggerError) -> Error {
        Error::Log(t)
    }
}

impl From<Box<error::Error + Sync + Send>> for Error {
    fn from(t: Box<error::Error + Sync + Send>) -> Error {
        Error::Log4rs(t)
    }
}

enum Format {
    #[cfg(feature = "yaml_format")] Yaml,
    #[cfg(feature = "json_format")] Json,
    #[cfg(feature = "toml_format")] Toml,
    #[cfg(feature = "xml_format")] Xml,
}

impl Format {
    fn from_path(path: &Path) -> Result<Format, Box<error::Error + Sync + Send>> {
        match path.extension().and_then(|s| s.to_str()) {
            #[cfg(feature = "yaml_format")]
            Some("yaml") | Some("yml") => Ok(Format::Yaml),
            #[cfg(not(feature = "yaml_format"))]
            Some("yaml") | Some("yml") => {
                Err("the `yaml_format` feature is required for YAML support".into())
            }
            #[cfg(feature = "json_format")]
            Some("json") => Ok(Format::Json),
            #[cfg(not(feature = "json_format"))]
            Some("json") => Err("the `json_format` feature is required for JSON support".into()),

            #[cfg(feature = "toml_format")]
            Some("toml") => Ok(Format::Toml),
            #[cfg(not(feature = "toml_format"))]
            Some("toml") => Err("the `toml_format` feature is required for TOML support".into()),

            #[cfg(feature = "xml_format")]
            Some("xml") => Ok(Format::Xml),
            #[cfg(not(feature = "xml_format"))]
            Some("xml") => Err("the `xml_format` feature is required for XML support".into()),

            Some(f) => Err(format!("unsupported file format `{}`", f).into()),
            None => Err("unable to determine the file format".into()),
        }
    }

    fn parse(&self, source: &str) -> Result<RawConfig, Box<error::Error + Sync + Send>> {
        match *self {
            #[cfg(feature = "yaml_format")]
            Format::Yaml => ::serde_yaml::from_str(source).map_err(Into::into),
            #[cfg(feature = "json_format")]
            Format::Json => ::serde_json::from_str(source).map_err(Into::into),
            #[cfg(feature = "toml_format")]
            Format::Toml => ::toml::from_str(source).map_err(Into::into),
            #[cfg(feature = "xml_format")]
            Format::Xml => ::serde_xml_rs::deserialize::<_, RawConfigXml>(source.as_bytes())
                .map(Into::into)
                .map_err(Into::into),
        }
    }
}

fn read_config(path: &Path) -> Result<String, Box<error::Error + Sync + Send>> {
    let mut file = File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

fn deserialize(config: &RawConfig, deserializers: &Deserializers) -> Config {
    let (appenders, errors) = config.appenders_lossy(deserializers);
    for error in &errors {
        handle_error(error);
    }

    let (config, errors) = Config::builder()
        .appenders(appenders)
        .loggers(config.loggers())
        .build_lossy(config.root());
    for error in &errors {
        handle_error(error);
    }

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
            path: path,
            format: format,
            source: source,
            modified: modified,
            deserializers: deserializers,
            handle: handle,
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
                Err(e) => handle_error(&*e),
            }
        }
    }

    fn run_once(
        &mut self,
        rate: Duration,
    ) -> Result<Option<Duration>, Box<error::Error + Sync + Send>> {
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
