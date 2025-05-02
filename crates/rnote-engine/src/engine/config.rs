// Imports
use super::{ExportPrefs, ImportPrefs};
use crate::pens::PensConfig;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Shared engine configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "engine_config")]
pub struct EngineConfig {
    #[serde(rename = "pens_config")]
    pub pens_config: PensConfig,
    #[serde(rename = "import_prefs")]
    pub import_prefs: ImportPrefs,
    #[serde(rename = "export_prefs")]
    pub export_prefs: ExportPrefs,
    #[serde(rename = "pen_sounds")]
    pub pen_sounds: bool,
    #[serde(rename = "optimize_epd")]
    pub optimize_epd: bool,
    #[serde(rename = "snap_positions")]
    pub snap_positions: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EngineConfigShared(pub(crate) Arc<RwLock<EngineConfig>>);

impl From<EngineConfig> for EngineConfigShared {
    fn from(value: EngineConfig) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }
}

impl Clone for EngineConfigShared {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl EngineConfigShared {
    pub fn read(&self) -> RwLockReadGuard<'_, EngineConfig> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, EngineConfig> {
        self.0.write().unwrap()
    }

    pub fn load_values(&self, config: EngineConfig) {
        let mut write = self.write();
        write.pens_config = config.pens_config;
        write.import_prefs = config.import_prefs;
        write.export_prefs = config.export_prefs;
        write.pen_sounds = config.pen_sounds;
        write.optimize_epd = config.optimize_epd;
        write.snap_positions = config.snap_positions;
    }
}
