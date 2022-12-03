//! The file format is expected only to break on minor versions in prelease (0.x.x) and on major versions after 1.0.0 release. (equivalent to API's conforming to the semver spec)
//! Older formats can be added, with the naming scheme RnoteFileMaj<X>Min<Y>, where X: semver major, Y: semver minor version.
//! Then TryFrom is implemented to allow conversions and chaining from older to newer versions.

pub(crate) mod maj0min5patch8;

use maj0min5patch8::RnoteFileMaj0Min5Patch8;

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::{FileFormatLoader, FileFormatSaver};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
/// the Rnote file in the newest format version. The actual (de-) serialization into strong types is happening in `rnote-engine`.
/// This struct exists to allow for upgrading older versions before loading the file in.

#[serde(rename = "rnotefile")]
pub struct Rnotefile {
    /// the document
    #[serde(rename = "document", alias = "sheet")]
    pub document: serde_json::Value,
    /// A snapshot of the store
    #[serde(rename = "store_snapshot")]
    pub store_snapshot: serde_json::Value,
}

impl FileFormatLoader for Rnotefile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Rnotefile> {
        let decompressed = String::from_utf8(decompress_from_gzip(bytes)?)?;

        let wrapped_rnote_file = serde_json::from_str::<RnotefileWrapper>(decompressed.as_str())?;

        // Conversions for older file format versions happens here
        if semver::Version::parse("0.5.8").unwrap() == wrapped_rnote_file.version {
            Ok(Self::try_from(serde_json::from_value::<
                RnoteFileMaj0Min5Patch8,
            >(wrapped_rnote_file.data)?)?)
        } else if semver::VersionReq::parse(">=0.5.0")
            .unwrap()
            .matches(&wrapped_rnote_file.version)
        {
            Ok(serde_json::from_value::<Rnotefile>(
                wrapped_rnote_file.data,
            )?)
        } else {
            Err(anyhow::anyhow!(
                "failed to load rnote file from bytes, unsupported version: {}",
                wrapped_rnote_file.version
            ))
        }
    }
}

impl FileFormatSaver for Rnotefile {
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let output = RnotefileWrapper {
            version: semver::Version::parse("0.5.9").unwrap(),
            data: serde_json::to_value(self)?,
        };

        let compressed = compress_to_gzip(serde_json::to_string(&output)?.as_bytes(), file_name)?;

        Ok(compressed)
    }
}
