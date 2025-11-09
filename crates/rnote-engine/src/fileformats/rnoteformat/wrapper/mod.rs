// Imports
use crate::fileformats::rnoteformat::v1::RnoteFileV1;

/// A wrapper around `RnoteFileV1` for Rnote version context
pub struct RnoteFileWrapperMaj0Min14(pub RnoteFileV1);

// Below you will find a template on how to create a new wrapper if something in `EngineSnapshot` changes
// (but not enough to require a new Rnote file version)

/*
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub struct RnoteFileWrapperMaj0Min15(RnoteFileV1);

impl TryFrom<RnoteFileWrapperMaj0Min14> for RnoteFileWrapperMaj0Min15 {
    type Error = anyhow::Error;
    fn try_from(value: RnoteFileWrapperMaj0Min14) -> Result<Self, Self::Error> {
        let mut inner = value.0;

        let es_json = inner
            .engine_snapshot_ir
            .core
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("engine snapshot is not a JSON object"))?;

        // Modify `es_json` as needed here.

        let sc_json_vec = inner
            .engine_snapshot_ir
            .strokechrono_chunks
            .iter_mut()
            .map(|ival| {
                ival.as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("chunk is not a JSON object"))
            })
            .collect::<Result<Vec<&mut ijson::IObject>, anyhow::Error>>()?;

        sc_json_vec.into_par_iter().for_each(|chunk| {
            // Modify `chunk` as needed here.
        });

        Ok(Self(inner))
    }
}
*/
