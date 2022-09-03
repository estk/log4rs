//! All things pertaining to log4rs config. See the docs root for examples.

use log::SetLoggerError;
use thiserror::Error;

use crate::Handle;

pub mod runtime;

#[cfg(feature = "config_parsing")]
mod file;
#[cfg(feature = "config_parsing")]
pub(crate) mod raw;

pub use runtime::{Appender, Config, Logger, Root};

#[cfg(feature = "config_parsing")]
pub use self::file::{init_file, load_config_file, FormatError};
#[cfg(feature = "config_parsing")]
pub use self::raw::RawConfig;
#[cfg(feature = "config_parsing")]
use self::runtime::IntoAppender;
#[cfg(feature = "config_parsing")]
use crate::filter::IntoFilter;
pub use raw::DeserializingConfigError;
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

/// Initializes the global logger as a log4rs logger with the provided config and error handler.
///
/// A `Handle` object is returned which can be used to adjust the logging
/// configuration.
pub fn init_config_with_err_handler(
    config: runtime::Config,
    err_handler: Box<dyn Send + Sync + Fn(&anyhow::Error)>,
) -> Result<crate::Handle, SetLoggerError> {
    let logger = crate::Logger::new_with_err_handler(config, err_handler);
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
pub fn init_raw_config<A, F>(config: RawConfig<A, F>) -> Result<(), InitError>
where
    A: Clone + IntoAppender,
    F: Clone + IntoFilter,
{
    let (appenders, errors) = config.appenders_lossy();
    if !errors.is_empty() {
        return Err(InitError::Deserializing(errors));
    }
    let config = Config::builder()
        .appenders(appenders)
        .loggers(config.loggers())
        .build(config.root())?;

    let logger = crate::Logger::new(config);
    log::set_max_level(logger.max_log_level());
    log::set_boxed_logger(Box::new(logger))?;
    Ok(())
}

/// Errors found when initializing.
#[derive(Debug, Error)]
pub enum InitError {
    /// There was an error deserializing.
    #[error("Errors found when deserializing the config: {0:#?}")]
    #[cfg(feature = "config_parsing")]
    Deserializing(#[from] raw::AppenderErrors),

    /// There was an error building the handle.
    #[error("Config building errors: {0:#?}")]
    BuildConfig(#[from] runtime::ConfigErrors),

    /// There was an error setting the global logger.
    #[error("Error setting the logger: {0:#?}")]
    SetLogger(#[from] log::SetLoggerError),
}

/// Local config or User config
#[cfg(feature = "config_parsing")]
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum LocalOrUser<L, U>
where
    L: Clone,
    U: Clone,
{
    /// Local config
    Local(L),
    /// User config
    User(U),
}

impl<L, U> Default for LocalOrUser<L, U>
where
    L: Clone + Default,
    U: Clone,
{
    fn default() -> Self {
        Self::Local(L::default())
    }
}
