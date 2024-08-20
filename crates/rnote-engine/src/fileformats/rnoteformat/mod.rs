//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod legacy;
pub(crate) mod maj0min12;
pub(crate) mod methods;

// Re-exports
pub use methods::{CompM, SerM};

// Imports
use super::{FileFormatLoader, FileFormatSaver};
use crate::engine::{save::SavePrefs, EngineSnapshot};
use legacy::LegacyRnoteFile;
use maj0min12::RnoteFileMaj0Min12;
use std::io::Write;

pub type RnoteFile = maj0min12::RnoteFileMaj0Min12;
pub type RnoteHeader = maj0min12::RnoteHeaderMaj0Min12;

impl RnoteFileMaj0Min12 {
    pub const MAGIC_NUMBER: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b];
    pub const VERSION: [u8; 3] = [0, 12, 0];
    pub const SEMVER: semver::Version = semver::Version::new(0, 12, 0);
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let json_header = serde_json::to_vec(&self.head)?;
        let header = [
            &Self::MAGIC_NUMBER[..],
            &Self::VERSION[..],
            &u32::try_from(json_header.len())?.to_le_bytes(),
            &json_header,
        ]
        .concat();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_all(&header)?;
        buffer.write_all(&self.body)?;
        Ok(buffer)
    }
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let magic_number = bytes
            .get(..9)
            .ok_or(anyhow::anyhow!("Failed to get magic number"))?;

        if magic_number != Self::MAGIC_NUMBER {
            // Gzip magic number
            if magic_number[..2] == [0x1f, 0x8b] {
                return RnoteFile::try_from(LegacyRnoteFile::load_from_bytes(bytes)?);
            } else {
                Err(anyhow::anyhow!("Unkown file format"))?;
            }
        }

        let mut version: [u8; 3] = [0; 3];
        version.copy_from_slice(
            bytes
                .get(9..12)
                .ok_or(anyhow::anyhow!("Failed to get version"))?,
        );
        let version = semver::Version::new(
            u64::from(version[0]),
            u64::from(version[1]),
            u64::from(version[2]),
        );

        let mut header_size: [u8; 4] = [0; 4];
        header_size.copy_from_slice(
            bytes
                .get(12..16)
                .ok_or(anyhow::anyhow!("Failed to get header size"))?,
        );
        let header_size = u32::from_le_bytes(header_size);
        let header_slice = bytes
            .get(16..16 + usize::try_from(header_size)?)
            .ok_or(anyhow::anyhow!("File head missing"))?;

        let body_slice = bytes
            .get(16 + usize::try_from(header_size)?..)
            .ok_or(anyhow::anyhow!("File body missing"))?;

        Ok(Self {
            head: RnoteHeader::load_from_slice(header_slice, &version)?,
            body: body_slice.to_vec(),
        })
    }
}

impl RnoteHeader {
    fn load_from_slice(slice: &[u8], version: &semver::Version) -> anyhow::Result<Self> {
        if semver::VersionReq::parse(">=0.12.0")
            .unwrap()
            .matches(version)
        {
            Ok(serde_json::from_slice(slice)?)
        } else {
            Err(anyhow::anyhow!("Unrecognized header"))
        }
    }
}

impl TryFrom<RnoteFile> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFile) -> Result<Self, Self::Error> {
        let uc_size = usize::try_from(value.head.uc_size).unwrap_or(usize::MAX);
        let uc_body = value.head.compression.decompress(uc_size, value.body)?;

        tracing::info!(
            "loaded rnote file\ncomp: {:?}\nseri: {:?}\nlock: {}\n",
            value.head.compression,
            value.head.serialization,
            value.head.method_lock,
        );

        let mut engine_snapshot = value.head.serialization.deserialize(&uc_body)?;
        // save preferences are only kept if method_lock is true or both the ser. method and comp. method are "defaults"
        let save_prefs = SavePrefs::from(value.head);
        if save_prefs.method_lock | save_prefs.conforms_to_default() {
            engine_snapshot.save_prefs = save_prefs;
        }
        Ok(engine_snapshot)
    }
}

impl TryFrom<&EngineSnapshot> for RnoteFile {
    type Error = anyhow::Error;

    fn try_from(value: &EngineSnapshot) -> Result<Self, Self::Error> {
        let save_prefs = value.save_prefs.clone_config();

        let compression = save_prefs.compression;
        let serialization = save_prefs.serialization;

        let uc_data = serialization.serialize(value)?;
        let uc_size = uc_data.len() as u64;
        let data = compression.compress(uc_data)?;
        let method_lock = save_prefs.method_lock;

        tracing::info!(
            "saving rnote file\ncomp: {compression:?}\nseri: {serialization:?}\nlock: {method_lock}\n"
        );

        Ok(Self {
            head: RnoteHeader {
                compression,
                serialization,
                uc_size,
                method_lock,
            },
            body: data,
        })
    }
}
