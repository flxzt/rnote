use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    fs::remove_file,
    path::{Path, PathBuf},
};

use super::{FileFormatLoader, FileFormatSaver};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Metadata of a revovery save
pub struct RecoveryMetadata {
    title: RefCell<Option<String>>,
    document_path: RefCell<Option<PathBuf>>,
    last_changed: Cell<i64>,
    recovery_file_path: PathBuf,
    #[serde(skip)]
    metdata_path: PathBuf,
}

impl FileFormatSaver for RecoveryMetadata {
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let data = serde_json::to_string(self).expect("Failed to parse recovery format");
        let bytes = data.as_bytes();
        std::fs::write(file_name, bytes).expect("Failed to write file");
        Ok(bytes.to_vec())
    }
}
impl FileFormatLoader for RecoveryMetadata {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_slice(bytes).context("failed to parse bytes")
    }
}

impl RecoveryMetadata {
    /// Get the path to the file being backed up
    pub fn document_path(&self) -> Option<PathBuf> {
        self.document_path.borrow().clone()
    }
    /// Remove recovery file and metadata from disk
    pub fn delete(&self) {
        if let Err(e) = remove_file(&self.recovery_file_path) {
            log::error!(
                "Failed to delete recovery file {}: {e}",
                self.recovery_file_path.display()
            )
        };
        if let Err(e) = remove_file(&self.metdata_path) {
            log::error!(
                "Failed to delete recovery metadata {}: {e}",
                self.metdata_path.display()
            )
        }
    }
    /// Get the last changed date as unix timestamp
    pub fn last_changed(&self) -> i64 {
        self.last_changed.get()
    }
    /// Load instance from given Path
    pub fn load_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).context("Failed to read file")?;
        Self::load_from_bytes(&bytes)
    }
    /// Get the metadata path
    pub fn metadata_path(&self) -> PathBuf {
        self.metdata_path.clone()
    }

    /// Create new Cargo metadata
    pub fn new(metadata_path: impl Into<PathBuf>, rnote_path: impl Into<PathBuf>) -> Self {
        let out = Self {
            title: RefCell::new(None),
            document_path: RefCell::new(None),
            last_changed: Cell::new(0),
            recovery_file_path: rnote_path.into(),
            metdata_path: metadata_path.into(),
        };
        out.update_last_changed();
        out
    }
    /// Get the path to Recovery file
    pub fn recovery_file_path(&self) -> PathBuf {
        self.recovery_file_path.clone()
    }
    /// Save recovery data
    pub fn save(&self) -> anyhow::Result<Vec<u8>> {
        self.save_as_bytes(self.metdata_path.to_str().unwrap())
    }
    /// Get the document title
    pub fn title(&self) -> Option<String> {
        self.title.borrow().clone()
    }
    /// Update Metadate based of the given document option
    pub fn update(&self, document_path: &Option<PathBuf>) {
        self.update_last_changed();
        match document_path {
            Some(p) if document_path.ne(&*self.document_path.borrow()) => {
                self.document_path.replace(document_path.clone());
                self.title
                    .borrow_mut()
                    .replace(p.file_stem().unwrap().to_string_lossy().to_string());
            }
            Some(_) => (),
            None => (),
        };
    }
    /// Replace last_changed with the current unix time
    pub(crate) fn update_last_changed(&self) {
        self.last_changed.replace(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Failed to get unix time")
                .as_secs() as i64,
        );
    }
}
