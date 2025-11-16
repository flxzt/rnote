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

/// Intermediate representation of the `EngineSnapshot` for save compatibility
#[derive(Debug, Clone)]
pub struct CompatibilityBridgeV1 {
    /// The `EngineSnapshot` without `stroke_components` and `chrono_components` (still there but empty) converted to an `IValue`
    pub es_gutted_ival: ijson::IValue,
    /// A vector of chunks, where each chunk is an array of `(Stroke, ChronoComponent)` values, converted to an `IValue`
    pub stroke_chrono_pair_ival_chunks: Vec<ijson::IValue>,
}

/// Information about a specific chunk, or the core.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "chunk_info")]
struct ChunkInfo {
    /// size of the serialized and compressed chunk
    pub c_size: usize,
    /// size of the serialized (and uncompressed) chunk
    pub uc_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rnote_header")]
struct RnoteHeaderV1 {
    /// method used to compress/decompress the body
    #[serde(rename = "compression_method")]
    pub compression_method: CompressionMethod,
    #[serde(rename = "core_info")]
    pub core_info: ChunkInfo,
    #[serde(rename = "chunk_info_vec")]
    pub chunk_info_vec: Vec<ChunkInfo>,
}

type EngineStrokes = Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>;
type EngineChronos = Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>;

#[allow(missing_debug_implementations)]
pub struct RnoteFileInterfaceV1;

impl RnoteFileInterfaceV1 {
    pub const FILE_VERSION: u16 = 1;

    // The generic argument `ES` dictates to what type the gutted serialized `EngineSnapshot` is deserialized into.
    //   → `ES` = `EngineSnapshot` when going straight from bytes to `EngineSnapshot`
    //   → `ES` = `ijson::IValue` when going from bytes to `CompatibilityBridgeV1`
    // The generic argument `SC` dictates to what type the stroke-chrono pair chunks are deserialized into.
    //   → `SC` = `Vec<(Stroke, ChronoComponent)>` when going straight from bytes to `EngineSnapshot`
    //   → `SC` = `ijson::IValue` when going from bytes to `CompatibilityBridgeV1`
    fn bytes_to_base<ES, SC>(
        mut cursor: BCursor,
        header_size: usize,
    ) -> anyhow::Result<(ES, Vec<SC>)>
    where
        ES: Send + DeserializeOwned,
        SC: Send + DeserializeOwned,
    {
        let header: RnoteHeaderV1 = serde_json::from_slice(cursor.try_capture(header_size)?)?;

        let engine_snapshot: ES = cursor
            .try_capture(header.core_info.c_size)
            .and_then(|c_data| {
                header
                    .compression_method
                    .decompress(c_data, header.core_info.uc_size)
            })
            .and_then(|uc_data| {
                serde_json::from_slice::<ES>(&uc_data).map_err(anyhow::Error::from)
            })?;

        // Not the most readable block of code but quite functional, we first gather
        // every serialized and compressed chunk with their respective uncompressed size,
        // to then decompress and deserialize them in parallel afterward.
        let stroke_chrono_pair_chunk_vec = header
            .chunk_info_vec
            .into_iter()
            .map(|info| {
                cursor
                    .try_capture(info.c_size)
                    .map(|slice| (slice, info.uc_size))
            })
            .collect::<Result<Vec<(&[u8], usize)>, anyhow::Error>>()?
            .into_par_iter()
            .map(|(data, uc_size)| {
                header
                    .compression_method
                    .decompress(data, uc_size)
                    .and_then(|uc_data| {
                        serde_json::from_slice::<SC>(&uc_data).map_err(anyhow::Error::from)
                    })
            })
            .collect::<Result<Vec<SC>, anyhow::Error>>()?;

        Ok((engine_snapshot, stroke_chrono_pair_chunk_vec))
    }

    fn components_to_engine_snapshot(
        mut engine_snapshot: EngineSnapshot,
        stroke_chrono_pair_chunk_vec: Vec<Vec<(Stroke, ChronoComponent)>>,
    ) -> EngineSnapshot {
        let capacity = stroke_chrono_pair_chunk_vec
            .iter()
            .map(Vec::len)
            .sum::<usize>();

        let mut stroke_components: HopSlotMap<StrokeKey, Arc<Stroke>> =
            HopSlotMap::with_capacity_and_key(capacity);
        let mut chrono_components: SecondaryMap<StrokeKey, Arc<ChronoComponent>> =
            SecondaryMap::with_capacity(capacity);

        stroke_chrono_pair_chunk_vec
            .into_iter()
            .flatten()
            .for_each(|(stroke, chrono)| {
                let key = stroke_components.insert(Arc::new(stroke));
                chrono_components.insert(key, Arc::new(chrono));
            });

        engine_snapshot.stroke_components = Arc::new(stroke_components);
        engine_snapshot.chrono_components = Arc::new(chrono_components);

        engine_snapshot
    }

    pub fn bytes_to_engine_snapshot(
        cursor: BCursor,
        header_size: usize,
    ) -> anyhow::Result<EngineSnapshot> {
        let (engine_snapshot, stroke_chrono_pair_chunk_vec) = Self::bytes_to_base::<
            EngineSnapshot,
            Vec<(Stroke, ChronoComponent)>,
        >(cursor, header_size)?;

        Ok(Self::components_to_engine_snapshot(
            engine_snapshot,
            stroke_chrono_pair_chunk_vec,
        ))
    }

    pub fn bytes_to_sc_bridge(
        cursor: BCursor,
        header_size: usize,
    ) -> anyhow::Result<CompatibilityBridgeV1> {
        Self::bytes_to_base::<ijson::IValue, ijson::IValue>(cursor, header_size).map(
            |(es_gutted_ival, stroke_chrono_pair_ival_chunks)| CompatibilityBridgeV1 {
                es_gutted_ival,
                stroke_chrono_pair_ival_chunks,
            },
        )
    }

    pub fn bridge_to_engine_snapshot(
        bridge: CompatibilityBridgeV1,
    ) -> anyhow::Result<EngineSnapshot> {
        let engine_snapshot: EngineSnapshot = ijson::from_value(&bridge.es_gutted_ival)?;

        let stroke_chrono_pair_chunk_vec = bridge
            .stroke_chrono_pair_ival_chunks
            .par_iter()
            .map(ijson::from_value::<Vec<(Stroke, ChronoComponent)>>)
            .collect::<Result<Vec<Vec<(Stroke, ChronoComponent)>>, serde_json::Error>>()?;

        Ok(Self::components_to_engine_snapshot(
            engine_snapshot,
            stroke_chrono_pair_chunk_vec,
        ))
    }

    pub fn engine_snapshot_to_bytes(
        mut engine_snapshot: EngineSnapshot,
        compression_method: CompressionMethod,
    ) -> anyhow::Result<Vec<u8>> {
        let engine_strokes: EngineStrokes = std::mem::take(&mut engine_snapshot.stroke_components);
        let engine_chronos: EngineChronos = std::mem::take(&mut engine_snapshot.chrono_components);

        let mut core_info = ChunkInfo {
            c_size: 0,
            uc_size: 0,
        };
        let core_bytes = compression_method
            .compress(
                serde_json::to_vec(&engine_snapshot)
                    .inspect(|encoded| core_info.uc_size = encoded.len())?,
            )
            .inspect(|compressed| core_info.c_size = compressed.len())?;

        let local_buffer: ThreadLocal<RefCell<Vec<u8>>> = ThreadLocal::new();
        engine_strokes
            .iter()
            .map(|(key, stroke)| (stroke, engine_chronos.get(key).unwrap()))
            .par_bridge()
            .for_each_init(
                || {
                    local_buffer.get_or(|| {
                        std::cell::RefCell::new({
                            let mut vec = Vec::with_capacity(4096);
                            vec.push(b'[');
                            vec
                        })
                    })
                },
                |&mut cell, stroke_chrono_pair: _| {
                    let mut buf = cell.borrow_mut();
                    if buf.len() > 1 {
                        buf.push(b',');
                    }
                    serde_json::to_writer(&mut *buf, &stroke_chrono_pair).unwrap()
                },
            );

        let mut chunk_vec = local_buffer
            .into_iter()
            .map(RefCell::into_inner)
            .collect_vec();

        let mut chunk_info_vec = Vec::with_capacity(chunk_vec.len());
        chunk_vec
            .par_iter_mut()
            .map(|chunk| {
                chunk.push(b']');
                let uc_size = chunk.len();
                compression_method.compress_mut(chunk).unwrap();
                let c_size = chunk.len();
                ChunkInfo { c_size, uc_size }
            })
            .collect_into_vec(&mut chunk_info_vec);

        let header = RnoteHeaderV1 {
            compression_method,
            core_info,
            chunk_info_vec,
        };
        let header_bytes = serde_json::to_vec(&header)?;

        let prelude_bytes = Prelude::new(
            Self::FILE_VERSION,
            semver::Version::parse(crate::utils::crate_version())?,
            header_bytes.len(),
        )
        .try_to_bytes()?;

        Ok([prelude_bytes, header_bytes, core_bytes, chunk_vec.concat()].concat())
    }
}

impl TryFrom<LegacyRnoteFile> for CompatibilityBridgeV1 {
    type Error = anyhow::Error;

    fn try_from(mut value: LegacyRnoteFile) -> Result<Self, Self::Error> {
        let engine_snapshot = value
            .engine_snapshot
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("engine snapshot is not a JSON object"))?;

        let engine_strokes: EngineStrokes =
            ijson::from_value(&engine_snapshot.remove("stroke_components").ok_or_else(|| {
                anyhow::anyhow!("`engine_snapshot` has no value `stroke_components`")
            })?)?;

        engine_snapshot.insert(
            "stroke_components",
            ijson::to_value::<EngineStrokes>(Default::default()).unwrap(),
        );

        let engine_chronos: EngineChronos =
            ijson::from_value(&engine_snapshot.remove("chrono_components").ok_or_else(|| {
                anyhow::anyhow!("`engine_snapshot` has no value `chrono_components`")
            })?)?;

        engine_snapshot.insert(
            "chrono_components",
            ijson::to_value::<EngineChronos>(Default::default()).unwrap(),
        );

        let strokechrono_vec = engine_strokes
            .iter()
            .map(|(key, stroke)| (stroke, engine_chronos.get(key).unwrap()))
            .collect_vec();

        // The number of chunks to split `strokechrono_vec` into
        let n = rayon::current_num_threads();
        //tracing::debug!("Splitting `strokechrono_vec` into {n} parts");

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

        let strokechrono_chunks = pre_chunks
            .into_par_iter()
            .map(ijson::to_value)
            .collect::<Result<Vec<ijson::IValue>, serde_json::Error>>()?;

        Ok(Self {
            es_gutted_ival: value.engine_snapshot,
            stroke_chrono_pair_ival_chunks: strokechrono_chunks,
        })
    }
}

// --- ✀ ---
// The code below this line could be commented out after a new file version is implemented
