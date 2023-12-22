use anyhow::Context;

use crate::engine::EngineSnapshot;

use super::{FileFormatLoader, FileFormatSaver};

#[derive(Debug)]
pub struct RnoteRecoveryFile {
    pub engine_snapshot: EngineSnapshot,
}

// impl From<&Engine> for RnoteRecoveryFile {
//     fn from(value: &Engine) -> Self {
//         Self {
//             engine_snapshot: bincode::serialize(value).unwrap(),
//         }
//     }
// }

impl FileFormatSaver for RnoteRecoveryFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let bytes = bincode::serialize(&self.engine_snapshot)?;
        Ok(bytes)
    }
}

impl FileFormatLoader for RnoteRecoveryFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            engine_snapshot: bincode::deserialize(bytes)
                .context("Failed to load recovery snapshot")?,
        })
    }
}

// impl From<RnoteRecoveryFile> for Engine {
//     fn from(val: RnoteRecoveryFile) -> Self {
//         bincode::deserialize(&val.engine_snapshot).unwrap()
//     }
// }
