use super::{
    legacy::maj0min9::RnoteFileMaj0Min9,
    methods::{CompM, SerM},
};
use serde::{Deserialize, Serialize};

/// ## Description of Rnote's 0.12.0 file format
/// ### rnote magic number
/// the magic number present at the start of each rnote file, used to identify them
/// "RNOTEϕλ" -> "RNOTEFILE" (a useful pun, phi-lambda ~ file)
/// [u8; 9]
/// ### version
/// [u8; 3] [major, minor, patch]
/// ### header size
/// size of the json-encoded header, represented by 4 bytes, little endian
/// [u8; 4]
/// ### header
/// describes how to decompress and deserialize the data
/// ### data
/// serialized and compressed engine snapshot
#[derive(Debug, Clone)]
pub struct RnoteFileMaj0Min12 {
    pub head: RnoteHeaderMaj0Min12,
    /// A serialized and (potentially) compressed engine snapshot
    pub body: Vec<u8>,
}

impl RnoteFileMaj0Min12 {
    pub const VERSION: [u8; 3] = [0, 12, 0];
    pub const SEMVER: semver::Version = semver::Version::new(0, 12, 0);
}

impl TryFrom<RnoteFileMaj0Min9> for RnoteFileMaj0Min12 {
    type Error = anyhow::Error;
    fn try_from(value: RnoteFileMaj0Min9) -> Result<Self, Self::Error> {
        Ok(Self {
            head: RnoteHeaderMaj0Min12 {
                serialization: SerM::Json,
                compression: CompM::None,
                uc_size: 0,
            },
            body: serde_json::to_vec(&value.engine_snapshot)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteHeaderMaj0Min12 {
    pub serialization: SerM,
    pub compression: CompM,
    pub uc_size: u64,
}
