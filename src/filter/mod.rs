//! Filters

use log::Record;
use std::fmt;

use crate::config::LocalOrUser;

#[cfg(feature = "config_parsing")]
use self::threshold::ThresholdFilterConfig;

#[cfg(feature = "threshold_filter")]
pub mod threshold;

/// The trait implemented by log4rs filters.
///
/// Filters are associated with appenders and limit the log events that will be
/// sent to that appender.
pub trait Filter: fmt::Debug + Send + Sync + 'static {
    /// Filters a log event.
    fn filter(&self, record: &Record) -> Response;
}

///
pub trait IntoFilter {
    ///
    fn into_filter(self) -> anyhow::Result<Box<dyn Filter>>;
}

/// LocalOrUserFilter configuration
#[cfg(feature = "config_parsing")]
pub type LocalOrUserFilter<T> = LocalOrUser<LocalFilter, T>;
#[cfg(feature = "config_parsing")]
impl<T> IntoFilter for LocalOrUserFilter<T>
where
    T: IntoFilter + Clone,
{
    fn into_filter(self) -> anyhow::Result<Box<dyn Filter>> {
        match self {
            LocalOrUser::Local(l) => l.into_filter(),
            LocalOrUser::User(u) => u.into_filter(),
        }
    }
}

/// The response returned by a filter.
pub enum Response {
    /// Accept the log event.
    ///
    /// It will be immediately passed to the appender, bypassing any remaining
    /// filters.
    Accept,

    /// Take no action on the log event.
    ///
    /// It will continue on to remaining filters or pass on to the appender if
    /// there are none remaining.
    Neutral,

    /// Reject the log event.
    Reject,
}

/// Configuration for a filter.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum LocalFilter {
    ///
    #[cfg(feature = "threshold_filter")]
    #[serde(rename = "threshold")]
    ThresholdFilter(ThresholdFilterConfig),
}

#[cfg(feature = "config_parsing")]
impl Default for LocalFilter {
    fn default() -> Self {
        Self::ThresholdFilter(ThresholdFilterConfig::default())
    }
}

#[cfg(feature = "config_parsing")]
impl IntoFilter for LocalFilter {
    fn into_filter(self) -> anyhow::Result<Box<dyn Filter>> {
        match self {
            LocalFilter::ThresholdFilter(t) => t.into_filter(),
        }
    }
}
