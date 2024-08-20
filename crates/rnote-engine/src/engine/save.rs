use serde::{Deserialize, Serialize};

use crate::fileformats::rnoteformat::{
    methods::{CompM, SerM},
    RnoteHeader,
};

// Rnote file save preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "save_prefs")]
pub struct SavePrefs {
    #[serde(rename = "serialization")]
    pub serialization: SerM,
    #[serde(rename = "compression")]
    pub compression: CompM,
    #[serde(rename = "method_lock")]
    pub method_lock: bool,
}

impl SavePrefs {
    pub fn clone_config(&self) -> Self {
        self.clone()
    }
    pub fn conforms_to_default(&self) -> bool {
        std::mem::discriminant(&self.serialization) == std::mem::discriminant(&SerM::default())
            && std::mem::discriminant(&self.compression)
                == std::mem::discriminant(&CompM::default())
    }
    /// The EngineExport should only contain SavePrefs that conform to the default
    /// otherwise, for example, new files could be created without any compression and encoded in JSON
    pub fn clone_conformed_config(&self) -> Self {
        if self.conforms_to_default() {
            self.clone_config()
        } else {
            Self::default()
        }
    }
}

impl Default for SavePrefs {
    fn default() -> Self {
        Self {
            serialization: SerM::default(),
            compression: CompM::default(),
            method_lock: false,
        }
    }
}

impl From<RnoteHeader> for SavePrefs {
    fn from(value: RnoteHeader) -> Self {
        Self {
            serialization: value.serialization,
            compression: value.compression,
            method_lock: value.method_lock,
        }
    }
}

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
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

impl CompM {
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
