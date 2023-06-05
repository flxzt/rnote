// Imports
use serde::{Deserialize, Serialize};
use std::{cell::Cell, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
/// Metadata of a revovery save
pub struct RecoveryMetadata {
    last_changed: Cell<u64>,
    rnote_path: PathBuf,
    #[serde(skip)]
    metdata_path: PathBuf,
}

impl RecoveryMetadata {
    /// Create new Cargo metadata
    pub fn new(metadata_path: impl Into<PathBuf>, rnote_path: impl Into<PathBuf>) -> Self {
        let out = Self {
            last_changed: Cell::new(0),
            rnote_path: rnote_path.into(),
            metdata_path: metadata_path.into(),
        };
        out.update_last_changed();
        out
    }
    /// Save recovery data
    pub fn save(&self) {
        std::fs::write(
            &self.metdata_path,
            serde_json::to_string(self).expect("Failed to parse recovery format"),
        )
        .expect("Failed to write file")
    }

    /// Replace last_changed with the current unix time
    pub fn update_last_changed(&self) {
        self.last_changed.replace(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Failed to get unix time")
                .as_secs(),
        );
    }
}
