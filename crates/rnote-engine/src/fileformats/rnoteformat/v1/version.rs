// Imports
use super::*;

/// A wrapper struct around `CompatV1`, makes upgrading versions relatively clear by
/// just implementing `TryFrom` between different version-tagged structs.
#[derive(Debug, Clone)]
#[repr(transparent)]
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
pub type Maj0Min20 = CompatV1For<0, 20, 0>;
pub type Maj0Min21 = CompatV1For<0, 21, 0>;

impl TryFrom<Maj0Min20> for Maj0Min21 {
    type Error = anyhow::Error;

    fn try_from(value: Maj0Min20) -> Result<Self, Self::Error> {
        let mut inner = value.0;

        // We can do the following, if we need to modify something in the gutted engine snapshot.
        let es_json = inner
            .engine_snapshot_gutted
            .as_object_mut()
            .context("engine snapshot is not a JSON objct")?;

        // Modify `es_json` as needed here. The following is a simple example
        println!("The gutted engine snapshot contains the following keys:");
        es_json.keys().for_each(|key| {
            println!(
                "• key name: '{key}' -> key value type: '{:?}'",
                es_json[key].type_()
            )
        });
        let document = es_json
            .get_mut("document")
            .context("`engine_snapshot` has no value `document`")?
            .as_object_mut()
            .context("`document` is not a JSON object")?;

        let document_width = document
            .get_mut("width")
            .context("`document` has no value `width`")?
            .as_number_mut()
            .context("`width` is not a JSON number")?;

        println!("old document width: {:?}", document_width);
        *document_width = (document_width.to_f64_lossy() / 2.0).try_into().unwrap();
        println!("new document width: {:?}\n", document_width);

        // Now, if we need to modify strokes or chrono pairs
        inner
            .stroke_chrono_pair_chunks
            .par_iter_mut()
            .try_for_each(|ival| {
                let iarr = ival.as_array_mut().context("chunk is not a JSON array")?;

                iarr.iter_mut().for_each(|sc_pair| {
                    // Using unwrap to save space, you might not want to do that though
                    let sc_pair = sc_pair.as_array_mut().unwrap();
                    let ([stroke], [chrono]) = sc_pair.split_at_mut(1) else {
                        unreachable!()
                    };
                    let stroke = stroke.as_object_mut().unwrap();
                    let chrono = chrono.as_object_mut().unwrap();

                    // Modify the stroke and chrono JSON objects as you wish here
                });

                anyhow::Ok(())
            })?;

        Ok(Self(inner))
    }
}
*/
