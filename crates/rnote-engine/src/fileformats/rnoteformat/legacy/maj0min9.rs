// Imports
use super::maj0min6::RnoteFileMaj0Min6;
use crate::Camera;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFileMaj0Min9 {
    /// A snapshot of the engine.
    #[serde(rename = "engine_snapshot")]
    pub engine_snapshot: ijson::IValue,
}

impl TryFrom<RnoteFileMaj0Min6> for RnoteFileMaj0Min9 {
    type Error = anyhow::Error;

    fn try_from(mut value: RnoteFileMaj0Min6) -> Result<Self, Self::Error> {
        let engine_snapsht = value
            .engine_snapshot
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("engine snapshot is not a JSON object."))?;

        engine_snapsht.insert("camera", ijson::to_value(Camera::default())?);

        Ok(Self {
            engine_snapshot: value.engine_snapshot,
        })
    }
}
