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
pub use methods::{CompressionMethod, SerializationMethod};

// Imports
use super::{FileFormatLoader, FileFormatSaver};
use crate::engine::{save::SavePrefs, EngineSnapshot};
use anyhow::Context;
use legacy::LegacyRnoteFile;
use maj0min12::RnoteFileMaj0Min12;
use std::io::Write;

pub type RnoteFile = maj0min12::RnoteFileMaj0Min12;
pub type RnoteHeader = maj0min12::RnoteHeaderMaj0Min12;

impl RnoteFileMaj0Min12 {
    // ideally, this should never change
    pub const MAGIC_NUMBER: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b];
    pub const SEMVER: &'static str = crate::utils::crate_version();
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let version = semver::Version::parse(Self::SEMVER)?;
        let pre_release = version.pre.as_str();
        let build_metadata = version.build.as_str();

        let json_header = serde_json::to_vec(&ijson::to_value(&self.head)?)?;
        let header = [
            &Self::MAGIC_NUMBER[..],
            &version.major.to_le_bytes(),
            &version.minor.to_le_bytes(),
            &version.patch.to_le_bytes(),
            &u16::try_from(pre_release.len())
                .context("Prerelease text is too long")?
                .to_le_bytes(),
            pre_release.as_bytes(),
            &u16::try_from(build_metadata.len())
                .context("BuildMetadata text is too long")?
                .to_le_bytes(),
            build_metadata.as_bytes(),
            &u32::try_from(json_header.len())
                .context("Serialized header is too large")?
                .to_le_bytes(),
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
        let mut prelude_idx: usize = 0;

        let magic_number = bytes
            .get(prelude_idx..9)
            .ok_or_else(|| anyhow::anyhow!("Failed to get magic number"))?;
        prelude_idx += 9;

        if magic_number != Self::MAGIC_NUMBER {
            // Gzip magic number
            // The legacy file is generally caught first in Snapshot::load_from_rnote_bytes
            // howver this less efficient catch is necessary for rnote-cli mutate
            if magic_number[..2] == [0x1f, 0x8b] {
                return RnoteFile::try_from(LegacyRnoteFile::load_from_bytes(bytes)?);
            } else {
                Err(anyhow::anyhow!("Unrecognized file format"))?;
            }
        }

        let mut major: [u8; 8] = [0; 8];
        major.copy_from_slice(
            bytes
                .get(prelude_idx..prelude_idx + 8)
                .ok_or_else(|| anyhow::anyhow!("Failed to get version"))?,
        );
        prelude_idx += 8;
        let major = u64::from_le_bytes(major);

        let mut minor: [u8; 8] = [0; 8];
        minor.copy_from_slice(
            bytes
                .get(prelude_idx..prelude_idx + 8)
                .ok_or_else(|| anyhow::anyhow!("Failed to get version"))?,
        );
        prelude_idx += 8;
        let minor = u64::from_le_bytes(minor);

        let mut patch: [u8; 8] = [0; 8];
        patch.copy_from_slice(
            bytes
                .get(prelude_idx..prelude_idx + 8)
                .ok_or_else(|| anyhow::anyhow!("Failed to get version"))?,
        );
        prelude_idx += 8;
        let patch = u64::from_le_bytes(patch);

        let mut pre_release_length: [u8; 2] = [0; 2];
        pre_release_length.copy_from_slice(
            bytes
                .get(prelude_idx..prelude_idx + 2)
                .ok_or_else(|| anyhow::anyhow!("Failed to get Prerelease length"))?,
        );
        prelude_idx += 2;
        let pre_release_length = usize::from(u16::from_le_bytes(pre_release_length));

        let pre_release = if pre_release_length == 0 {
            semver::Prerelease::EMPTY
        } else {
            let text = core::str::from_utf8(
                bytes
                    .get(prelude_idx..prelude_idx + pre_release_length)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get Prerelease"))?,
            )?;
            prelude_idx += pre_release_length;
            semver::Prerelease::new(text)?
        };

        let mut build_metadata_length: [u8; 2] = [0; 2];
        build_metadata_length.copy_from_slice(
            bytes
                .get(prelude_idx..prelude_idx + 2)
                .ok_or_else(|| anyhow::anyhow!("Failed to get BuildMetadata length"))?,
        );
        prelude_idx += 2;
        let build_metadata_length = usize::from(u16::from_le_bytes(build_metadata_length));

        let build_metadata = if build_metadata_length == 0 {
            semver::BuildMetadata::EMPTY
        } else {
            let text = core::str::from_utf8(
                bytes
                    .get(prelude_idx..prelude_idx + build_metadata_length)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get BuildMetadata"))?,
            )?;
            prelude_idx += build_metadata_length;
            semver::BuildMetadata::new(text)?
        };

        let version = semver::Version {
            major,
            minor,
            patch,
            pre: pre_release,
            build: build_metadata,
        };

        let mut header_size: [u8; 4] = [0; 4];
        header_size.copy_from_slice(
            bytes
                .get(prelude_idx..prelude_idx + 4)
                .ok_or_else(|| anyhow::anyhow!("Failed to get header size"))?,
        );
        prelude_idx += 4;
        let header_size = usize::try_from(u32::from_le_bytes(header_size))
            .context("Rnote file header is too large")?;

        let header_slice = bytes
            .get(prelude_idx..prelude_idx + header_size)
            .ok_or_else(|| anyhow::anyhow!("File header missing"))?;
        prelude_idx += header_size;

        let body_slice = bytes
            .get(prelude_idx..)
            .ok_or_else(|| anyhow::anyhow!("File body missing"))?;

        Ok(Self {
            head: RnoteHeader::load_from_slice(header_slice, &version)?,
            body: body_slice.to_vec(),
        })
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
            Err(anyhow::anyhow!(
                "Unrecognized version specifier '{}' for header",
                version
            ))
        }
    }
}

impl TryFrom<RnoteFile> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFile) -> Result<Self, Self::Error> {
        let uc_size = usize::try_from(value.head.uc_size).unwrap_or(usize::MAX);
        let uc_body = value.head.compression.decompress(uc_size, value.body)?;
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
