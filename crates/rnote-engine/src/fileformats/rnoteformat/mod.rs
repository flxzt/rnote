//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod maj0min5patch8;
pub(crate) mod maj0min5patch9;
pub(crate) mod maj0min6;
pub(crate) mod maj0min9;

// Imports
use self::maj0min5patch8::RnoteFileMaj0Min5Patch8;
use self::maj0min5patch9::RnoteFileMaj0Min5Patch9;
use self::maj0min6::RnoteFileMaj0Min6;
use self::maj0min9::RnoteFileMaj0Min9;
use super::{FileFormatLoader, FileFormatSaver};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Compress bytes with gzip.
fn compress_to_gzip(to_compress: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut encoder =
        flate2::write::GzEncoder::new(Vec::<u8>::new(), flate2::Compression::default());
    encoder.write_all(to_compress)?;
    Ok(encoder.finish()?)
}

/// Decompress from gzip.
fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = flate2::read::MultiGzDecoder::new(compressed);
    let mut bytes: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    Ok(bytes)
}

/// The rnote file wrapper.
///
/// Used to extract and match the version up front, before deserializing the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnotefile_wrapper")]
struct RnotefileWrapper {
    #[serde(rename = "version")]
    version: semver::Version,
    #[serde(rename = "data")]
    data: ijson::IValue,
}

/// The Rnote file in the newest format version.
///
/// This struct exists to allow for upgrading older versions before loading the file in.
pub type RnoteFile = RnoteFileMaj0Min9;

impl RnoteFile {
    pub const SEMVER: &'static str = "0.10.2";
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let wrapper = serde_json::from_slice::<RnotefileWrapper>(
            &decompress_from_gzip(bytes).context("decompressing bytes failed.")?,
        )
        .context("deserializing RnotefileWrapper from bytes failed.")?;

        // Conversions for older file format versions happen here
        if semver::VersionReq::parse(">=0.9.0")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min9>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min9 failed.")
        } else if semver::VersionReq::parse(">=0.5.10")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min6>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min6 failed.")
                .and_then(RnoteFileMaj0Min9::try_from)
                .context("converting RnoteFileMaj0Min6 to newest file version failed.")
        } else if semver::VersionReq::parse(">=0.5.9")
            .unwrap()
            .matches(&wrapper.version)
        {
            ijson::from_value::<RnoteFileMaj0Min5Patch9>(&wrapper.data)
                .context("deserializing RnoteFileMaj0Min5Patch9 failed.")
                .and_then(RnoteFileMaj0Min6::try_from)
                .and_then(RnoteFileMaj0Min9::try_from)
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
                .context("converting RnoteFileMaj0Min5Patch8 to newest file version failed.")
        } else {
            Err(anyhow::anyhow!(
                "failed to load rnote file from bytes, unsupported version: {}.",
                wrapper.version
            ))
        }
    }
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let wrapper = RnotefileWrapper {
            version: semver::Version::parse(Self::SEMVER).unwrap(),
            data: ijson::to_value(self).context("converting RnoteFile to JSON value failed.")?,
        };
        let compressed = compress_to_gzip(
            serde_json::to_string(&wrapper)
                .context("Serializing RnoteFileWrapper failed.")?
                .as_bytes(),
        )
        .context("compressing bytes failed.")?;

        Ok(compressed)
    }
}
