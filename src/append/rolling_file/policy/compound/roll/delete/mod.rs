//! The delete roller.
//!
//! Requires the `delete_roller` feature.

use std::error::Error;
use std::fs;
use std::path::Path;

use append::rolling_file::policy::compound::roll::Roll;
use file::{Deserialize, Deserializers};

include!("config.rs");

/// A roller which deletes the log file.
#[derive(Debug)]
pub struct DeleteRoller(());

impl Roll for DeleteRoller {
    fn roll(&self, file: &Path) -> Result<(), Box<Error>> {
        fs::remove_file(file).map_err(Into::into)
    }
}

impl DeleteRoller {
    /// Returns a new `DeleteRoller`.
    pub fn new() -> DeleteRoller {
        DeleteRoller(())
    }
}

/// A deserializer for the `DeleteRoller`.
///
/// # Configuration
///
/// ```yaml
/// kind: delete
/// ```
pub struct DeleteRollerDeserializer;

impl Deserialize for DeleteRollerDeserializer {
    type Trait = Roll;

    type Config = DeleteRollerConfig;

    fn deserialize(&self,
                   _: DeleteRollerConfig,
                   _: &Deserializers)
                   -> Result<Box<Roll>, Box<Error>> {
        Ok(Box::new(DeleteRoller::new()))
    }
}
