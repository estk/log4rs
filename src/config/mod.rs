use log::SetLoggerError;
use thiserror::Error;

use crate::Handle;

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
pub fn init_raw_config(config: RawConfig) -> Result<(), InitError> {
    let (appenders, errors) = config.appenders_lossy(&Deserializers::default());
    if !errors.is_empty() {
        return Err(InitError::Deserializing(errors));
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

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Errors found when deserializing the config: {0:#?}")]
    Deserializing(#[from] raw::AppenderErrors),

    #[error("Config building errors: {0:#?}")]
    BuildConfig(#[from] runtime::ConfigErrors),

    #[error("Error setting the logger: {0:#?}")]
    SetLogger(#[from] log::SetLoggerError),
}
