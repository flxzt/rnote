//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod bcursor;
pub(crate) mod compression;
pub(crate) mod legacy;
pub(crate) mod maj0min14;
pub(crate) mod prelude;

// Imports
use crate::{
    engine::EngineSnapshot,
    fileformats::{
        FileFormatLoader, FileFormatSaver,
        rnoteformat::{bcursor::BCursor, compression::CompressionMethod, prelude::Prelude},
    },
    store::{ChronoComponent, StrokeKey},
    strokes::Stroke,
};
use rayon::prelude::*;
use slotmap::{HopSlotMap, SecondaryMap};
use std::sync::Arc;

pub type RnoteFile = maj0min14::RnoteFileMaj0Min14;

impl FileFormatSaver for RnoteFile {
    #[allow(unused_variables)]
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let (body, chunk_info_vec): (Vec<Vec<u8>>, Vec<maj0min14::ChunkInfo>) = self
            .engine_snapshot_ir
            .strokechrono_chunks
            .par_iter()
            .chain(rayon::iter::once(&self.engine_snapshot_ir.core))
            .map(|ival| {
                let uc_data = serde_json::to_vec(ival)?;
                let uc_size = uc_data.len();
                let c_data = self.compression_method.compress(uc_data)?;
                let c_size = c_data.len();
                Ok((c_data, maj0min14::ChunkInfo { c_size, uc_size }))
            })
            .collect::<Result<Vec<(Vec<u8>, maj0min14::ChunkInfo)>, anyhow::Error>>()?
            .into_iter()
            .unzip();

        let body_bytes = body.concat();

        let header = maj0min14::RnoteHeaderMaj0Min14 {
            compression_method: self.compression_method,
            compression_lock: self.compression_lock,
            chunk_info_vec,
        };
        let header_bytes = serde_json::to_vec(&ijson::to_value(&header)?)?;

        let prelude_bytes = Prelude::new(
            semver::Version::parse(crate::utils::crate_version())?,
            header_bytes.len(),
        )
        .try_to_bytes()?;

        Ok([prelude_bytes, header_bytes, body_bytes].concat())
    }
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut cursor = BCursor::new(bytes);

        let prelude = if cursor.try_seek(2)? != [0x1f, 0x8b] {
            Prelude::try_from_bytes(&mut cursor)?
        } else {
            // We create a "phony" prelude if the file is found to be entirely compressed with gzip (meaning it's a legacy Rnote file)
            Prelude::new(semver::Version::new(0, 13, 0), 0)
        };

        if semver::VersionReq::parse(">=0.14.0")
            .unwrap()
            .matches(&prelude.version)
        {
            maj0min14::RnoteFileMaj0Min14::load(cursor, prelude.header_size)
        } else {
            legacy::LegacyRnoteFile::load_from_bytes(bytes)?.try_into()
        }
    }
}

impl TryFrom<EngineSnapshot> for RnoteFile {
    type Error = anyhow::Error;

    fn try_from(mut value: EngineSnapshot) -> Result<Self, Self::Error> {
        let extracted_strokes = std::mem::take(&mut value.stroke_components);
        let extracted_chronos = std::mem::take(&mut value.chrono_components);

        let strokechrono_chunks = maj0min14::chunk_serialize(extracted_strokes, extracted_chronos)?;

        Ok(Self {
            compression_method: CompressionMethod::default(),
            compression_lock: false,
            engine_snapshot_ir: maj0min14::EngineSnaphotIR {
                strokechrono_chunks,
                core: ijson::to_value(&value)?,
            },
        })
    }
}

impl TryFrom<RnoteFile> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFile) -> Result<Self, Self::Error> {
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
