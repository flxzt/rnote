use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    str::FromStr,
};

use crate::engine::EngineSnapshot;

/// Compression methods that can be applied to the serialized engine snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "compression_method")]
pub enum CompM {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "gzip")]
    Gzip(u8),
    /// Zstd supports negative compression levels but I don't see the point in allowing these for Rnote files
    #[serde(rename = "zstd")]
    Zstd(u8),
}

/// Serialization methods that can be applied to a snapshot of the engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "serialization_method")]
pub enum SerM {
    #[serde(rename = "bincode")]
    Bincode,
    #[serde(rename = "bitcode")]
    Bitcode,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "postcard")]
    Postcard,
}

impl CompM {
    pub fn compress(&self, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::None => Ok(data),
            Self::Gzip(compression_level) => {
                let mut encoder = flate2::write::GzEncoder::new(
                    Vec::new(),
                    flate2::Compression::new(u32::from(*compression_level)),
                );
                encoder.write_all(&data)?;
                Ok(encoder.finish()?)
            }
            Self::Zstd(compression_level) => {
                let mut encoder =
                    zstd::Encoder::new(Vec::<u8>::new(), i32::from(*compression_level))?;
                if let Ok(num_workers) = std::thread::available_parallelism() {
                    encoder.multithread(num_workers.get() as u32)?;
                }
                encoder.write_all(&data)?;
                Ok(encoder.finish()?)
            }
        }
    }
    pub fn decompress(&self, uc_size: usize, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::None => Ok(data),
            Self::Gzip { .. } => {
                let mut bytes: Vec<u8> = Vec::with_capacity(uc_size);
                let mut decoder = flate2::read::MultiGzDecoder::new(&data[..]);
                decoder.read_to_end(&mut bytes)?;
                Ok(bytes)
            }
            Self::Zstd { .. } => {
                let mut bytes: Vec<u8> = Vec::with_capacity(uc_size);
                let mut decoder = zstd::Decoder::new(&data[..])?;
                decoder.read_to_end(&mut bytes)?;
                Ok(bytes)
            }
        }
    }
    pub fn update_compression_level(&mut self, new: u8) -> anyhow::Result<()> {
        match self {
            Self::None => {
                tracing::warn!("Cannot update the compression level of 'None'");
                Ok(())
            }
            Self::Gzip(ref mut curr) => {
                if !(0..=9).contains(&new) {
                    Err(anyhow::anyhow!(
                        "Invalid compression level for Gzip, expected a value between 0 and 9"
                    ))
                } else {
                    *curr = new;
                    Ok(())
                }
            }
            Self::Zstd(ref mut curr) => {
                if !zstd::compression_level_range().contains(&i32::from(new)) {
                    Err(anyhow::anyhow!(
                        "Invalid compression level for Zstd, expected a value between 0 and 22"
                    ))
                } else {
                    *curr = new;
                    Ok(())
                }
            }
        }
    }
    pub const VALID_STR_ARRAY: [&'static str; 6] = ["None", "none", "Gzip", "gzip", "Zstd", "zstd"];
}

impl Default for CompM {
    fn default() -> Self {
        Self::Zstd(9)
    }
}

impl FromStr for CompM {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" | "none" => Ok(Self::None),
            "Gzip" | "gzip" => Ok(Self::Gzip(5)),
            "Zstd" | "zstd" => Ok(Self::Zstd(9)),
            _ => Err("Unknown compression method"),
        }
    }
}

impl SerM {
    pub fn serialize(&self, engine_snapshot: &EngineSnapshot) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Bincode => Ok::<Vec<u8>, anyhow::Error>(bincode::serialize(engine_snapshot)?),
            Self::Bitcode => Ok(bitcode::serialize(engine_snapshot)?),
            Self::Json => Ok(serde_json::to_vec(&ijson::to_value(engine_snapshot)?)?),
            Self::Postcard => Ok(postcard::to_allocvec(engine_snapshot)?),
        }
    }
    pub fn deserialize(&self, data: &[u8]) -> anyhow::Result<EngineSnapshot> {
        match self {
            Self::Bincode => Ok(bincode::deserialize(data)?),
            Self::Bitcode => Ok(bitcode::deserialize(data)?),
            Self::Json => Ok(ijson::from_value(&serde_json::from_slice(data)?)?),
            Self::Postcard => Ok(postcard::from_bytes(data)?),
        }
    }
    pub const VALID_STR_ARRAY: [&'static str; 9] = [
        "Bincode", "bincode", "Bitcode", "bitcode", "Json", "JSON", "json", "Postcard", "postcard",
    ];
}

impl Default for SerM {
    fn default() -> Self {
        Self::Bitcode
    }
}

impl FromStr for SerM {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bincode" | "bincode" => Ok(Self::Bincode),
            "Bitcode" | "bitcode" => Ok(Self::Bitcode),
            "Json" | "JSON" | "json" => Ok(Self::Json),
            "Postcard" | "postcard" => Ok(Self::Postcard),
            _ => Err("Unknown serialization method"),
        }
    }
}
