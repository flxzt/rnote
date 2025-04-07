//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod compression;
pub(crate) mod legacy;
pub(crate) mod maj0min12;
pub(crate) mod prelude;
pub(crate) mod serialization;

// Re-exports
pub use compression::CompressionMethod;
pub use serialization::SerializationMethod;

// Imports
use super::{FileFormatLoader, FileFormatSaver};
use crate::engine::{EngineSnapshot, save::SavePrefs};
use legacy::LegacyRnoteFile;
use maj0min12::RnoteFileMaj0Min12;
use prelude::{Prelude, PreludeError};

pub type RnoteFile = maj0min12::RnoteFileMaj0Min12;
pub type RnoteHeader = maj0min12::RnoteHeaderMaj0Min12;

impl RnoteFileMaj0Min12 {
    pub const SEMVER: &'static str = crate::utils::crate_version();
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let version = semver::Version::parse(Self::SEMVER)?;
        let header = serde_json::to_vec(&ijson::to_value(&self.header)?)?;
        let prelude = Prelude::new(version, header.len()).try_to_bytes()?;

        // From running simple tests, using concat seems to be the best choice, it's much faster than either append extend.
        Ok([prelude.as_slice(), header.as_slice(), self.body.as_slice()].concat())
    }
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        match Prelude::try_from_bytes(bytes) {
            Ok((prelude, mut cursor)) => {
                let header_slice =
                    bytes
                        .get(cursor..cursor + prelude.header_size)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Failed to get RnoteHeader, insufficient bytes")
                        })?;
                cursor += prelude.header_size;

                let body_slice = bytes
                    .get(cursor..)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get body, insufficient bytes"))?;

                Ok(Self {
                    header: RnoteHeader::load_from_slice(header_slice, &prelude.version)?,
                    body: body_slice.to_vec(),
                })
            }
            Err(error) => match error.downcast_ref::<PreludeError>() {
                Some(PreludeError::LegacyRnoteFile) => {
                    RnoteFile::try_from(LegacyRnoteFile::load_from_bytes(bytes)?)
                }
                None => Err(error),
            },
        }
    }
}

impl RnoteHeader {
    fn load_from_slice(slice: &[u8], version: &semver::Version) -> anyhow::Result<Self> {
        if semver::VersionReq::parse(">=0.11.0")
            .unwrap()
            .matches(version)
        {
            Ok(ijson::from_value(&serde_json::from_slice(slice)?)?)
        } else {
            Err(anyhow::anyhow!("Unsupported version: '{}'", version))
        }
    }
}

impl TryFrom<RnoteFile> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFile) -> Result<Self, Self::Error> {
        let uc_size = usize::try_from(value.header.uc_size).unwrap_or(usize::MAX);
        let uc_body = value.header.compression.decompress(uc_size, value.body)?;
        let mut engine_snapshot = value.header.serialization.deserialize(&uc_body)?;
        engine_snapshot.save_prefs = SavePrefs::from(value.header);

        Ok(engine_snapshot)
    }
}

impl TryFrom<&EngineSnapshot> for RnoteFile {
    type Error = anyhow::Error;

    fn try_from(value: &EngineSnapshot) -> Result<Self, Self::Error> {
        let save_prefs = value.save_prefs;

        let compression = save_prefs.compression;
        let serialization = save_prefs.serialization;

        let uc_data = serialization.serialize(value)?;
        let uc_size = uc_data.len() as u64;
        let data = compression.compress(uc_data)?;
        let method_lock = save_prefs.method_lock;

        Ok(Self {
            header: RnoteHeader {
                compression,
                serialization,
                uc_size,
                method_lock,
            },
            body: data,
        })
    }
}
