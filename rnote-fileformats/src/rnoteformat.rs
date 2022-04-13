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

/// A .rnote file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFile {
    /// The version of the file
    // TODO: make the version strongly typed
    pub version: String,
    /// The sheet
    pub sheet: serde_json::Value,
    /// expand mode
    pub expand_mode: serde_json::Value,
    /// strokes state
    pub strokes_state: serde_json::Value,
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let decompressed = String::from_utf8(decompress_from_gzip(&bytes)?)?;
        let file = serde_json::from_str::<Self>(&decompressed)?;

        // Conversions for older file format versions happens here
        match file.version.as_str() {
            "0.5.0" => Ok(file),
            version => Err(anyhow::anyhow!(
                "failed to load rnote file from bytes, invalid version: {}",
                version
            )),
        }
    }
}

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, file_name: &str) -> Result<Vec<u8>, anyhow::Error> {
        let output = serde_json::to_string(self)?;
        let compressed = compress_to_gzip(output.as_bytes(), file_name)?;

        Ok(compressed)
    }
}
