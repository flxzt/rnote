// Imports
use super::*;

/// A wrapper struct around `CompatV1`, makes upgrading versions relatively clear by
/// just implementing `TryFrom` between different version-tagged structs.
pub(crate) struct CompatV1For<const MAJ: u64, const MIN: u64, const PATCH: u64>(CompatV1);

impl<const MAJ: u64, const MIN: u64, const PATCH: u64> From<CompatV1>
    for CompatV1For<MAJ, MIN, PATCH>
{
    fn from(value: CompatV1) -> Self {
        Self(value)
    }
}

impl<const MAJ: u64, const MIN: u64, const PATCH: u64> From<CompatV1For<MAJ, MIN, PATCH>>
    for CompatV1
{
    fn from(value: CompatV1For<MAJ, MIN, PATCH>) -> Self {
        value.0
    }
}

// Template for upgrading between incompatible Rnote versions.
/*
impl TryFrom<CompatV1For<0, 14, 0>> for CompatV1For<0, 15, 0> {
    type Error = anyhow::Error;
    fn try_from(value: CompatV1For<0, 14, 0>) -> Result<Self, Self::Error> {
        let mut inner = value.0;

        let es_json = inner
            .engine_snapshot_gutted
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("engine snapshot is not a JSON object"))?;

        // Modify `es_json` as needed here.

        let sc_pair_json_vec = inner
            .stroke_chrono_pair_chunks
            .iter_mut()
            .map(|ival| {
                ival.as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("chunk is not a JSON object"))
            })
            .collect::<Result<Vec<&mut ijson::IObject>, anyhow::Error>>()?;

        sc_pair_json_vec.into_par_iter().for_each(|chunk| {
            // Modify `chunk` as needed here.
        });

        Ok(Self(inner))
    }
}
*/
