//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`, where X: semver major, Y: semver minor version.
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod maj0min5patch8;
pub(crate) mod maj0min5patch9;
pub(crate) mod maj0min6;

// Imports
use self::maj0min5patch8::RnoteFileMaj0Min5Patch8;
use self::maj0min5patch9::RnoteFileMaj0Min5Patch9;
use self::maj0min6::RnoteFileMaj0Min6;
use crate::{FileFormatLoader, FileFormatSaver};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Compress bytes with gzip.
fn compress_to_gzip(to_compress: &[u8], file_name: &str) -> Result<Vec<u8>, anyhow::Error> {
    let compressed_bytes = Vec::<u8>::new();

    let mut encoder = flate2::GzBuilder::new()
        .filename(file_name)
        .write(compressed_bytes, flate2::Compression::default());

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
/// Used to extract and match to the version up front, before deserializing the actual data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnotefile_wrapper")]
struct RnotefileWrapper {
    #[serde(rename = "version")]
    version: semver::Version,
    #[serde(rename = "data")]
    data: serde_json::Value,
}

/// The Rnote file in the newest format version. The actual (de-) serialization into strong types is happening in `rnote-engine`.
///
/// This struct exists to allow for upgrading older versions before loading the file in.
pub type RnoteFile = RnoteFileMaj0Min6;

impl RnoteFile {
    pub const SEMVER: &str = "0.6.0";
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let wrapper = serde_json::from_slice::<RnotefileWrapper>(
            &decompress_from_gzip(bytes).context("decompress_from_gzip() failed")?,
        )
        .context("from_slice() for RnoteFileWrapper failed")?;

        // Conversions for older file format versions happen here
        if semver::VersionReq::parse(">=0.5.10")
            .unwrap()
            .matches(&wrapper.version)
        {
            Ok(serde_json::from_value::<Self>(wrapper.data)
                .context("from_value() for RnoteFile failed")?)
        } else if semver::VersionReq::parse(">=0.5.9")
            .unwrap()
            .matches(&wrapper.version)
        {
            Ok(Self::try_from(
                serde_json::from_value::<RnoteFileMaj0Min5Patch9>(wrapper.data)
                    .context("from_value() for RnoteFileMaj0Min5Patch9 failed")?,
            )
            .context("converting from RnoteFileMaj0Min5Patch9 to newest file version failed")?)
        } else if semver::VersionReq::parse(">=0.5.0")
            .unwrap()
            .matches(&wrapper.version)
        {
            RnoteFileMaj0Min5Patch9::try_from(
                serde_json::from_value::<RnoteFileMaj0Min5Patch8>(wrapper.data)
                    .context("from_value() for RnoteFileMaj0Min5Patch8 failed")?,
            )
            .and_then(Self::try_from)
            .context("converting RnoteFileMaj0Min5Patch8 to newest file version failed")
        } else {
            Err(anyhow::anyhow!(
                "failed to load rnote file from bytes, unsupported version: {}",
                wrapper.version
            ))
        }
    }
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let output = RnotefileWrapper {
            version: semver::Version::parse(Self::SEMVER).unwrap(),
            data: serde_json::to_value(self).context("to_value() for RnoteFile failed")?,
        };

        let compressed = compress_to_gzip(
            serde_json::to_string(&output)
                .context("serde_json::to_string() for output RnoteFileWrapper failed")?
                .as_bytes(),
            file_name,
        )
        .context("compress_to_gzip() failed")?;

        Ok(compressed)
    }
}
