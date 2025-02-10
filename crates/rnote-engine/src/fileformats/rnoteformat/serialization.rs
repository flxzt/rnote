// Imports
use crate::engine::EngineSnapshot;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Serialization methods that can be applied to a snapshot of the engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerializationMethod {
    #[serde(rename = "bitcode")]
    Bitcode,
    #[serde(rename = "json")]
    Json,
}

impl Default for SerializationMethod {
    fn default() -> Self {
        Self::Json
    }
}

impl SerializationMethod {
    pub const VALID_STR_ARRAY: [&'static str; 5] = ["Bitcode", "bitcode", "Json", "JSON", "json"];

    /// Keeping this function to mimic the behaviour of CompressionMethod and forward-comptability
    pub fn is_similar_to(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
    pub fn serialize(&self, engine_snapshot: &EngineSnapshot) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Bitcode => Ok(bitcode::serialize(engine_snapshot)?),
            Self::Json => Ok(serde_json::to_vec(&ijson::to_value(engine_snapshot)?)?),
        }
    }
    pub fn deserialize(&self, data: &[u8]) -> anyhow::Result<EngineSnapshot> {
        match self {
            Self::Bitcode => Ok(bitcode::deserialize(data)?),
            Self::Json => Ok(ijson::from_value(&serde_json::from_slice(data)?)?),
        }
    }
}

impl FromStr for SerializationMethod {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bitcode" | "bitcode" => Ok(Self::Bitcode),
            "Json" | "JSON" | "json" => Ok(Self::Json),
            _ => Err("Unknown serialization method"),
        }
    }
}
