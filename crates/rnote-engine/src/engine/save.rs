// Imports
use crate::fileformats::rnoteformat::{
    compression::CompressionMethod, serialization::SerializationMethod, RnoteHeader,
};
use serde::{Deserialize, Serialize};
use std::mem::discriminant;

/// Rnote file save preferences, a subset of RnoteHeader
/// used by EngineSnapshot, Engine, and EngineConfig
///
/// when loading in an Rnote file, SavePrefs will be created from RnoteHeader
/// if RnoteHeader's serialization and compression methods conform to the defaults, or method_lock is set to true
/// => SavePrefs override EngineSnapshot's default SavePrefs
/// => SavePrefs transferred from EngineSnapshot to Engine
///
/// as for EngineConfig, if and only if Engine's SavePrefs conform to the defaults
/// => SavePrefs cloned from Engine into EngineConfig
///
/// please note that the compression level is not used to check whether or not the methods conform to the defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "save_prefs")]
pub struct SavePrefs {
    #[serde(rename = "serialization")]
    pub serialization: SerializationMethod,
    #[serde(rename = "compression")]
    pub compression: CompressionMethod,
    #[serde(rename = "method_lock")]
    pub method_lock: bool,
}

impl SavePrefs {
    pub fn clone_config(&self) -> Self {
        self.clone()
    }
    pub fn conforms_to_default(&self) -> bool {
        discriminant(&self.serialization) == discriminant(&SerializationMethod::default())
            && discriminant(&self.compression) == discriminant(&CompressionMethod::default())
    }
    /// The EngineExport should only contain SavePrefs that conform to the default
    /// otherwise for example, after having opened an uncompressed and JSON-encoded Rnote
    /// save file while debugging, all new save files would be using the same methods
    pub fn clone_conformed_config(&self) -> Self {
        if self.conforms_to_default() {
            self.clone_config()
        } else {
            Self::default()
        }
    }
}

impl Default for SavePrefs {
    fn default() -> Self {
        Self {
            serialization: SerializationMethod::default(),
            compression: CompressionMethod::default(),
            method_lock: false,
        }
    }
}

impl From<RnoteHeader> for SavePrefs {
    fn from(value: RnoteHeader) -> Self {
        Self {
            serialization: value.serialization,
            compression: value.compression,
            method_lock: value.method_lock,
        }
    }
}
