//! Loading and saving Rnote's `.rnote` file format

// Modules
mod load;
mod save;
mod version;

// Re-exports
#[allow(unused_imports)]
pub use version::*;

// Imports
use super::compression::CompressionMethod;
use crate::{
    engine::EngineSnapshot,
    fileformats::rnoteformat::{bcursor::BCursor, legacy::LegacyRnoteFile, prelude::Prelude},
    store::{ChronoComponent, StrokeKey},
    strokes::Stroke,
};
use anyhow::Ok;
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use slotmap::{HopSlotMap, SecondaryMap};
use std::{cell::RefCell, sync::Arc};
use thread_local::ThreadLocal;

/// An interface used to manage the saving and loading of `.rnote` files.
#[derive(Debug)]
pub(crate) struct RnoteFileInterfaceV1;

impl RnoteFileInterfaceV1 {
    pub const FILE_VERSION: u16 = 1;
}

/// Intermediate representation of the `EngineSnapshot` for save compatibility, see `version.rs` for more info
#[derive(Debug, Clone)]
pub(crate) struct CompatV1 {
    /// The `EngineSnapshot` without `stroke_components` and `chrono_components` (still there but empty) represented as an `IValue`
    pub engine_snapshot_gutted: ijson::IValue,
    /// A vector of chunks, where each chunk is an array of `(Stroke, ChronoComponent)` values, represented as an `IValue`
    pub stroke_chrono_pair_chunks: Vec<ijson::IValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnote_file_header")]
struct RnoteFileHeaderV1 {
    #[serde(rename = "compression_method")]
    pub compression_method: CompressionMethod,
    #[serde(rename = "core_info")]
    pub core_info: ChunkInfo,
    #[serde(rename = "chunk_info_vec")]
    pub chunk_info_vec: Vec<ChunkInfo>,
}

/// Information about the size of a specific chunk (or the core).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "chunk_info")]
struct ChunkInfo {
    /// Size of the serialized and compressed chunk.
    pub c_size: usize,
    /// Size of the serialized (but uncompressed) chunk
    /// Note: somewhat redundant as Zstd can store this, but annoying to extract
    pub uc_size: usize,
}
