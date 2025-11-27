//! # Rnote File Format (v1)
//!
//! This module contains almost all the implementation for loading and saving the current
//! Rnote file format. It supersedes the older logic found in the `legacy` module.
//!
//! ## Saving
//!
//! In contrast to the legacy implementation, saving no longer serializes the
//! entire [`EngineSnapshot`] through an intermediate JSON representation ([`ijson::IValue`]).
//!
//! Instead:
//!
//! 1. The `stroke_components` and `chrono_components` are extracted from the snapshot.
//! 2. They are grouped together, chunked, and serialized in parallel directly to bytes.
//! 3. The chunks are compressed (still in parallel of course) using [`zstd`] (instead of `gzip`).
//! 4. The same (serialization + compression) is performed on what remains of the [`EngineSnapshot`].
//! 5. The [`Prelude`], and [`RnoteFileHeaderV1`] are created, bytes are concatenated together and finally returned.
//!
//! See the method [`RnoteFileInterfaceV1::engine_snapshot_to_bytes`] in `save.rs` for more details.
//!
//! ## Loading
//!
//! Files (`.rnote`) begin with a small prelude that records the version of Rnote
//! that last saved them, this determines if the file can be deserialized directly or if
//! going through a compatibility layer (the `CompatV1` struct and its wrappers) is needed.
//!
//! - If the corresponding [`EngineSnapshot`] format has not changed (or changes can be handled
//!   via serde attributes), then the file can be deserialized directly into [`EngineSnapshot`].
//!
//! - If changes require intervention, the file is first loaded into [`CompatV1`], a glorified
//!   container of multiple [`ijson::IValue`]. It is then wrapped in the appropriate [`CompatV1For`],
//!   which provides type-level context on the last version of Rnote that directly supported it.
//!   Using `TryFrom` implementations, the [`CompatV1For`] struct is incrementally upgraded until it
//!   can finally be converted into a fully valid [`EngineSnapshot`].
//!
//! Just as before, the decompression and deserialization is performed in parallel.
//!
//! See `save.rs` as well as `version.rs` for more details.

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
