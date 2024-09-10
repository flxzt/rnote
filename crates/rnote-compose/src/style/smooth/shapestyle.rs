// Imports
use fragile::Fragile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShapeStyle {
    line_edge: LineEdge,
    line_style: LineStyle,
    #[serde(skip)]
    inner: Fragile<piet::StrokeStyle>,
}

impl ShapeStyle {
    fn update_line_edge(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum LineEdge {
    Straight,
    Rounded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum LineStyle {
    Solid,
    Dotted,
    DashedEquidistant,
}
