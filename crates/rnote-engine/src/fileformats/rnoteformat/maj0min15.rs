// Imports
use crate::fileformats::rnoteformat::maj0min13::RnoteFileMaj0Min13;
use anyhow::anyhow;
use ijson::IValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFileMaj0Min15 {
    /// A snapshot of the engine.
    #[serde(rename = "engine_snapshot")]
    pub engine_snapshot: ijson::IValue,
}

impl TryFrom<RnoteFileMaj0Min13> for RnoteFileMaj0Min15 {
    type Error = anyhow::Error;

    fn try_from(mut value: RnoteFileMaj0Min13) -> Result<Self, Self::Error> {
        let engine_snapshot = value
            .engine_snapshot
            .as_object_mut()
            .ok_or(anyhow!("engine snapshot is not a JSON object."))?;

        for comp in engine_snapshot
            .get_mut("stroke_components")
            .ok_or(anyhow!(
                "engine snapshot does not contain 'stroke_components'."
            ))?
            .as_array_mut()
            .ok_or(anyhow!("stroke components is not a JSON array."))?
            .iter_mut()
        {
            let value = comp
                .as_object_mut()
                .ok_or(anyhow!("stroke component is not a JSON object."))?
                .get_mut("value")
                .ok_or(anyhow!("stroke component does not contain 'value'."))?;
            if value.is_null() {
                continue;
            }
            if let Some(shapestroke) = value
                .as_object_mut()
                .ok_or(anyhow!("value is not a JSON object."))?
                .get_mut("shapestroke")
            {
                let shape = shapestroke
                    .as_object_mut()
                    .ok_or(anyhow!("shapestroke is not a JSON object."))?
                    .get_mut("shape")
                    .ok_or(anyhow!("shapestroke does not contain 'shape'."))?;

                if let Some(rect) = shape
                    .as_object_mut()
                    .ok_or(anyhow!("shape is not a JSON object."))?
                    .get_mut("rect")
                {
                    convert_transform_to_affine_in_obj(rect)?;
                } else if let Some(ellipse) = shape
                    .as_object_mut()
                    .ok_or(anyhow!("shape is not a JSON object."))?
                    .get_mut("ellipse")
                {
                    convert_transform_to_affine_in_obj(ellipse)?;
                }
            } else if let Some(textstroke) = value
                .as_object_mut()
                .ok_or(anyhow!("value is not a JSON object."))?
                .get_mut("textstroke")
            {
                convert_transform_to_affine_in_obj(textstroke)?;
            } else if let Some(vectorimage) = value
                .as_object_mut()
                .ok_or(anyhow!("value is not a JSON object."))?
                .get_mut("vectorimage")
            {
                convert_transform_to_affine_in_obj(
                    vectorimage
                        .as_object_mut()
                        .ok_or(anyhow!("vectorimage is not a JSON object."))?
                        .get_mut("rectangle")
                        .ok_or(anyhow!("vectorimage does not contain 'rectangle'."))?,
                )?;
            } else if let Some(bitmapimage) = value
                .as_object_mut()
                .ok_or(anyhow!("value is not a JSON object."))?
                .get_mut("bitmapimage")
            {
                let bitmapimage = bitmapimage
                    .as_object_mut()
                    .ok_or(anyhow!("bitmapimage is not a JSON object."))?;

                convert_transform_to_affine_in_obj(
                    bitmapimage
                        .get_mut("image")
                        .ok_or(anyhow!("bitmapimage does not contain 'image'."))?
                        .as_object_mut()
                        .ok_or(anyhow!("image is not a JSON object."))?
                        .get_mut("rectangle")
                        .ok_or(anyhow!("image does not contain 'rectangle'."))?,
                )?;
                convert_transform_to_affine_in_obj(
                    bitmapimage
                        .get_mut("rectangle")
                        .ok_or(anyhow!("bitmapimage does not contain 'rectangle'."))?,
                )?;
            }
        }

        //dbg!(&value);

        Ok(Self {
            engine_snapshot: value.engine_snapshot,
        })
    }
}

/// Converts a "object->transform->nalgebra-affine" to "object->(glamx-)affine"
fn convert_transform_to_affine_in_obj(obj: &mut IValue) -> anyhow::Result<()> {
    let obj = obj
        .as_object_mut()
        .ok_or(anyhow!("supplied value is not a JSON object."))?;
    let transform = obj
        .remove("transform")
        .ok_or(anyhow!("rect does not contain 'transform'."))?;
    let transform = transform
        .as_object()
        .ok_or(anyhow!("transform is not a JSON object."))?;
    let affine = transform
        .get("affine")
        .ok_or(anyhow!("transform does not contain 'affine'."))?
        .as_array()
        .ok_or(anyhow!("affine not an array."))?;

    obj.insert(
        "affine",
        vec![
            #[allow(clippy::get_first)]
            affine
                .get(0)
                .cloned()
                .ok_or(anyhow!("affine does not have value at index 0"))?,
            affine
                .get(1)
                .cloned()
                .ok_or(anyhow!("affine does not have value at index 1"))?,
            affine
                .get(3)
                .cloned()
                .ok_or(anyhow!("affine does not have value at index 3"))?,
            affine
                .get(4)
                .cloned()
                .ok_or(anyhow!("affine does not have value at index 4"))?,
            affine
                .get(6)
                .cloned()
                .ok_or(anyhow!("affine does not have value at index 6"))?,
            affine
                .get(7)
                .cloned()
                .ok_or(anyhow!("affine does not have value at index 7"))?,
        ],
    );

    Ok(())
}
