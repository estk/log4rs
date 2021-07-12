//! The delete roller.
//!
//! Requires the `delete_roller` feature.

use std::{fs, path::Path};

use crate::append::rolling_file::policy::compound::roll::Roll;
#[cfg(feature = "config_parsing")]
use crate::config::{Deserialize, Deserializers};

/// Configuration for the delete roller.
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteRollerConfig {
    #[serde(skip_deserializing)]
    _p: (),
}

/// A roller which deletes the log file.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct DeleteRoller(());

impl Roll for DeleteRoller {
    fn roll(&self, file: &Path) -> anyhow::Result<()> {
        fs::remove_file(file).map_err(Into::into)
    }
}

impl DeleteRoller {
    /// Returns a new `DeleteRoller`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A deserializer for the `DeleteRoller`.
///
/// # Configuration
///
/// ```yaml
/// kind: delete
/// ```
#[cfg(feature = "config_parsing")]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct DeleteRollerDeserializer;

#[cfg(feature = "config_parsing")]
impl Deserialize for DeleteRollerDeserializer {
    type Trait = dyn Roll;

    type Config = DeleteRollerConfig;

    fn deserialize(
        &self,
        _: DeleteRollerConfig,
        _: &Deserializers,
    ) -> anyhow::Result<Box<dyn Roll>> {
        Ok(Box::new(DeleteRoller::default()))
    }
}
