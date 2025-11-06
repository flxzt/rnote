use super::compression::CompressionMethod;
use crate::{
    fileformats::rnoteformat::{bcursor::BCursor, legacy::LegacyRnoteFile},
    store::{ChronoComponent, StrokeKey},
    strokes::Stroke,
};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};
use std::sync::Arc;

/// Using "RnoteFile" might be a bit of a misnomer, as this struct is an intermediate representation
/// between the Rnote `EngineSnapshot` and the actual `.rnote` file (bytes)
#[derive(Debug, Clone)]
pub struct RnoteFileMaj0Min14 {
    pub compression_method: CompressionMethod,
    pub compression_lock: bool,
    pub engine_snapshot_ir: EngineSnaphotIR,
}

/// Intermediate representation of the `EngineSnapshot`
#[derive(Debug, Clone)]
pub struct EngineSnaphotIR {
    /// A vector of chunks, where each chunk is an array of (`Stroke`, `ChronoComponent`) converted to an `IValue`
    pub strokechrono_chunks: Vec<ijson::IValue>,
    /// The `EngineSnapshot` without `stroke_components` and `chrono_components` (still there but empty) converted to an `IValue`
    pub core: ijson::IValue,
}

/// Information about a specific chunk, or the core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// size of the serialized and compressed chunk
    pub c_size: usize,
    /// size of the serialized (and uncompressed) chunk
    pub uc_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "header")]
pub struct RnoteHeaderMaj0Min14 {
    /// method used to compress/decompress the body
    #[serde(rename = "compression_method")]
    pub compression_method: CompressionMethod,
    /// if set to true, the file can use non-standard compression and will not be forced back into using defaults
    #[serde(rename = "compression_lock")]
    pub compression_lock: bool,
    /// information required to handle the chunked body, the last element corresponds to the `core`
    /// only used when going from and to file representation, can be left empty otherwise
    #[serde(rename = "chunk_info_vec")]
    pub chunk_info_vec: Vec<ChunkInfo>,
}

type EngineStrokes = Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>;
type EngineChronos = Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>;

/// This function pairs, chunks, then serializes (in parallel) `stroke_components` and `chrono_components` obtained from an `EngineSnapshot`
pub fn chunk_serialize(
    engine_strokes: EngineStrokes,
    engine_chronos: EngineChronos,
) -> Result<Vec<ijson::IValue>, serde_json::Error> {
    let strokechrono_vec = engine_strokes
        .iter()
        .map(|(key, stroke)| (stroke, engine_chronos.get(key).unwrap()))
        .collect_vec();

    // The number of chunks to split `strokechrono_vec` into
    let n = rayon::current_num_threads();
    tracing::debug!("Splitting `strokechrono_vec` into {n} parts");

    let len = strokechrono_vec.len();
    let base = len / n;
    let rem = len % n;

    let mut pre_chunks = Vec::with_capacity(n);
    let mut start: usize = 0;

    for i in 0..n {
        let size = base + if i < rem { 1 } else { 0 };
        let end = start + size;
        pre_chunks.push(&strokechrono_vec[start..end]);
        start = end;
    }

    pre_chunks
        .into_par_iter()
        .map(ijson::to_value)
        .collect::<Result<Vec<ijson::IValue>, serde_json::Error>>()
}

impl RnoteFileMaj0Min14 {
    pub(super) fn load(mut cursor: BCursor, header_size: usize) -> anyhow::Result<Self> {
        let header: RnoteHeaderMaj0Min14 =
            ijson::from_value(&serde_json::from_slice(cursor.try_capture(header_size)?)?)?;

        // Not the most readable block of code but quite functional, we first gather
        // every serialized and compressed chunk with their respective uncompressed size,
        // to then decompress and deserialize them in parallel afterward.
        let strokechrono_chunks_and_core = header
            .chunk_info_vec
            .into_iter()
            .map(|info| {
                cursor
                    .try_capture(info.c_size)
                    .map(|slice| (slice, info.uc_size))
            })
            .collect::<Result<Vec<(&[u8], usize)>, anyhow::Error>>()?
            .into_par_iter()
            .map(|(slice, uc_size)| {
                header
                    .compression_method
                    .decompress(uc_size, slice)
                    .and_then(|uc_slice| {
                        serde_json::from_slice::<ijson::IValue>(&uc_slice)
                            .map_err(anyhow::Error::from)
                    })
            })
            .collect::<Result<Vec<ijson::IValue>, anyhow::Error>>()?;

        fn split_last_owned(mut input: Vec<ijson::IValue>) -> (Vec<ijson::IValue>, ijson::IValue) {
            let last = input.pop().unwrap();
            (input, last)
        }

        let (strokechrono_chunks, core) = split_last_owned(strokechrono_chunks_and_core);

        Ok(Self {
            compression_method: header.compression_method,
            compression_lock: header.compression_lock,
            engine_snapshot_ir: EngineSnaphotIR {
                strokechrono_chunks,
                core,
            },
        })
    }
}

impl TryFrom<LegacyRnoteFile> for RnoteFileMaj0Min14 {
    type Error = anyhow::Error;

    fn try_from(mut value: LegacyRnoteFile) -> Result<Self, Self::Error> {
        let engine_snapshot = value
            .engine_snapshot
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("engine snapshot is not a JSON object"))?;

        let extracted_strokes: EngineStrokes =
            ijson::from_value(&engine_snapshot.remove("stroke_components").ok_or_else(|| {
                anyhow::anyhow!("`engine_snapshot` has no value `stroke_components`")
            })?)?;

        engine_snapshot.insert(
            "stroke_components",
            ijson::to_value::<EngineStrokes>(Default::default()).unwrap(),
        );

        let extracted_chronos: EngineChronos =
            ijson::from_value(&engine_snapshot.remove("chrono_components").ok_or_else(|| {
                anyhow::anyhow!("`engine_snapshot` has no value `chrono_components`")
            })?)?;

        engine_snapshot.insert(
            "chrono_components",
            ijson::to_value::<EngineChronos>(Default::default()).unwrap(),
        );

        let strokechrono_chunks = chunk_serialize(extracted_strokes, extracted_chronos)?;

        Ok(Self {
            compression_method: CompressionMethod::default(), // TODO: SavePrefs
            compression_lock: false,
            engine_snapshot_ir: EngineSnaphotIR {
                strokechrono_chunks,
                core: value.engine_snapshot,
            },
        })
    }
}
