// Imports
use super::maj0min5patch8::{
    PenPathMaj0Min5Patch8, RnoteFileMaj0Min5Patch8, SegmentMaj0Min5Patch8,
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

/// Rnote file in version: maj 0 min 5 patch 9.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RnoteFileMaj0Min5Patch9 {
    /// The document.
    #[serde(rename = "document", alias = "sheet")]
    pub(crate) document: ijson::IValue,
    /// The snapshot of the store.
    #[serde(rename = "store_snapshot")]
    pub(crate) store_snapshot: ijson::IValue,
}

impl TryFrom<RnoteFileMaj0Min5Patch8> for RnoteFileMaj0Min5Patch9 {
    type Error = anyhow::Error;

    fn try_from(mut file: RnoteFileMaj0Min5Patch8) -> Result<RnoteFileMaj0Min5Patch9, Self::Error> {
        let stroke_components = file
            .store_snapshot
            .get_mut("stroke_components")
            .ok_or_else(|| anyhow!("no value `stroke_components` in `store_snapshot`"))?
            .as_array_mut()
            .ok_or_else(|| anyhow!("value `stroke_components` is not a JSON array."))?;

        for value in stroke_components {
            let stroke = value
                .as_object_mut()
                .ok_or_else(|| anyhow!("value in `stroke_components` array is not a JSON Object."))?
                .get_mut("value")
                .ok_or_else(|| {
                    anyhow!("no value `value` in JSON object of `stroke_components` array.")
                })?;

            if stroke.is_null() {
                continue;
            }

            if let Some(brushstroke) = stroke
                .as_object_mut()
                .ok_or_else(|| anyhow!("stroke value is not a JSON Object."))?
                .get_mut("brushstroke")
            {
                let brushstroke = brushstroke
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("brushstroke is not a JSON object."))?;
                let path = ijson::from_value::<PenPathMaj0Min5Patch8>(
                    &brushstroke
                        .remove("path")
                        .ok_or_else(|| anyhow!("brushstroke has no value `path`."))?,
                )?;
                let mut path_upgraded = ijson::IObject::new();
                let mut seg_iter = path.inner().into_iter().peekable();

                if let Some(start) = seg_iter.peek() {
                    let start = match start {
                        SegmentMaj0Min5Patch8::Dot { element } => element,
                        SegmentMaj0Min5Patch8::Line { start, .. } => start,
                        SegmentMaj0Min5Patch8::QuadBez { start, .. } => start,
                        SegmentMaj0Min5Patch8::CubBez { start, .. } => start,
                    };
                    path_upgraded.insert(String::from("start"), ijson::to_value(start)?);
                    let mut segments_upgraded = ijson::IArray::new();

                    for seg in seg_iter {
                        let mut segment_upgraded = ijson::IObject::new();

                        match seg {
                            SegmentMaj0Min5Patch8::Dot { element } => {
                                let mut lineto = ijson::IObject::new();
                                lineto.insert(String::from("end"), ijson::to_value(element)?);
                                segment_upgraded.insert(String::from("lineto"), lineto);
                            }
                            SegmentMaj0Min5Patch8::Line { start, end } => {
                                let mut lineto = ijson::IObject::new();
                                lineto.insert(String::from("start"), ijson::to_value(start)?);
                                lineto.insert(String::from("end"), ijson::to_value(end)?);
                                segment_upgraded.insert(String::from("lineto"), lineto);
                            }
                            SegmentMaj0Min5Patch8::QuadBez { start, cp, end } => {
                                let mut quadbezto = ijson::IObject::new();
                                quadbezto.insert(String::from("start"), ijson::to_value(start)?);
                                quadbezto.insert(String::from("cp"), ijson::to_value(cp)?);
                                quadbezto.insert(String::from("end"), ijson::to_value(end)?);
                                segment_upgraded.insert(String::from("quadbezto"), quadbezto);
                            }
                            SegmentMaj0Min5Patch8::CubBez {
                                start,
                                cp1,
                                cp2,
                                end,
                            } => {
                                let mut cubbezto = ijson::IObject::new();
                                cubbezto.insert(String::from("start"), ijson::to_value(start)?);
                                cubbezto.insert(String::from("cp1"), ijson::to_value(cp1)?);
                                cubbezto.insert(String::from("cp2"), ijson::to_value(cp2)?);
                                cubbezto.insert(String::from("end"), ijson::to_value(end)?);
                                segment_upgraded.insert(String::from("cubbezto"), cubbezto);
                            }
                        };
                        segments_upgraded.push(segment_upgraded);
                    }
                    path_upgraded.insert(String::from("segments"), segments_upgraded);
                }
                brushstroke.insert(String::from("path"), path_upgraded);
            }
        }
        Ok(Self {
            store_snapshot: file.store_snapshot,
            document: file.document,
        })
    }
}
