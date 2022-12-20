//! The file format is expected only to break on minor versions in prelease (0.x.x) and on major versions after 1.0.0 release. (equivalent to API's conforming to the semver spec)
//! Older formats can be added, with the naming scheme RnoteFileMaj<X>Min<Y>, where X: semver major, Y: semver minor version.
//! Then TryFrom is implemented to allow conversions and chaining from older to newer versions.

pub(crate) mod maj0min5patch8;
pub(crate) mod maj0min5patch9;

use anyhow::Context;
use maj0min5patch8::RnoteFileMaj0Min5Patch8;

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::{FileFormatLoader, FileFormatSaver};

use self::maj0min5patch9::RnoteFileMaj0Min5Patch9;

/// Compress bytes with gzip
fn compress_to_gzip(to_compress: &[u8], file_name: &str) -> Result<Vec<u8>, anyhow::Error> {
    let compressed_bytes = Vec::<u8>::new();

    let mut encoder = flate2::GzBuilder::new()
        .filename(file_name)
        .write(compressed_bytes, flate2::Compression::default());

    encoder.write_all(to_compress)?;

    Ok(encoder.finish()?)
}

/// Decompress from gzip
fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = flate2::read::MultiGzDecoder::new(compressed);
    let mut bytes: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    Ok(bytes)
}

/// The rnote file wrapper. used to extract and match to the version up front, before deserializing the actual data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnotefile_wrapper")]
struct RnotefileWrapper {
    #[serde(rename = "version")]
    version: semver::Version,
    #[serde(rename = "data")]
    data: serde_json::Value,
}

/// the Rnote file in the newest format version. The actual (de-) serialization into strong types is happening in `rnote-engine`.
/// This struct exists to allow for upgrading older versions before loading the file in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnotefile")]
pub struct RnoteFile {
    /// A snapshot of the engine
    #[serde(rename = "engine_snapshot")]
    pub engine_snapshot: serde_json::Value,
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<RnoteFile> {
        let decompressed = String::from_utf8(
            decompress_from_gzip(bytes).context("decompress_from_gzip() failed.")?,
        )
        .context("String::from_utf8() with unzipped data failed.")?;

        let wrapped_rnote_file = serde_json::from_str::<RnotefileWrapper>(decompressed.as_str())
            .context("from_str() for RnoteFileWrapper failed.")?;

        // Conversions for older file format versions happens here
        if semver::VersionReq::parse(">=0.5.10")
            .unwrap()
            .matches(&wrapped_rnote_file.version)
        {
            Ok(serde_json::from_value::<RnoteFile>(wrapped_rnote_file.data)
                .context("from_value() for RnoteFile failed.")?)
        } else if semver::VersionReq::parse(">=0.5.9")
            .unwrap()
            .matches(&wrapped_rnote_file.version)
        {
            Ok(Self::try_from(
                serde_json::from_value::<RnoteFileMaj0Min5Patch9>(wrapped_rnote_file.data)
                    .context("from_value() for RnoteFileMaj0Min5Patch9 failed.")?,
            )
            .context("converting from RnoteFileMaj0Min5Patch9 to newest file version failed.")?)
        } else if semver::VersionReq::parse(">=0.5.0")
            .unwrap()
            .matches(&wrapped_rnote_file.version)
        {
            RnoteFileMaj0Min5Patch9::try_from(
                serde_json::from_value::<RnoteFileMaj0Min5Patch8>(wrapped_rnote_file.data)
                    .context("from_value() for RnoteFileMaj0Min5Patch8 failed.")?,
            )
            .and_then(Self::try_from)
            .context("converting RnoteFileMaj0Min5Patch8 to newest file version failed.")
        } else {
            Err(anyhow::anyhow!(
                "failed to load rnote file from bytes, unsupported version: {}",
                wrapped_rnote_file.version
            ))
        }
    }
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let output = RnotefileWrapper {
            version: semver::Version::parse("0.5.10").unwrap(),
            data: serde_json::to_value(self).context("to_value() for RnoteFile failed.")?,
        };

        let compressed = compress_to_gzip(
            serde_json::to_string(&output)
                .context("serde_json::to_string() for output RnoteFileWrapper failed.")?
                .as_bytes(),
            file_name,
        )
        .context("compress_to_gzip() failed")?;

        Ok(compressed)
    }
}
