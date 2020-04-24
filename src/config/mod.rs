use crate::Handle;
use log::SetLoggerError;
use thiserror::Error;

pub mod runtime;

#[cfg(feature = "config_parsing")]
mod file;
#[cfg(feature = "config_parsing")]
mod raw;

pub use runtime::{Appender, Config, Logger, Root};

#[cfg(feature = "config_parsing")]
pub use self::file::{init_file, load_config_file, FormatError};
#[cfg(feature = "config_parsing")]
pub use self::raw::{Deserializable, Deserialize, Deserializers, RawConfig};

/// Initializes the global logger as a log4rs logger with the provided config.
///
/// A `Handle` object is returned which can be used to adjust the logging
/// configuration.
pub fn init_config(config: runtime::Config) -> Result<crate::Handle, SetLoggerError> {
    let logger = crate::Logger::new(config);
    log::set_max_level(logger.max_log_level());
    let handle = Handle {
        shared: logger.0.clone(),
    };
    log::set_boxed_logger(Box::new(logger)).map(|()| handle)
}

/// Initializes the global logger as a log4rs logger using the provided raw config.
///
/// This will return errors if the appenders configuration is malformed or if we fail to set the global logger.
#[cfg(feature = "config_parsing")]
pub fn init_raw_config(config: RawConfig) -> anyhow::Result<()> {
    let (appenders, errors) = config.appenders_lossy(&Deserializers::default());
    if !errors.is_empty() {
        return Err(InitErrors(errors).into());
    }
    let config = Config::builder()
        .appenders(appenders)
        .loggers(config.loggers())
        .build(config.root())?;

    let logger = crate::Logger::new(config);
    log::set_max_level(log::LevelFilter::Info);
    log::set_boxed_logger(Box::new(logger))?;
    Ok(())
}

/// Collects the set of errors that occur when deserializing the appenders.
#[derive(Debug, Error)]
#[error("Errors on initialization: {0:#?}")]
pub struct InitErrors(Vec<anyhow::Error>);
