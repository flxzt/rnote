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

    fn try_from(value: RnoteFileMaj0Min5Patch9) -> Result<Self, Self::Error> {
        Err(anyhow::anyhow!("FIXME"))
    }
}
