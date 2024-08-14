use super::{maj0min9::RnoteFileMaj0Min9, CompressionMethods, SerializationMethods};
use serde::{Deserialize, Serialize};

/// ## Rnote 0.12.0 format
/// ### rnote magic number
/// [u8; 9] "RNOTEϕλ""
/// ### version
/// [u8; 3] [major, minor, patch]
/// ### header size
/// size of the json-encoded header, represented by two bytes, little endian
/// [u8; 2]
/// ### header
/// describes how to decompress and deserialize the data
/// ### data
/// serialized and compressed engine snapshot
#[derive(Debug, Clone)]
pub struct RnoteFileMaj0Min12 {
    pub header: RnoteHeaderMaj0Min12,
    /// A compressed and serialized snapshot of the engine.
    pub engine_snapshot: Vec<u8>,
}

impl RnoteFileMaj0Min12 {
    // "RNOTEΦΛ"
    pub const MAGIC_NUMBER: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b];
    pub const VERSION: [u8; 3] = [0, 12, 0];
}

impl TryFrom<RnoteFileMaj0Min9> for RnoteFileMaj0Min12 {
    type Error = anyhow::Error;
    fn try_from(value: RnoteFileMaj0Min9) -> Result<Self, Self::Error> {
        Ok(Self {
            header: RnoteHeaderMaj0Min12 {
                serialization: SerializationMethods::Json,
                compression: CompressionMethods::None,
                size: 0,
            },
            engine_snapshot: serde_json::to_vec(&value.engine_snapshot)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteHeaderMaj0Min12 {
    pub serialization: SerializationMethods,
    pub compression: CompressionMethods,
    pub size: u64,
}
