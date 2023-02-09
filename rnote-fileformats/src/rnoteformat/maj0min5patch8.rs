use serde::{Deserialize, Serialize};

use super::maj0min5patch9::RnoteFileMaj0Min5Patch9;

impl TryFrom<RnoteFileMaj0Min5Patch8> for RnoteFileMaj0Min5Patch9 {
    type Error = anyhow::Error;

    fn try_from(mut file: RnoteFileMaj0Min5Patch8) -> Result<RnoteFileMaj0Min5Patch9, Self::Error> {
        for value in file.store_snapshot["stroke_components"]
            .as_array_mut()
            .ok_or_else(|| anyhow::anyhow!("failure"))?
        {
            let stroke = value
                .get_mut("value")
                .ok_or_else(|| anyhow::anyhow!("failure"))?;

            if let Some(brushstroke) = stroke.get_mut("brushstroke") {
                let brushstroke = brushstroke
                    .as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("failure"))?;

                let path = serde_json::from_value::<PenPathMaj0Min5Patch8>(
                    brushstroke
                        .remove("path")
                        .ok_or_else(|| anyhow::anyhow!("failure"))?,
                )?;

                let mut path_upgraded = serde_json::Map::new();

                let mut seg_iter = path.0.into_iter().peekable();
                if let Some(start) = seg_iter.peek() {
                    let start = match start {
                        SegmentMaj0Min5Patch8::Dot { element } => element,
                        SegmentMaj0Min5Patch8::Line { start, .. } => start,
                        SegmentMaj0Min5Patch8::QuadBez { start, .. } => start,
                        SegmentMaj0Min5Patch8::CubBez { start, .. } => start,
                    };

                    path_upgraded.insert(String::from("start"), serde_json::to_value(start)?);

                    let mut segments_upgraded = Vec::new();
                    for seg in seg_iter {
                        let mut segment_upgraded = serde_json::Map::new();

                        match seg {
                            SegmentMaj0Min5Patch8::Dot { element } => {
                                let mut lineto = serde_json::Map::new();
                                lineto.insert(String::from("end"), serde_json::to_value(element)?);

                                segment_upgraded.insert(String::from("lineto"), lineto.into());
                            }
                            SegmentMaj0Min5Patch8::Line { start, end } => {
                                let mut lineto = serde_json::Map::new();
                                lineto.insert(String::from("start"), serde_json::to_value(start)?);
                                lineto.insert(String::from("end"), serde_json::to_value(end)?);

                                segment_upgraded.insert(String::from("lineto"), lineto.into());
                            }
                            SegmentMaj0Min5Patch8::QuadBez { start, cp, end } => {
                                let mut quadbezto = serde_json::Map::new();
                                quadbezto
                                    .insert(String::from("start"), serde_json::to_value(start)?);
                                quadbezto.insert(String::from("cp"), serde_json::to_value(cp)?);
                                quadbezto.insert(String::from("end"), serde_json::to_value(end)?);

                                segment_upgraded
                                    .insert(String::from("quadbezto"), quadbezto.into());
                            }
                            SegmentMaj0Min5Patch8::CubBez {
                                start,
                                cp1,
                                cp2,
                                end,
                            } => {
                                let mut cubbezto = serde_json::Map::new();
                                cubbezto
                                    .insert(String::from("start"), serde_json::to_value(start)?);
                                cubbezto.insert(String::from("cp1"), serde_json::to_value(cp1)?);
                                cubbezto.insert(String::from("cp2"), serde_json::to_value(cp2)?);
                                cubbezto.insert(String::from("end"), serde_json::to_value(end)?);

                                segment_upgraded.insert(String::from("cubbezto"), cubbezto.into());
                            }
                        };

                        segments_upgraded.push(segment_upgraded.into());
                    }

                    path_upgraded.insert(
                        String::from("segments"),
                        serde_json::Value::Array(segments_upgraded),
                    );
                }

                brushstroke.insert(String::from("path"), path_upgraded.into());
            }
        }
        Ok(Self {
            store_snapshot: file.store_snapshot,
            document: file.document,
        })
    }
}

/// Rnote file in version: maj 0 min 5 patch 8
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnoteFileMaj0Min5Patch8 {
    /// the document
    #[serde(rename = "document", alias = "sheet")]
    pub document: serde_json::Value,
    /// A snapshot of the store
    #[serde(rename = "store_snapshot")]
    pub store_snapshot: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "pen_path")]
struct PenPathMaj0Min5Patch8(Vec<SegmentMaj0Min5Patch8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "segment")]
enum SegmentMaj0Min5Patch8 {
    #[serde(rename = "dot")]
    Dot {
        #[serde(rename = "element")]
        element: ElementMaj0Min5Patch8,
    },
    #[serde(rename = "line")]
    Line {
        #[serde(rename = "start")]
        start: ElementMaj0Min5Patch8,
        #[serde(rename = "end")]
        end: ElementMaj0Min5Patch8,
    },
    #[serde(rename = "quadbez")]
    QuadBez {
        #[serde(rename = "start")]
        start: ElementMaj0Min5Patch8,
        #[serde(rename = "cp")]
        cp: na::Vector2<f64>,
        #[serde(rename = "end")]
        end: ElementMaj0Min5Patch8,
    },
    #[serde(rename = "cubbez")]
    CubBez {
        #[serde(rename = "start")]
        start: ElementMaj0Min5Patch8,
        #[serde(rename = "cp1")]
        cp1: na::Vector2<f64>,
        #[serde(rename = "cp2")]
        cp2: na::Vector2<f64>,
        #[serde(rename = "end")]
        end: ElementMaj0Min5Patch8,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "element")]
struct ElementMaj0Min5Patch8 {
    #[serde(rename = "pos")]
    pub pos: na::Vector2<f64>,
    #[serde(rename = "pressure")]
    pub pressure: f64,
}
