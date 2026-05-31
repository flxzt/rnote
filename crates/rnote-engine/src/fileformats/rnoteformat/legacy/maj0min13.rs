// Imports
use super::maj0min9::RnoteFileMaj0Min9;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFileMaj0Min13 {
    /// A snapshot of the engine.
    #[serde(rename = "engine_snapshot")]
    pub engine_snapshot: ijson::IValue,
}

impl TryFrom<RnoteFileMaj0Min9> for RnoteFileMaj0Min13 {
    type Error = anyhow::Error;

    fn try_from(mut value: RnoteFileMaj0Min9) -> Result<Self, Self::Error> {
        let engine_snapshot = value
            .engine_snapshot
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("engine snapshot is not a JSON object."))?;
        let document = engine_snapshot
            .get_mut("document")
            .ok_or_else(|| anyhow!("`engine_snapshot` has no value `document`."))?
            .as_object_mut()
            .ok_or_else(|| anyhow!("`document` is not a JSON object."))?;

        let format = document
            .remove("format")
            .ok_or_else(|| anyhow!("document has no value `format`."))?;
        let background = document
            .remove("background")
            .ok_or_else(|| anyhow!("document has no value `background`."))?;
        let layout = document
            .remove("layout")
            .ok_or_else(|| anyhow!("document has no value `layout`."))?;
        // discard `snap_positions`, this config is now global.
        document.remove("snap_positions");

        let mut document_config = ijson::IObject::new();
        document_config.insert("format", format);
        document_config.insert("background", background);
        document_config.insert("layout", layout);
        document.insert("config", document_config);

        Ok(Self {
            engine_snapshot: value.engine_snapshot,
        })
    }
}
