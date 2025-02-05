// Imports
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    str::FromStr,
};

/// Compression methods that can be applied to the serialized engine snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMethod {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "gzip")]
    Gzip(u8),
    /// Zstd supports negative compression levels but I don't see the point in allowing these for Rnote files
    #[serde(rename = "zstd")]
    Zstd(u8),
}

impl Default for CompressionMethod {
    fn default() -> Self {
        Self::Zstd(9)
    }
}

#[derive(Debug, Clone, Copy, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum CompressionLevel {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
    None,
}

impl TryFrom<u32> for CompressionLevel {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "CompressionLevel try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

impl CompressionMethod {
    pub const VALID_STR_ARRAY: [&'static str; 6] = ["None", "none", "Gzip", "gzip", "Zstd", "zstd"];

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
    pub fn get_compression_level(&self) -> CompressionLevel {
        match self {
            Self::None => CompressionLevel::None,
            Self::Gzip(val) => match *val {
                0..=1 => CompressionLevel::VeryLow,
                2..=3 => CompressionLevel::Low,
                4..=5 => CompressionLevel::Medium,
                6..=7 => CompressionLevel::High,
                8..=9 => CompressionLevel::VeryHigh,
                _ => unreachable!(),
            },
            Self::Zstd(val) => match *val {
                0..=4 => CompressionLevel::VeryLow,
                5..=8 => CompressionLevel::Low,
                9..=12 => CompressionLevel::Medium,
                13..=16 => CompressionLevel::High,
                17..=22 => CompressionLevel::VeryHigh,
                _ => unreachable!(),
            },
        }
    }

    pub fn set_compression_level(&mut self, level: CompressionLevel) {
        match self {
            Self::None => (),
            Self::Gzip(ref mut val) => {
                *val = match level {
                    CompressionLevel::VeryHigh => 8,
                    CompressionLevel::High => 6,
                    CompressionLevel::Medium => 5,
                    CompressionLevel::Low => 3,
                    CompressionLevel::VeryLow => 1,
                    CompressionLevel::None => unreachable!(),
                }
            }
            Self::Zstd(ref mut val) => {
                *val = match level {
                    CompressionLevel::VeryHigh => 17,
                    CompressionLevel::High => 13,
                    CompressionLevel::Medium => 9,
                    CompressionLevel::Low => 5,
                    CompressionLevel::VeryLow => 1,
                    CompressionLevel::None => unreachable!(),
                }
            }
        }
    }
}

impl FromStr for CompressionMethod {
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
