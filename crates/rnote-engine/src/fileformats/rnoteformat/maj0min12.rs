use super::methods::{CompM, SerM};
use serde::{Deserialize, Serialize};

/// # Rnote File Format Specifications
/// ## Prelude (not included in this struct)
/// * magic number: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b], "RNOTEϕλ"
/// * version: [u8; 3] = [major, minor, patch]
/// * header size: [u8; 4], little endian repr.
/// ## Header
/// the header is a forward-compatible json-encoded struct
/// containing additional information on the file
/// * serialization: method used to serialize/deserialize the engine snapshot
/// * compression: method used to compress/decompress the serialized engine snapshot
/// * uncompressed size: size of the uncompressed and serialized engine snapshot
/// * method_lock: if set to true, the file can keep using non-standard methods and will not be forced back into using defaults
/// ## Body
/// the body contains the serialized and (potentially) compressed engine snapshot
#[derive(Debug, Clone)]
pub struct RnoteFileMaj0Min12 {
    pub head: RnoteHeaderMaj0Min12,
    /// A serialized and (potentially) compressed engine snapshot
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "header")]
pub struct RnoteHeaderMaj0Min12 {
    /// method used to serialize/deserialize the engine snapshot
    #[serde(rename = "serialization")]
    pub serialization: SerM,
    /// method used to compress/decompress the serialized engine snapshot
    #[serde(rename = "compression")]
    pub compression: CompM,
    /// size of the uncompressed and serialized engine snapshot
    #[serde(rename = "uncompressed_size")]
    pub uc_size: u64,
    #[serde(rename = "method_lock")]
    pub method_lock: bool,
}
