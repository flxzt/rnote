use super::{
    legacy::maj0min9::RnoteFileMaj0Min9,
    methods::{CompressionMethod, SerializationMethod},
};
use serde::{Deserialize, Serialize};

/// # Rnote File Format Specifications
///
/// ## Prelude (not included in this struct, u16,u32,... are represented using little endian)
/// * magic number: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b], "RNOTEϕλ"
/// * version: [u64, u64, u64, u16, str, u16, str] (almost one-to-one representation of semver::Version)
///            [major, minor, patch, Prerelease size, Prerelease, BuildMetadata size, Buildmetadata]
/// * header size: u32
///
/// ## Header
/// a forward-compatible json-encoded struct
/// * serialization: method used to serialize/deserialize the engine snapshot
/// * compression: method used to compress/decompress the serialized engine snapshot
/// * uncompressed size: size of the uncompressed and serialized engine snapshot
/// * method lock: if set to true, the file can keep using non-standard methods and will not be forced back into using defaults
///
/// ## Body
/// the body contains the serialized and (potentially) compressed engine snapshot
#[derive(Debug, Clone)]
pub struct RnoteFileMaj0Min12 {
    /// The file's head is composed of the prelude plus the header (below).
    /// Contains the necessary information to efficiently compress/decompress, serialize/deserialize the rnote file.
    pub header: RnoteHeaderMaj0Min12,
    /// The serialized and (potentially) compressed engine snapshot.
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "header")]
pub struct RnoteHeaderMaj0Min12 {
    /// method used to serialize/deserialize the engine snapshot
    #[serde(rename = "serialization")]
    pub serialization: SerializationMethod,
    /// method used to compress/decompress the serialized engine snapshot
    #[serde(rename = "compression")]
    pub compression: CompressionMethod,
    /// size of the uncompressed and serialized engine snapshot
    #[serde(rename = "uncompressed_size")]
    pub uc_size: u64,
    #[serde(rename = "method_lock")]
    pub method_lock: bool,
}

impl TryFrom<RnoteFileMaj0Min9> for RnoteFileMaj0Min12 {
    type Error = anyhow::Error;
    /// Inefficient conversion, as the legacy struct stores the ijson EngineSnapshot and not the compressed and serialized bytes, thankfully bypassed in EngineSnapshot::load_from_rnote_bytes. Therefore in general this would only be used by rnote-cli mutate.
    fn try_from(value: RnoteFileMaj0Min9) -> Result<Self, Self::Error> {
        Ok(Self {
            header: RnoteHeaderMaj0Min12 {
                serialization: SerializationMethod::Json,
                compression: CompressionMethod::None,
                uc_size: 0,
                method_lock: false,
            },
            body: serde_json::to_vec(&value.engine_snapshot)?,
        })
    }
}
