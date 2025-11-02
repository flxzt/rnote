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
        rnoteformat::{
            bcursor::BCursor, compression::CompressionMethod, maj0min14::RnoteHeaderMaj0Min14,
            prelude::Prelude,
        },
    },
    store::{ChronoComponent, StrokeKey},
    strokes::Stroke,
};
use itertools::Itertools;
use rayon::prelude::*;
use slotmap::{HopSlotMap, SecondaryMap};
use std::{num::NonZero, sync::Arc};

pub type RnoteFile = maj0min14::RnoteFileMaj0Min14;

impl FileFormatSaver for RnoteFile {
    #[allow(unused_variables)]
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let core_data = serde_json::to_vec(&self.core)?;
        let core_length = core_data.len();

        let mut para_data_vec = self
            .para_vec
            .par_iter()
            .map(serde_json::to_vec)
            .collect::<Result<Vec<Vec<u8>>, serde_json::Error>>()?;
        let para_length_vec = para_data_vec.iter().map(Vec::len).collect_vec();

        let body = {
            para_data_vec.insert(0, core_data);
            self.header
                .compression_method
                .compress(para_data_vec.concat())?
        };

        let header = {
            let mut file_header = self.header.clone();
            file_header.uc_size = core_length + para_length_vec.iter().sum::<usize>();
            file_header.core_length = core_length;
            file_header.para_length_vec = para_length_vec;
            serde_json::to_vec(&ijson::to_value(&file_header)?)?
        };

        let prelude = Prelude::new(
            semver::Version::parse(crate::utils::crate_version())?,
            header.len(),
        )
        .try_to_bytes()?;

        Ok([prelude, header, body].concat())
    }
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // handle for legacy files
        if bytes
            .get(..2)
            .ok_or_else(|| anyhow::anyhow!("Invalid file, too small"))?
            == [0x1f, 0x8b]
        {
            let legacy = legacy::LegacyRnoteFile::load_from_bytes(bytes)?;
            return legacy.try_into();
        }

        let mut cursor = BCursor::new(bytes);

        let prelude = Prelude::try_from_bytes(&mut cursor)?;

        if semver::VersionReq::parse(">=0.14.0")
            .unwrap()
            .matches(&prelude.version)
        {
            maj0min14::RnoteFileMaj0Min14::load(cursor, prelude.header_size)
        } else {
            anyhow::bail!("Unknown version: '{}'", prelude.version);
        }
    }
}

impl TryFrom<EngineSnapshot> for RnoteFile {
    type Error = anyhow::Error;

    fn try_from(mut value: EngineSnapshot) -> Result<Self, Self::Error> {
        let extracted_strokes = std::mem::take(&mut value.stroke_components);
        let extracted_chronos = std::mem::take(&mut value.chrono_components);

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
            core: ijson::to_value(&value)?,
            para_vec,
        })
    }
}

impl TryFrom<RnoteFile> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFile) -> Result<Self, Self::Error> {
        let mut engine_snapshot: EngineSnapshot = ijson::from_value(&value.core)?;

        let strokes_with_chronos_multi_vec = value
            .para_vec
            .par_iter()
            .map(ijson::from_value::<Vec<(Stroke, ChronoComponent)>>)
            .collect::<Result<Vec<Vec<(Stroke, ChronoComponent)>>, serde_json::Error>>()?;

        let capacity = strokes_with_chronos_multi_vec
            .iter()
            .map(Vec::len)
            .sum::<usize>();

        let mut stroke_components: HopSlotMap<StrokeKey, Arc<Stroke>> =
            HopSlotMap::with_capacity_and_key(capacity);
        let mut chrono_components: SecondaryMap<StrokeKey, Arc<ChronoComponent>> =
            SecondaryMap::with_capacity(capacity);

        strokes_with_chronos_multi_vec
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
