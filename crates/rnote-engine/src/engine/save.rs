// Imports
use crate::fileformats::rnoteformat::{
    RnoteHeader, compression::CompressionMethod, serialization::SerializationMethod,
};
use serde::{Deserialize, Serialize};

// Re-exports
pub use crate::fileformats::rnoteformat::compression::CompressionLevel;

/// The SavePrefs struct is similar to RnoteHeader, it is used by Engine, EngineSnapshot, and EngineConfig.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default, rename = "save_prefs")]
pub struct SavePrefs {
    #[serde(rename = "serialization")]
    pub serialization: SerializationMethod,
    #[serde(rename = "compression")]
    pub compression: CompressionMethod,
    #[serde(skip)]
    pub method_lock: bool,
    #[serde(skip)]
    pub on_next_save: Option<(SerializationMethod, CompressionMethod)>,
}

impl SavePrefs {
    pub(crate) fn new(
        serialization: SerializationMethod,
        compression: CompressionMethod,
        method_lock: bool,
        on_next_save: Option<(SerializationMethod, CompressionMethod)>,
    ) -> Self {
        Self {
            serialization,
            compression,
            method_lock,
            on_next_save,
        }
    }

    pub(crate) fn new_simple(
        serialization: SerializationMethod,
        compression: CompressionMethod,
    ) -> Self {
        Self {
            serialization,
            compression,
            method_lock: false,
            on_next_save: None,
        }
    }

    /// Checks that serialization and compression are default methods.
    #[rustfmt::skip]
    pub(crate) fn uses_default_methods(&self) -> bool {
        self.serialization.is_similar_to(&SerializationMethod::default())
        && self.compression.is_similar_to(&CompressionMethod::default())
    }

    /// Used to export the SavePrefs of the Engine to an EngineSnapshot.
    pub(crate) fn clone_from_engine_to_enginesnapshot(&self) -> Self {
        match self.on_next_save {
            Some((serialization, compression)) => {
                // Necessary as the RnoteFile ignores the `on_next_save` component
                Self::new(serialization, compression, self.method_lock, None)
            }
            None => *self,
        }
    }

    /// Used to load the SavePrefs of the EngineConfig into the Engine.
    pub(crate) fn clone_from_engineconfig_to_engine(&self) -> Self {
        // SavePrefs can only be retrieved from the EngineConfig if they conform to the default methods.
        if self.uses_default_methods() {
            Self::new_simple(self.serialization, self.compression)
        } else {
            Self::default()
        }
    }

    /// Used to export the SavePrefs of the Engine to EngineConfig.
    pub(crate) fn clone_from_engine_to_engineconfig(&self) -> Self {
        match (self.uses_default_methods(), self.on_next_save) {
            (true | false, Some((serialization, compression))) => {
                Self::new_simple(serialization, compression)
            }
            (true, None) => *self,
            (false, None) => Self::default(),
        }
    }

    /// Used by engine to merge the incoming SavePrefs originating from a Rnote file into its current SavePrefs.
    pub(crate) fn merge(&mut self, new: &Self) {
        let on_next_save = match (new.uses_default_methods(), new.method_lock) {
            (true, true) | (true, false) | (false, true) => None,
            (false, false) => Some((self.serialization, self.compression)),
        };
        self.serialization = new.serialization;
        self.compression = new.compression;
        self.method_lock = new.method_lock;
        self.on_next_save = on_next_save;
    }

    pub fn finished_saving(&mut self) {
        if let Some((serialization, compression)) = self.on_next_save {
            self.serialization = serialization;
            self.compression = compression;
            self.on_next_save = None;
        }
    }

    #[rustfmt::skip]
    pub fn update_compression_level(&mut self, new_level: CompressionLevel) {
        let self_level = self.compression.get_compression_level();
        if self_level.ne(&new_level) {
            match self.on_next_save {
                Some((_, ref mut compression)) => compression.set_compression_level(&new_level),
                None => self.on_next_save = Some((self.serialization, self.compression.clone_with_new_compression_level(&new_level)))
            }
        }
        else if let Some((serialization, compression)) = self.on_next_save {
            if self.serialization.is_similar_to(&serialization) && self.compression.is_similar_to(&compression) {
                self.on_next_save = None
            }
        }
    }
}

impl From<RnoteHeader> for SavePrefs {
    fn from(value: RnoteHeader) -> Self {
        Self {
            serialization: value.serialization,
            compression: value.compression,
            method_lock: value.method_lock,
            on_next_save: None,
        }
    }
}
