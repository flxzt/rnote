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
    #[serde(rename = "zstd")]
    Zstd(u8),
}

impl Default for CompressionMethod {
    fn default() -> Self {
        Self::Zstd(12)
    }
}

impl CompressionMethod {
    pub const VALID_STR_ARRAY: [&'static str; 6] = ["None", "none", "Gzip", "gzip", "Zstd", "zstd"];
    pub fn is_similar_to(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
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
                let _ = encoder.set_parameter(
                    zstd::zstd_safe::CParameter::EnableLongDistanceMatching(true),
                );
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
    pub fn update_compression_integer(&mut self, new: u8) -> anyhow::Result<()> {
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
                        "Invalid compression level for Zstd, expected a value between 1 and 22"
                    ))
                } else {
                    *curr = new;
                    Ok(())
                }
            }
        }
    }

    // Uses unreachable!() as this function is only used by rnote-ui in a coherent way.
    pub fn get_compression_level(&self) -> CompressionLevel {
        match self {
            Self::None => CompressionLevel::None,
            Self::Gzip(val) => match *val {
                0..=1 => CompressionLevel::VeryLow,
                2..=3 => CompressionLevel::Low,
                4..=5 => CompressionLevel::Medium,
                6..=7 => CompressionLevel::High,
                8..=9 => CompressionLevel::VeryHigh,
                10.. => {
                    tracing::warn!("Compression integer of {self:?} is greater than expected");
                    CompressionLevel::VeryHigh
                }
            },
            Self::Zstd(val) => match *val {
                1..=5 => CompressionLevel::VeryLow,
                6..=9 => CompressionLevel::Low,
                10..=13 => CompressionLevel::Medium,
                14..=17 => CompressionLevel::High,
                18..=22 => CompressionLevel::VeryHigh,
                0 => {
                    tracing::warn!("Compression integer of {self:?} is lower than expected");
                    CompressionLevel::VeryLow
                }
                23.. => {
                    tracing::warn!("Compression integer of {self:?} is greater than expected");
                    CompressionLevel::VeryHigh
                }
            },
        }
    }
    fn get_compression_integer_from_compression_level(&self, level: &CompressionLevel) -> u8 {
        match self {
            Self::None => 0,
            Self::Gzip(..) => match level {
                &CompressionLevel::VeryHigh => 8,
                &CompressionLevel::High => 6,
                &CompressionLevel::Medium => 5,
                &CompressionLevel::Low => 3,
                &CompressionLevel::VeryLow => 1,
                &CompressionLevel::None => unreachable!(),
            },
            Self::Zstd(..) => match level {
                &CompressionLevel::VeryHigh => 20,
                &CompressionLevel::High => 16,
                &CompressionLevel::Medium => 12,
                &CompressionLevel::Low => 8,
                &CompressionLevel::VeryLow => 3,
                &CompressionLevel::None => unreachable!(),
            },
        }
    }
    pub fn set_compression_level(&mut self, level: &CompressionLevel) {
        let new_integer = self.get_compression_integer_from_compression_level(&level);
        match self {
            Self::None => unreachable!(),
            Self::Gzip(ref mut integer) | Self::Zstd(ref mut integer) => *integer = new_integer,
        }
    }

    pub fn clone_with_new_compression_level(&self, level: &CompressionLevel) -> Self {
        let new_integer = self.get_compression_integer_from_compression_level(level);
        match self {
            Self::None => Self::None,
            Self::Gzip(..) => Self::Gzip(new_integer),
            Self::Zstd(..) => Self::Zstd(new_integer),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
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
