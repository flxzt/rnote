// Imports
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize)]
/// Metadata of a revovery save
pub struct RecoveryMetadata {
    title: RefCell<Option<String>>,
    document_path: RefCell<Option<PathBuf>>,
    last_changed: Cell<i64>,
    recovery_file_path: PathBuf,
    #[serde(skip)]
    metdata_path: PathBuf,
}

impl RecoveryMetadata {
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
    /// Save recovery data
    pub fn save(&self) {
        std::fs::write(
            &self.metdata_path,
            serde_json::to_string(self).expect("Failed to parse recovery format"),
        )
        .expect("Failed to write file")
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
    /// Get the metadata path
    pub fn metadata_path(&self) -> PathBuf {
        self.metdata_path.clone()
    }
    /// Get the path to the file being backed up
    pub fn document_path(&self) -> Option<PathBuf> {
        self.document_path.borrow().clone()
    }
    /// Get the path to Recovery file
    pub fn recovery_file_path(&self) -> PathBuf {
        self.recovery_file_path.clone()
    }
    /// Get the last changed date as unix timestamp
    pub fn last_changed(&self) -> i64 {
        self.last_changed.get()
    }
    /// Get the document title
    pub fn title(&self) -> Option<String> {
        self.title.borrow().clone()
    }
}
