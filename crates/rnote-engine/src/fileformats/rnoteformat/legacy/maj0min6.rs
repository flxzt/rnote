// Imports
use super::maj0min5patch9::RnoteFileMaj0Min5Patch9;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFileMaj0Min6 {
    /// A snapshot of the engine.
    #[serde(rename = "engine_snapshot")]
    pub engine_snapshot: ijson::IValue,
}

impl TryFrom<RnoteFileMaj0Min5Patch9> for RnoteFileMaj0Min6 {
    type Error = anyhow::Error;

    fn try_from(mut value: RnoteFileMaj0Min5Patch9) -> Result<Self, Self::Error> {
        let mut engine_snapshot = ijson::IObject::new();

        let store_snapshot = value
            .store_snapshot
            .as_object_mut()
            .ok_or_else(|| anyhow!("store snapshot is not a JSON object."))?;

        engine_snapshot.insert(String::from("document"), value.document);
        engine_snapshot.insert(
            String::from("stroke_components"),
            store_snapshot
                .remove("stroke_components")
                .ok_or_else(|| anyhow!("store snapshot has no value `stroke_components`."))?,
        );
        engine_snapshot.insert(
            String::from("chrono_components"),
            store_snapshot
                .remove("chrono_components")
                .ok_or_else(|| anyhow!("store snapshot has no value `chrono_components`."))?,
        );
        engine_snapshot.insert(
            String::from("chrono_counter"),
            store_snapshot
                .remove("chrono_counter")
                .ok_or_else(|| anyhow!("store snapshot has no value `chrono_counter`."))?,
        );

        Ok(Self {
            engine_snapshot: engine_snapshot.into(),
        })
    }
}
