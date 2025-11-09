use anyhow::anyhow;
// Imports
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    str::FromStr,
};

type GzipCompressionInteger = deranged::RangedI32<1, 9>; // Technically the level `0` is supported but it just means there is no compression
static DEFAULT_GZIP_COMPRESSION_INTEGER: GzipCompressionInteger =
    GzipCompressionInteger::new_static::<5>();

type ZstdCompressionInteger = deranged::RangedI32<-7, 22>;
static DEFAULT_ZSTD_COMPRESSION_INTEGER: ZstdCompressionInteger =
    ZstdCompressionInteger::new_static::<12>();

/// Compression methods that can be applied to the serialized engine snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMethod {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "gzip")]
    Gzip(GzipCompressionInteger),
    #[serde(rename = "zstd")]
    Zstd(ZstdCompressionInteger),
}

impl Default for CompressionMethod {
    fn default() -> Self {
        Self::Zstd(DEFAULT_ZSTD_COMPRESSION_INTEGER)
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
            Self::Gzip(comp_int) => {
                let mut encoder = flate2::write::GzEncoder::new(
                    Vec::new(),
                    flate2::Compression::new(comp_int.get() as u32),
                );
                encoder.write_all(&data)?;
                Ok(encoder.finish()?)
            }
            Self::Zstd(comp_int) => {
                zstd::bulk::compress(&data, comp_int.get()).map_err(anyhow::Error::from)
            }
        }
    }

    pub fn decompress(&self, uc_size: usize, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::None => Ok(data.to_vec()),
            Self::Gzip { .. } => {
                let mut bytes: Vec<u8> = Vec::with_capacity(uc_size);
                let mut decoder = flate2::read::MultiGzDecoder::new(data);
                decoder.read_to_end(&mut bytes)?;
                Ok(bytes)
            }
            Self::Zstd { .. } => zstd::bulk::decompress(data, uc_size).map_err(anyhow::Error::from),
        }
    }

    pub fn update_compression_integer(&mut self, new: i32) -> anyhow::Result<()> {
        match self {
            Self::None => {
                tracing::warn!("Cannot update the compression level of `None`");
            }
            Self::Gzip(curr) => {
                *curr = GzipCompressionInteger::new(new).ok_or_else(|| {
                    anyhow!("Invalid compression level for Gzip, expected a value between 0 and 9")
                })?;
            }
            Self::Zstd(curr) => {
                *curr = ZstdCompressionInteger::new(new).ok_or_else(|| {
                    anyhow!(
                        "Invalid compression level for Zstd, expected a value between -7 and 22"
                    )
                })?;
            }
        }
        Ok(())
    }

    pub fn get_compression_level(&self) -> CompressionLevel {
        match self {
            Self::None => CompressionLevel::None,
            Self::Gzip(val) => match val.get() {
                1 => CompressionLevel::VeryLow,
                2..=3 => CompressionLevel::Low,
                4..=5 => CompressionLevel::Medium,
                6..=7 => CompressionLevel::High,
                8..=9 => CompressionLevel::VeryHigh,
                _ => unreachable!(),
            },
            Self::Zstd(val) => match val.get() {
                -7..=5 => CompressionLevel::VeryLow,
                6..=9 => CompressionLevel::Low,
                10..=13 => CompressionLevel::Medium,
                14..=17 => CompressionLevel::High,
                18..=22 => CompressionLevel::VeryHigh,
                _ => unreachable!(),
            },
        }
    }

    pub fn set_compression_level(&mut self, level: &CompressionLevel) {
        match self {
            Self::None => tracing::warn!(
                "Attempting to set the compression level for `CompressionMethod::None` "
            ),
            Self::Gzip(comp_int) => match level {
                CompressionLevel::VeryHigh => *comp_int = GzipCompressionInteger::new_static::<8>(),
                CompressionLevel::High => *comp_int = GzipCompressionInteger::new_static::<6>(),
                CompressionLevel::Medium => *comp_int = GzipCompressionInteger::new_static::<5>(),
                CompressionLevel::Low => *comp_int = GzipCompressionInteger::new_static::<3>(),
                CompressionLevel::VeryLow => *comp_int = GzipCompressionInteger::new_static::<1>(),
                CompressionLevel::None => tracing::warn!(
                    "Attempting to set the compression level for `CompressionMethod::Gzip` to `None` "
                ),
            },
            Self::Zstd(comp_int) => match level {
                CompressionLevel::VeryHigh => {
                    *comp_int = ZstdCompressionInteger::new_static::<20>()
                }
                CompressionLevel::High => *comp_int = ZstdCompressionInteger::new_static::<16>(),
                CompressionLevel::Medium => *comp_int = ZstdCompressionInteger::new_static::<12>(),
                CompressionLevel::Low => *comp_int = ZstdCompressionInteger::new_static::<8>(),
                CompressionLevel::VeryLow => *comp_int = ZstdCompressionInteger::new_static::<3>(),
                CompressionLevel::None => tracing::warn!(
                    "Attempting to set the compression level for `CompressionMethod::Zstd` to `None` "
                ),
            },
        }
    }

    pub fn with_compression_level(mut self, level: &CompressionLevel) -> Self {
        self.set_compression_level(level);
        self
    }
}

impl FromStr for CompressionMethod {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" | "none" => Ok(Self::None),
            "Gzip" | "gzip" => Ok(Self::Gzip(DEFAULT_GZIP_COMPRESSION_INTEGER)),
            "Zstd" | "zstd" => Ok(Self::Zstd(DEFAULT_ZSTD_COMPRESSION_INTEGER)),
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
