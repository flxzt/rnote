use serde::{Deserialize, Serialize};

use super::RnoteFile;

/// Rnote file in version: maj 0 min 5 patch 9
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFileMaj0Min5Patch9 {
    /// the document
    #[serde(rename = "document", alias = "sheet")]
    pub document: serde_json::Value,
    /// A snapshot of the store
    #[serde(rename = "store_snapshot")]
    pub store_snapshot: serde_json::Value,
}

impl TryFrom<RnoteFileMaj0Min5Patch9> for RnoteFile {
    type Error = anyhow::Error;

    fn try_from(mut value: RnoteFileMaj0Min5Patch9) -> Result<Self, Self::Error> {
        let mut engine_snapshot = serde_json::Map::new();

        let store_snapshot = value
            .store_snapshot
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("store snapshot is not a json map."))?;

        engine_snapshot.insert(String::from("document"), value.document);
        engine_snapshot.insert(
            String::from("stroke_components"),
            store_snapshot.remove("stroke_components").ok_or_else(|| {
                anyhow::anyhow!("store snapshot has no value `stroke_compoenents`")
            })?,
        );
        engine_snapshot.insert(
            String::from("chrono_components"),
            store_snapshot.remove("chrono_components").ok_or_else(|| {
                anyhow::anyhow!("store snapshot has no value `chrono_compoenents`")
            })?,
        );
        engine_snapshot.insert(
            String::from("chrono_counter"),
            store_snapshot
                .remove("chrono_counter")
                .ok_or_else(|| anyhow::anyhow!("store snapshot has no value `chrono_counter`"))?,
        );

        Ok(Self {
            engine_snapshot: engine_snapshot.into(),
        })
    }
}
