// Imports
use crate::fileformats::rnoteformat::bcursor::BCursor;
use anyhow::{Context, bail};

/// # Prelude
/// * Magic number: [u8; 10] = [52, 4e, 4f, 54, 45, 2d, ce, a6, ce, 9b] = "RNOTE-ΦΛ"
/// * File version: u16
/// * Rnote version: [major (u64), minor (u64), patch (u64), Prerelease size (u16), Prerelease (str), BuildMetadata size (u16), BuildMetadata (str)]
/// * Header size: u32
#[derive(Debug, Clone)]
pub struct Prelude {
    pub file_version: u16,
    pub rnote_version: semver::Version,
    pub header_size: usize,
}

impl Prelude {
    /// The magic number used to identify Rnote files, do not modify.
    /// Translates to "RNOTE-ΦΛ" in utf-8
    pub const MAGIC_NUMBER: [u8; 10] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0x2d, 0xce, 0xa6, 0xce, 0x9b];

    /// Creates a new prelude.
    pub fn new(file_version: u16, rnote_version: semver::Version, header_size: usize) -> Self {
        Self {
            file_version,
            rnote_version,
            header_size,
        }
    }

    /// Returns the byte representation of the prelude
    pub fn try_to_bytes(self) -> anyhow::Result<Vec<u8>> {
        let pre_release: &str = self.rnote_version.pre.as_str();
        let build_metadata: &str = self.rnote_version.build.as_str();

        Ok([
            &Self::MAGIC_NUMBER[..],
            &self.file_version.to_le_bytes(),
            &self.rnote_version.major.to_le_bytes(),
            &self.rnote_version.minor.to_le_bytes(),
            &self.rnote_version.patch.to_le_bytes(),
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
        let magic_number = cursor.try_capture(Self::MAGIC_NUMBER.len())?;
        if magic_number != Self::MAGIC_NUMBER {
            bail!("Unrecognized file format");
        }

        let file_version = u16::from_le_bytes(cursor.try_capture_exact::<2>()?);

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

        let rnote_version = semver::Version {
            major,
            minor,
            patch,
            pre: pre_release,
            build: build_metadata,
        };

        let header_size: usize = u32::from_le_bytes(cursor.try_capture_exact::<4>()?).try_into()?;

        Ok(Self::new(file_version, rnote_version, header_size))
    }
}
