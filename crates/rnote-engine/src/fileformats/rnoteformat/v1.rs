// Imports
use super::compression::CompressionMethod;
use crate::{
    engine::EngineSnapshot,
    fileformats::rnoteformat::{bcursor::BCursor, legacy::LegacyRnoteFile, prelude::Prelude},
    store::{ChronoComponent, StrokeKey},
    strokes::Stroke,
};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};
use std::sync::Arc;

/// Using "RnoteFile" might be a bit of a misnomer, as this struct is an intermediate representation
/// between the `EngineSnapshot` and the actual `.rnote` file (bytes)
#[derive(Debug, Clone)]
pub struct RnoteFileV1 {
    pub compression_method: CompressionMethod,
    pub compression_lock: bool,
    pub engine_snapshot_ir: EngineSnaphotIR,
}

/// Intermediate representation of the `EngineSnapshot`
#[derive(Debug, Clone)]
pub struct EngineSnaphotIR {
    /// A vector of chunks, where each chunk is an array of `(Stroke, ChronoComponent)` values, converted to an `IValue`
    pub strokechrono_chunks: Vec<ijson::IValue>,
    /// The `EngineSnapshot` without `stroke_components` and `chrono_components` (still there but empty) converted to an `IValue
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
#[serde(rename = "rnote_header")]
pub struct RnoteHeaderV1 {
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

impl RnoteFileV1 {
    const FILE_VERSION: u16 = 1;

    pub(super) fn load(mut cursor: BCursor, header_size: usize) -> anyhow::Result<Self> {
        let header: RnoteHeaderV1 =
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

impl TryFrom<LegacyRnoteFile> for RnoteFileV1 {
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
            compression_method: CompressionMethod::default(),
            compression_lock: false,
            engine_snapshot_ir: EngineSnaphotIR {
                strokechrono_chunks,
                core: value.engine_snapshot,
            },
        })
    }
}

// --- ✀ ---
// The code below this line could be commented out after a new file version is implemented

impl RnoteFileV1 {
    pub(super) fn save(&self) -> anyhow::Result<Vec<u8>> {
        let (body, chunk_info_vec): (Vec<Vec<u8>>, Vec<ChunkInfo>) = self
            .engine_snapshot_ir
            .strokechrono_chunks
            .par_iter()
            .chain(rayon::iter::once(&self.engine_snapshot_ir.core))
            .map(|ival| {
                let uc_data = serde_json::to_vec(ival)?;
                let uc_size = uc_data.len();
                let c_data = self.compression_method.compress(uc_data)?;
                let c_size = c_data.len();
                Ok((c_data, ChunkInfo { c_size, uc_size }))
            })
            .collect::<Result<Vec<(Vec<u8>, ChunkInfo)>, anyhow::Error>>()?
            .into_iter()
            .unzip();

        let body_bytes = body.concat();

        let header = RnoteHeaderV1 {
            compression_method: self.compression_method,
            compression_lock: self.compression_lock,
            chunk_info_vec,
        };
        let header_bytes = serde_json::to_vec(&ijson::to_value(&header)?)?;

        let prelude_bytes = Prelude::new(
            Self::FILE_VERSION,
            semver::Version::parse(crate::utils::crate_version())?,
            header_bytes.len(),
        )
        .try_to_bytes()?;

        Ok([prelude_bytes, header_bytes, body_bytes].concat())
    }
}

impl TryFrom<EngineSnapshot> for RnoteFileV1 {
    type Error = anyhow::Error;

    fn try_from(mut value: EngineSnapshot) -> Result<Self, Self::Error> {
        let extracted_strokes = std::mem::take(&mut value.stroke_components);
        let extracted_chronos = std::mem::take(&mut value.chrono_components);

        let strokechrono_chunks = chunk_serialize(extracted_strokes, extracted_chronos)?;

        Ok(Self {
            compression_method: CompressionMethod::default(),
            compression_lock: false,
            engine_snapshot_ir: EngineSnaphotIR {
                strokechrono_chunks,
                core: ijson::to_value(&value)?,
            },
        })
    }
}

impl TryFrom<RnoteFileV1> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFileV1) -> Result<Self, Self::Error> {
        let mut engine_snapshot: EngineSnapshot =
            ijson::from_value(&value.engine_snapshot_ir.core)?;

        let strokechrono_chunks = value
            .engine_snapshot_ir
            .strokechrono_chunks
            .par_iter()
            .map(ijson::from_value::<Vec<(Stroke, ChronoComponent)>>)
            .collect::<Result<Vec<Vec<(Stroke, ChronoComponent)>>, serde_json::Error>>()?;

        let capacity = strokechrono_chunks.iter().map(Vec::len).sum::<usize>();

        let mut stroke_components: HopSlotMap<StrokeKey, Arc<Stroke>> =
            HopSlotMap::with_capacity_and_key(capacity);
        let mut chrono_components: SecondaryMap<StrokeKey, Arc<ChronoComponent>> =
            SecondaryMap::with_capacity(capacity);

        strokechrono_chunks
            .into_iter()
            .flatten()
            .for_each(|(stroke, chrono)| {
                let key = stroke_components.insert(Arc::new(stroke));
                chrono_components.insert(key, Arc::new(chrono));
            });

        engine_snapshot.stroke_components = Arc::new(stroke_components);
        engine_snapshot.chrono_components = Arc::new(chrono_components);

        Ok(engine_snapshot)
    }
}
