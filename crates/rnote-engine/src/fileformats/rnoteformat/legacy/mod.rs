//! Loading and saving Rnote's legacy `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod maj0min13;
pub(crate) mod maj0min5patch8;
pub(crate) mod maj0min5patch9;
pub(crate) mod maj0min6;
pub(crate) mod maj0min9;

// Imports
use self::maj0min5patch8::RnoteFileMaj0Min5Patch8;
use self::maj0min5patch9::RnoteFileMaj0Min5Patch9;
use self::maj0min6::RnoteFileMaj0Min6;
use self::maj0min9::RnoteFileMaj0Min9;
use self::maj0min13::RnoteFileMaj0Min13;
use crate::fileformats::FileFormatLoader;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Decompress from gzip.
fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    // Optimization for the gzip format, defined by RFC 1952
    // capacity of the vector defined by the size of the uncompressed data
    // given in little endian format, by the last 4 bytes of "compressed"
    //
    //   ISIZE (Input SIZE)
    //     This contains the size of the original (uncompressed) input data modulo 2^32.
    let mut bytes: Vec<u8> = {
        let mut decompressed_size: [u8; 4] = [0; 4];
        let idx_start = compressed
            .len()
            .checked_sub(4)
            // only happens if the file has less than 4 bytes
            .ok_or_else(|| {
                anyhow::anyhow!("Invalid file")
                    .context("Failed to get the size of the decompressed data")
            })?;
        decompressed_size.copy_from_slice(&compressed[idx_start..]);
        // u32 -> usize to avoid issues on 32-bit architectures
        // also more reasonable since the uncompressed size is given by 4 bytes
        Vec::with_capacity(u32::from_le_bytes(decompressed_size) as usize)
    };

    let mut decoder = flate2::read::MultiGzDecoder::new(compressed);
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// The legacy rnote file wrapper.
///
/// Used to extract and match the version up front, before deserializing the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnotefile_wrapper")]
struct LegacyRnotefileWrapper {
    #[serde(rename = "version")]
    version: semver::Version,
    #[serde(rename = "data")]
    data: ijson::IValue,
}

/// The Rnote file in the last legacy format version.
///
/// This struct exists to allow for upgrading older versions before loading the file in.
pub type LegacyRnoteFile = RnoteFileMaj0Min13;

impl FileFormatLoader for LegacyRnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let wrapper = serde_json::from_slice::<LegacyRnotefileWrapper>(
            &decompress_from_gzip(bytes).context("decompressing bytes failed.")?,
        )
        .context("deserializing RnotefileWrapper from bytes failed.")?;

        // Conversions for older file format versions happen here
        if semver::VersionReq::parse(">=0.13.0")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min13>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min13 failed.")
        } else if semver::VersionReq::parse(">=0.9.0")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min9>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min9 failed.")
                .and_then(RnoteFileMaj0Min13::try_from)
                .context("converting RnoteFileMaj0Min9 to newest file version failed.")
        } else if semver::VersionReq::parse(">=0.5.10")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min6>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min6 failed.")
                .and_then(RnoteFileMaj0Min9::try_from)
                .and_then(RnoteFileMaj0Min13::try_from)
                .context("converting RnoteFileMaj0Min6 to newest file version failed.")
        } else if semver::VersionReq::parse(">=0.5.9")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min5Patch9>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min5Patch9 failed.")
                .and_then(RnoteFileMaj0Min6::try_from)
                .and_then(RnoteFileMaj0Min9::try_from)
                .and_then(RnoteFileMaj0Min13::try_from)
                .context("converting RnoteFileMaj0Min5Patch9 to newest file version failed.")
        } else if semver::VersionReq::parse(">=0.5.0")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min5Patch8>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min5Patch8 failed")
                .and_then(RnoteFileMaj0Min5Patch9::try_from)
                .and_then(RnoteFileMaj0Min6::try_from)
                .and_then(RnoteFileMaj0Min9::try_from)
                .and_then(RnoteFileMaj0Min13::try_from)
                .context("converting RnoteFileMaj0Min5Patch8 to newest file version failed.")
        } else {
            Err(anyhow::anyhow!(
                "failed to load rnote file from bytes, unsupported version: {}.",
                wrapper.version
            ))
        }
    }
}
