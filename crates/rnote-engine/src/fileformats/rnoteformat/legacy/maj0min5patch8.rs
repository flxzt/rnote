// Imports
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Rnote file in version: maj 0 min 5 patch 8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RnoteFileMaj0Min5Patch8 {
    /// The document.
    #[serde(rename = "document", alias = "sheet")]
    pub(crate) document: ijson::IValue,
    /// The snapshot of the store.
    #[serde(rename = "store_snapshot")]
    pub(crate) store_snapshot: ijson::IValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "pen_path")]
pub(crate) struct PenPathMaj0Min5Patch8(Vec<SegmentMaj0Min5Patch8>);

impl Deref for PenPathMaj0Min5Patch8 {
    type Target = Vec<SegmentMaj0Min5Patch8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PenPathMaj0Min5Patch8 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PenPathMaj0Min5Patch8 {
    pub(crate) fn inner(self) -> Vec<SegmentMaj0Min5Patch8> {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "segment")]
pub(crate) enum SegmentMaj0Min5Patch8 {
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
pub(crate) struct ElementMaj0Min5Patch8 {
    #[serde(rename = "pos")]
    pub(crate) pos: na::Vector2<f64>,
    #[serde(rename = "pressure")]
    pub(crate) pressure: f64,
}
