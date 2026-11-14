// Imports
use serde::{Deserialize, Serialize};

pub type ZstdCompressionInteger = deranged::RangedI32<-7, 22>;
pub static DEFAULT_ZSTD_COMPRESSION_INTEGER: ZstdCompressionInteger =
    ZstdCompressionInteger::new_static::<9>();

/// Compression methods that can be applied to the serialized engine snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMethod {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "zstd")]
    Zstd(ZstdCompressionInteger),
}

impl Default for CompressionMethod {
    fn default() -> Self {
        Self::Zstd(DEFAULT_ZSTD_COMPRESSION_INTEGER)
    }
}

impl CompressionMethod {
    pub fn compress(&self, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::None => Ok(data),
            Self::Zstd(comp_int) => {
                zstd::bulk::compress(&data, comp_int.get()).map_err(anyhow::Error::from)
            }
        }
    }

    pub fn compress_mut(&self, data: &mut Vec<u8>) -> anyhow::Result<()> {
        if let Self::Zstd(comp_int) = self {
            *data = zstd::bulk::compress(data, comp_int.get())?
        }
        Ok(())
    }

    pub fn decompress(&self, data: &[u8], uc_size: usize) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::None => Ok(data.to_vec()),
            Self::Zstd { .. } => zstd::bulk::decompress(data, uc_size).map_err(anyhow::Error::from),
        }
    }

    pub fn update_compression_integer(&mut self, new: i32) -> anyhow::Result<()> {
        match self {
            Self::None => {
                tracing::warn!("Cannot update the compression level of `None`");
            }
            Self::Zstd(curr) => {
                *curr = ZstdCompressionInteger::new(new).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Invalid compression level for Zstd, expected a value between -7 and 22"
                    )
                })?;
            }
        }
        Ok(())
    }
}
