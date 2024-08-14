//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod legacy;
pub(crate) mod maj0min12;
pub(crate) mod maj0min5patch8;
pub(crate) mod maj0min5patch9;
pub(crate) mod maj0min6;
pub(crate) mod maj0min9;

use crate::engine::EngineSnapshot;

use super::{FileFormatLoader, FileFormatSaver};
use legacy::LegacyRnoteFile;
use maj0min12::RnoteFileMaj0Min12;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    usize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionMethods {
    None,
    Gzip,
    Zstd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationMethods {
    Bincode,
    Json,
}

/// Compress bytes with gzip.
fn compress_to_gzip(to_compress: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::<u8>::new(), flate2::Compression::new(5));
    encoder.write_all(to_compress)?;
    Ok(encoder.finish()?)
}

/// Decompress from gzip.
fn decompress_from_gzip(
    compressed: &[u8],
    uncompressed_siez: usize,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut bytes: Vec<u8> = Vec::with_capacity(uncompressed_siez);
    let mut decoder = flate2::read::MultiGzDecoder::new(compressed);
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Compress bytes with zstd
pub fn compress_to_zstd(to_compress: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut encoder = zstd::Encoder::new(Vec::<u8>::new(), 9)?;
    if let Ok(num_workers) = std::thread::available_parallelism() {
        encoder.multithread(num_workers.get() as u32)?;
    }
    encoder.write_all(to_compress)?;
    Ok(encoder.finish()?)
}

/// Decompress bytes with zstd
pub fn decompress_from_zstd(
    compressed: &[u8],
    uncompressed_siez: usize,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut bytes: Vec<u8> = Vec::with_capacity(uncompressed_siez);
    let mut decoder = zstd::Decoder::new(compressed)?;
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub type RnoteFile = maj0min12::RnoteFileMaj0Min12;
pub type RnoteHeader = maj0min12::RnoteHeaderMaj0Min12;

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let json_header = serde_json::to_vec(&self.header)?;
        let header = [
            &Self::MAGIC_NUMBER[..],
            &Self::VERSION[..],
            &u16::try_from(json_header.len())?.to_le_bytes(),
            &json_header,
        ]
        .concat();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_all(&header)?;
        buffer.write_all(&self.engine_snapshot)?;
        Ok(buffer)
    }
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let magic_number = bytes
            .get(..9)
            .ok_or(anyhow::anyhow!("Failed to get magic number"))?;

        if magic_number != Self::MAGIC_NUMBER {
            if magic_number[..2] == [0x1f, 0x8b] {
                return Ok(RnoteFile::try_from(LegacyRnoteFile::load_from_bytes(
                    bytes,
                )?)?);
            } else {
                Err(anyhow::anyhow!("Unkown file"))?;
            }
        }

        let mut version: [u8; 3] = [0; 3];
        version.copy_from_slice(
            bytes
                .get(9..12)
                .ok_or(anyhow::anyhow!("Failed to get version"))?,
        );

        let mut header_size: [u8; 2] = [0; 2];
        header_size.copy_from_slice(
            bytes
                .get(12..14)
                .ok_or(anyhow::anyhow!("Failed to get header size"))?,
        );
        let header_size = u16::from_le_bytes(header_size);

        let header_slice = bytes
            .get(14..14 + header_size as usize)
            .ok_or(anyhow::anyhow!("Missing header"))?;

        let body_slice = bytes
            .get(14 + header_size as usize..)
            .ok_or(anyhow::anyhow!("Missing body"))?;

        Ok(Self {
            header: serde_json::from_slice(header_slice)?,
            engine_snapshot: body_slice.to_vec(),
        })
    }
}

impl TryFrom<RnoteFileMaj0Min12> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFileMaj0Min12) -> Result<Self, Self::Error> {
        let uncompressed = match value.header.compression {
            CompressionMethods::None => value.engine_snapshot,
            CompressionMethods::Gzip => decompress_from_gzip(
                &value.engine_snapshot,
                usize::try_from(value.header.size).unwrap_or(usize::MAX),
            )?,
            CompressionMethods::Zstd => decompress_from_zstd(
                &value.engine_snapshot,
                usize::try_from(value.header.size).unwrap_or(usize::MAX),
            )?,
        };

        match value.header.serialization {
            SerializationMethods::Json => Ok(ijson::from_value(&serde_json::from_slice::<
                ijson::IValue,
            >(&uncompressed)?)?),
            SerializationMethods::Bincode => unreachable!(),
        }
    }
}

impl TryFrom<EngineSnapshot> for RnoteFile {
    type Error = anyhow::Error;

    fn try_from(value: EngineSnapshot) -> Result<Self, Self::Error> {
        let json = serde_json::to_vec(&ijson::to_value(value)?)?;
        let size = json.len() as u64;

        let engine_snapshot = compress_to_zstd(&json)?;

        Ok(Self {
            header: RnoteHeader {
                compression: CompressionMethods::Zstd,
                serialization: SerializationMethods::Json,
                size,
            },
            engine_snapshot,
        })
    }
}
