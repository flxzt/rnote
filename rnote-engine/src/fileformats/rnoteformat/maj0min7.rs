// Imports
use super::maj0min6::RnoteFileMaj0Min6;
use crate::engine::EngineSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFileMaj0Min7 {
    /// A snapshot of the engine.
    #[serde(rename = "engine_snapshot")]
    pub engine_snapshot: EngineSnapshot,
}

impl TryFrom<RnoteFileMaj0Min6> for RnoteFileMaj0Min7 {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFileMaj0Min6) -> Result<Self, Self::Error> {
        Ok(Self {
            engine_snapshot: serde_json::from_value(value.engine_snapshot)?,
        })
    }
}
