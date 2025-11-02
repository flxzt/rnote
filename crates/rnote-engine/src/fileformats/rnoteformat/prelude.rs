// Imports
use anyhow::{Context, bail};

use crate::fileformats::rnoteformat::bcursor::BCursor;

/// # Prelude
/// * magic number: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b] = "RNOTEϕλ"
/// * version: [u64, u64, u64, u16, str, u16, str] (almost one-to-one representation of semver::Version)
///            [major, minor, patch, Prerelease size, Prerelease, BuildMetadata size, BuildMetadata]
/// * header size: u32
#[derive(Debug, Clone)]
pub struct Prelude {
    pub version: semver::Version,
    pub header_size: usize,
}

impl Prelude {
    /// The magic number used to identify Rnote files, do not modify.
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

    /// Returns the prelude alongside the cursor (index) at which it left off.
    pub fn try_from_bytes(cursor: &mut BCursor) -> anyhow::Result<Self> {
        // Used to wrap any error we get with more information, could be switched to a try block when those get stabilized
        let mut inner = || -> anyhow::Result<Self> {
            let magic_number = cursor.try_capture(9)?;
            if magic_number != Self::MAGIC_NUMBER {
                bail!("Unrecognized file format");
            }

            let major = u64::from_le_bytes(cursor.try_capture_exact::<8>()?);
            let minor = u64::from_le_bytes(cursor.try_capture_exact::<8>()?);
            let patch = u64::from_le_bytes(cursor.try_capture_exact::<8>()?);

            let pre_release_size = u16::from_le_bytes(cursor.try_capture_exact::<2>()?);
            let pre_release = if pre_release_size == 0 {
                semver::Prerelease::EMPTY
            } else {
                let text = core::str::from_utf8(cursor.try_capture(pre_release_size.into())?)?;
                semver::Prerelease::new(text)?
            };

            let build_metadata_size = u16::from_le_bytes(cursor.try_capture_exact::<2>()?);
            let build_metadata = if build_metadata_size == 0 {
                semver::BuildMetadata::EMPTY
            } else {
                let text = core::str::from_utf8(cursor.try_capture(build_metadata_size.into())?)?;
                semver::BuildMetadata::new(text)?
            };

            let version = semver::Version {
                major,
                minor,
                patch,
                pre: pre_release,
                build: build_metadata,
            };

            let header_size: usize =
                u32::from_le_bytes(cursor.try_capture_exact::<4>()?).try_into()?;

            Ok(Self::new(version, header_size))
        };

        inner().with_context(|| "Failed to load the prelude of the file")
    }
}
