// Imports
use fragile::Fragile;
use serde::{Deserialize, Serialize};
use std::ops::{AddAssign, MulAssign};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShapeStyle {
    pub line_edge: LineEdge,
    pub line_style: LineStyle,
    #[serde(skip)]
    pub inner: Fragile<piet::StrokeStyle>,
}

impl ShapeStyle {
    pub(crate) fn update_line_edge(&mut self, line_edge: LineEdge, stroke_width: f64) {
        self.line_edge = line_edge;
        self.update_inner_strokedash(stroke_width);
    }
    pub(crate) fn update_line_style(&mut self, line_style: LineStyle, stroke_width: f64) {
        self.line_style = line_style;
        self.update_inner_strokedash(stroke_width);
    }
    pub(crate) fn update_inner_strokedash(&mut self, stroke_width: f64) {
        let mut unscaled = self.line_style.as_unscaled_vector();
        match self.line_edge {
            LineEdge::Straight => unscaled
                .iter_mut()
                .for_each(|e| e.mul_assign(stroke_width * 3.0)),
            LineEdge::Rounded => unscaled.iter_mut().enumerate().for_each(|(idx, e)| {
                e.mul_assign(stroke_width * 3.0);
                if idx % 2 == 1 {
                    e.add_assign(stroke_width)
                }
            }),
        }
        let Ok(inner) = self.inner.try_get_mut() else {
            tracing::warn!("Failed to acquire inner strokestyle, invalid thread access");
            return;
        };
        inner.set_dash_pattern(unscaled)
    }
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            line_edge: LineEdge::Straight,
            line_style: LineStyle::Solid,
            inner: Fragile::new(
                piet::StrokeStyle::new()
                    .line_join(piet::LineJoin::default())
                    .line_cap(piet::LineCap::Butt)
                    .dash_pattern(&[])
                    .dash_offset(0.0),
            ),
        }
    }
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

impl LineStyle {
    fn as_unscaled_vector(&self) -> Vec<f64> {
        match self {
            Self::Solid => Vec::new(),
            Self::Dotted => vec![0.0, 1.0],
            Self::DashedEquidistant => vec![1.0, 1.0],
        }
    }
}
