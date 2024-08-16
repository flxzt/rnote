use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    str::FromStr,
};

use crate::engine::EngineSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Compression methods that can be applied to the serialized engine snapshot
pub enum CompM {
    None,
    Gzip,
    Zstd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Serialization methods that can be applied to a snapshot of the engine
pub enum SerM {
    Bincode,
    Bitcode,
    Cbor,
    Json,
    Postcard,
    Toml,
}

impl CompM {
    pub fn compress(&self, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::None => Ok(data),
            Self::Gzip => {
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(5));
                encoder.write_all(&data)?;
                Ok(encoder.finish()?)
            }
            Self::Zstd => {
                let mut encoder = zstd::Encoder::new(Vec::<u8>::new(), 14)?;
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
            Self::Gzip => {
                let mut bytes: Vec<u8> = Vec::with_capacity(uc_size);
                let mut decoder = flate2::read::MultiGzDecoder::new(&data[..]);
                decoder.read_to_end(&mut bytes)?;
                Ok(bytes)
            }
            Self::Zstd => {
                let mut bytes: Vec<u8> = Vec::with_capacity(uc_size);
                let mut decoder = zstd::Decoder::new(&data[..])?;
                decoder.read_to_end(&mut bytes)?;
                Ok(bytes)
            }
        }
    }
}

impl SerM {
    pub fn serialize(&self, engine_snapshot: &EngineSnapshot) -> anyhow::Result<Vec<u8>> {
        let res = match self {
            Self::Bincode => Ok::<Vec<u8>, anyhow::Error>(bincode::serialize(engine_snapshot)?),
            Self::Bitcode => Ok(bitcode::serialize(engine_snapshot)?),
            Self::Cbor => {
                let mut writer = Vec::new();
                ciborium::into_writer(engine_snapshot, &mut writer)?;
                Ok(writer)
            }
            Self::Json => Ok(serde_json::to_vec(&ijson::to_value(engine_snapshot)?)?),
            Self::Postcard => Ok(postcard::to_allocvec(engine_snapshot)?),
            Self::Toml => Ok(toml::to_string(engine_snapshot)?.into_bytes()),
        }?;
        tracing::info!("{} MB", res.len() as f64 / 1e6);
        Ok(res)
    }
    pub fn deserialize(&self, data: &[u8]) -> anyhow::Result<EngineSnapshot> {
        match self {
            Self::Bincode => Ok(bincode::deserialize(data)?),
            Self::Bitcode => Ok(bitcode::deserialize(data)?),
            Self::Cbor => Ok(ciborium::from_reader(data)?),
            Self::Json => Ok(ijson::from_value(&serde_json::from_slice(data)?)?),
            Self::Postcard => Ok(postcard::from_bytes(data)?),
            Self::Toml => Ok(toml::from_str(std::str::from_utf8(data)?)?),
        }
    }
}

impl FromStr for SerM {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bincode" => Ok(Self::Bincode),
            "Bitcode" => Ok(Self::Bitcode),
            "Cbor" => Ok(Self::Cbor),
            "Json" => Ok(Self::Json),
            "Postcard" => Ok(Self::Postcard),
            "Toml" => Ok(Self::Toml),
            _ => Err("Unknown SerM"),
        }
    }
}
