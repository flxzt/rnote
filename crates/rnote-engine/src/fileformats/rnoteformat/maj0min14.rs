use std::{num::NonZero, sync::Arc};

use super::compression::CompressionMethod;
use crate::{
    fileformats::rnoteformat::{bcursor::BCursor, legacy::LegacyRnoteFile},
    store::{ChronoComponent, StrokeKey},
    strokes::Stroke,
};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use slotmap::HopSlotMap;

#[derive(Debug, Clone)]
pub struct RnoteFileMaj0Min14 {
    /// The file's head is composed of the prelude plus the header (below).
    /// Contains the necessary information to efficiently compress/decompress, serialize/deserialize the rnote file.
    pub header: RnoteHeaderMaj0Min14,
    pub core: ijson::IValue,
    pub para_vec: Vec<ijson::IValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "header")]
pub struct RnoteHeaderMaj0Min14 {
    /// method used to compress/decompress the body
    #[serde(rename = "compression_method")]
    pub compression_method: CompressionMethod,
    /// if set to true, the file can keep using non-standard methods and will not be forced back into using defaults
    #[serde(rename = "method_lock")]
    pub method_lock: bool,
    /// size of the uncompressed body
    #[serde(rename = "uc_size")]
    pub uc_size: usize,
    /// length of the core, which is the serialized engine snapshot, minus the gutted `stroke_components` and `chrono_components`
    #[serde(rename = "core_length")]
    pub core_length: usize,
    /// length of the various (stroke, chrono) vectors
    #[serde(rename = "para_length_vec")]
    pub para_length_vec: Vec<usize>,
}

impl RnoteFileMaj0Min14 {
    pub(super) fn load(mut cursor: BCursor, header_size: usize) -> anyhow::Result<Self> {
        let header = ijson::from_value::<RnoteHeaderMaj0Min14>(&serde_json::from_slice(
            cursor.try_capture(header_size)?,
        )?)?;

        let body = header
            .compression_method
            .decompress(header.uc_size, cursor.get_rest())?;

        let mut body_cursor = BCursor::new(body.as_slice());

        let core_slice = body_cursor.try_capture(header.core_length)?;
        let core = serde_json::from_slice::<ijson::IValue>(core_slice)?;

        let para_slice_vec = header
            .para_length_vec
            .iter()
            .cloned()
            .map(|length| body_cursor.try_capture(length))
            .collect::<Result<Vec<&[u8]>, anyhow::Error>>()?;
        let para_vec = para_slice_vec
            .into_par_iter()
            .map(serde_json::from_slice::<ijson::IValue>)
            .collect::<Result<Vec<ijson::IValue>, serde_json::Error>>()?;

        Ok(Self {
            header,
            core,
            para_vec,
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

        let extracted_strokes: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>> =
            ijson::from_value(&engine_snapshot.remove("stroke_components").ok_or_else(|| {
                anyhow::anyhow!("`engine_snapshot` has no value `stroke_components`")
            })?)?;

        let extracted_chronos: Arc<HopSlotMap<StrokeKey, Arc<ChronoComponent>>> =
            ijson::from_value(&engine_snapshot.remove("chrono_components").ok_or_else(|| {
                anyhow::anyhow!("`engine_snapshot` has no value `chrono_components`")
            })?)?;

        {
            engine_snapshot.insert(
                "stroke_components",
                ijson::to_value::<Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>>(Default::default())
                    .unwrap(),
            );
            engine_snapshot.insert(
                "chrono_components",
                ijson::to_value::<Arc<HopSlotMap<StrokeKey, Arc<ChronoComponent>>>>(
                    Default::default(),
                )
                .unwrap(),
            );
        }

        let strokes_with_chronos = extracted_strokes
            .iter()
            .map(|(key, stroke)| (stroke, extracted_chronos.get(key).unwrap()))
            .collect_vec();

        let nb_para = std::thread::available_parallelism()
            .map(NonZero::get)
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to get available parallelism, {e}");
                1_usize
            });

        let para_vec = {
            let n = nb_para;
            let len = strokes_with_chronos.len();

            let base = len / n;
            let rem = len % n;

            let mut res = Vec::with_capacity(n);
            let mut start = 0usize;

            for i in 0..n {
                let size = base + if i < rem { 1 } else { 0 };
                let end = start + size;
                res.push(&strokes_with_chronos[start..end]);
                start = end;
            }

            res
        }
        .into_par_iter()
        .map(ijson::to_value)
        .collect::<Result<Vec<ijson::IValue>, serde_json::Error>>()?;

        Ok(Self {
            header: RnoteHeaderMaj0Min14 {
                compression_method: CompressionMethod::default(),
                method_lock: false,
                uc_size: 0,
                core_length: 0,
                para_length_vec: vec![0; para_vec.len()],
            },
            core: ijson::to_value(&engine_snapshot)?,
            para_vec,
        })
    }
}
