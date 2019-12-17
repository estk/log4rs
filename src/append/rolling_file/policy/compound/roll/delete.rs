//! The delete roller.
//!
//! Requires the `delete_roller` feature.

use std::error::Error;
use std::fs;
use std::path::Path;

use crate::append::rolling_file::policy::compound::roll::Roll;
#[cfg(feature = "file")]
use crate::file::{Deserialize, Deserializers};

/// Configuration for the delete roller.
#[cfg(feature = "file")]
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeleteRollerConfig {
    #[serde(skip_deserializing)]
    _p: (),
}

/// A roller which deletes the log file.
#[derive(Debug, Default)]
pub struct DeleteRoller(());

impl Roll for DeleteRoller {
    fn roll(&self, file: &Path) -> Result<(), Box<dyn Error + Sync + Send>> {
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
#[cfg(feature = "file")]
pub struct DeleteRollerDeserializer;

#[cfg(feature = "file")]
impl Deserialize for DeleteRollerDeserializer {
    type Trait = dyn Roll;

    type Config = DeleteRollerConfig;

    fn deserialize(
        &self,
        _: DeleteRollerConfig,
        _: &Deserializers,
    ) -> Result<Box<dyn Roll>, Box<dyn Error + Sync + Send>> {
        Ok(Box::new(DeleteRoller::default()))
    }
}
