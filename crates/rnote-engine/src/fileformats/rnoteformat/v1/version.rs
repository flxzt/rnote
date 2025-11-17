// Imports
use super::*;

pub(crate) struct CompatBridgeV1Ver<const MAJ: u64, const MIN: u64, const PATCH: u64>(
    CompatBridgeV1,
);

impl<const MAJ: u64, const MIN: u64, const PATCH: u64> From<CompatBridgeV1>
    for CompatBridgeV1Ver<MAJ, MIN, PATCH>
{
    fn from(value: CompatBridgeV1) -> Self {
        Self(value)
    }
}

impl<const MAJ: u64, const MIN: u64, const PATCH: u64> From<CompatBridgeV1Ver<MAJ, MIN, PATCH>>
    for CompatBridgeV1
{
    fn from(value: CompatBridgeV1Ver<MAJ, MIN, PATCH>) -> Self {
        value.0
    }
}

// Example on upgrading versions
/*
impl TryFrom<CompatBridgeV1Ver<0, 14, 0>> for CompatBridgeV1Ver<0, 15, 0> {
    type Error = anyhow::Error;
    fn try_from(value: CompatBridgeV1Ver<0, 14, 0>) -> Result<Self, Self::Error> {
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
