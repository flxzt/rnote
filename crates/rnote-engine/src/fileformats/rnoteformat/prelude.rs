// Imports
use anyhow::{anyhow, bail, Context};
use thiserror::Error;

/// # Prelude
/// * magic number: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b], "RNOTEϕλ"
/// * version: [u64, u64, u64, u16, str, u16, str] (almost one-to-one representation of semver::Version)
///            [major, minor, patch, Prerelease size, Prerelease, BuildMetadata size, Buildmetadata]
/// * header size: u32
#[derive(Debug, Clone)]
pub struct Prelude {
    pub version: semver::Version,
    pub header_size: usize,
}

impl Prelude {
    /// The magic number used to identify rnote files, do not modify.
    pub const MAGIC_NUMBER: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b];

    /// Creates a new prelude.
    pub fn new(version: semver::Version, header_size: usize) -> Self {
        Self {
            version,
            header_size,
        }
    }

    /// Returns the byte representation of the prelude
    pub fn try_to_bytes(self) -> anyhow::Result<Vec<u8>> {
        let pre_release: &str = self.version.pre.as_str();
        let build_metadata: &str = self.version.build.as_str();

        Ok([
            &Self::MAGIC_NUMBER[..],
            &self.version.major.to_le_bytes(),
            &self.version.minor.to_le_bytes(),
            &self.version.patch.to_le_bytes(),
            &u16::try_from(pre_release.len())
                .context("Prerelease exceeds maximum size (u16::MAX)")?
                .to_le_bytes(),
            pre_release.as_bytes(),
            &u16::try_from(build_metadata.len())
                .context("BuildMetadata exceeds maximum size (u16::MAX)")?
                .to_le_bytes(),
            build_metadata.as_bytes(),
            &u32::try_from(self.header_size)
                .context("Serialized RnoteHeader exceeds maximum size (u32::MAX)")?
                .to_le_bytes(),
        ]
        .concat())
    }

    /// Returns the prelude alongside the cursor which is the index at which it left off.
    pub fn try_from_bytes(bytes: &[u8]) -> anyhow::Result<(Self, usize)> {
        let mut cursor: usize = 0;

        let magic_number = bytes
            .get(cursor..9)
            .ok_or_else(|| anyhow!("Failed to get magic number"))?;
        cursor += 9;

        if magic_number != Self::MAGIC_NUMBER {
            // Checks for legacy files using the gzip magic number.
            if magic_number[..2] == [0x1f, 0x8b] {
                return Err(anyhow::Error::new(PreludeError::LegacyRnoteFile));
            } else {
                bail!("Unrecognized file format");
            }
        }

        let mut major: [u8; 8] = [0; 8];
        major.copy_from_slice(
            bytes
                .get(cursor..cursor + 8)
                .ok_or_else(|| anyhow!("Failed to get version.major, insufficient bytes"))?,
        );
        cursor += 8;
        let major = u64::from_le_bytes(major);

        let mut minor: [u8; 8] = [0; 8];
        minor.copy_from_slice(
            bytes
                .get(cursor..cursor + 8)
                .ok_or_else(|| anyhow!("Failed to get version.minor, insufficient bytes"))?,
        );
        cursor += 8;
        let minor = u64::from_le_bytes(minor);

        let mut patch: [u8; 8] = [0; 8];
        patch.copy_from_slice(
            bytes
                .get(cursor..cursor + 8)
                .ok_or_else(|| anyhow!("Failed to get version.patch, insufficient bytes"))?,
        );
        cursor += 8;
        let patch = u64::from_le_bytes(patch);

        let mut pre_release_size: [u8; 2] = [0; 2];
        pre_release_size.copy_from_slice(
            bytes
                .get(cursor..cursor + 2)
                .ok_or_else(|| anyhow!("Failed to get size of version.pre, insufficient bytes"))?,
        );
        cursor += 2;
        let pre_release = match usize::from(u16::from_le_bytes(pre_release_size)) {
            0 => semver::Prerelease::EMPTY,
            len => {
                let text =
                    core::str::from_utf8(bytes.get(cursor..cursor + len).ok_or_else(|| {
                        anyhow!("Failed to get version.pre, insufficient bytes")
                    })?)?;
                cursor += len;
                semver::Prerelease::new(text)?
            }
        };

        let mut build_metadata_size: [u8; 2] = [0; 2];
        build_metadata_size.copy_from_slice(
            bytes.get(cursor..cursor + 2).ok_or_else(|| {
                anyhow!("Failed to get size of version.build, insufficient bytes")
            })?,
        );
        cursor += 2;
        let build_metadata = match usize::from(u16::from_le_bytes(build_metadata_size)) {
            0 => semver::BuildMetadata::EMPTY,
            len => {
                let text =
                    core::str::from_utf8(bytes.get(cursor..cursor + len).ok_or_else(|| {
                        anyhow!("Failed to get version.build, insufficient bytes")
                    })?)?;
                cursor += len;
                semver::BuildMetadata::new(text)?
            }
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
                .get(cursor..cursor + 4)
                .ok_or_else(|| anyhow!("Failed to get header size, insufficient bytes"))?,
        );
        cursor += 4;
        let header_size = usize::try_from(u32::from_le_bytes(header_size))
            .context("Serialized RnoteHeader exceeds maximum size (usize::MAX)")?;

        Ok((Self::new(version, header_size), cursor))
    }
}

/// Custom error used to handle legacy rnote files.
#[derive(Debug, Error)]
pub enum PreludeError {
    #[error("")]
    LegacyRnoteFile,
}
