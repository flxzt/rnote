// Imports
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::ops::{AddAssign, MulAssign};
use tracing::info;

#[derive(Debug, Clone, Serialize)]
#[serde(into = "ShapeStylePrecursor")]
pub struct ShapeStyle {
    pub line_cap: LineCap,
    pub line_style: LineStyle,
    pub inner: piet::StrokeStyle,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shape_style")]
pub struct ShapeStylePrecursor {
    #[serde(rename = "line_edge")]
    pub line_cap: LineCap,
    #[serde(rename = "line_style")]
    pub line_style: LineStyle,
}

impl From<ShapeStyle> for ShapeStylePrecursor {
    fn from(value: ShapeStyle) -> Self {
        Self {
            line_cap: value.line_cap,
            line_style: value.line_style,
        }
    }
}

// In theory this should be safe, as all stroke elements adhere to the borrow checker's rules
// e.g. one mutable borrow at a time, will not be read after being freed
unsafe impl Send for ShapeStyle {}
unsafe impl Sync for ShapeStyle {}

impl ShapeStyle {
    pub fn update_line_cap(&mut self, line_cap: LineCap, stroke_width: f64) {
        self.inner.set_line_cap(line_cap.into());
        self.line_cap = line_cap;
        self.update_inner_strokedash(stroke_width);
    }
    pub fn update_line_style(&mut self, line_style: LineStyle, stroke_width: f64) {
        // Dotted style requires a round LineCap
        if line_style.is_dotted() {
            self.line_cap = LineCap::Rounded;
            self.inner.set_line_cap(piet::LineCap::Round);
        }
        self.line_style = line_style;
        self.update_inner_strokedash(stroke_width);
    }
    pub fn update_inner_strokedash(&mut self, stroke_width: f64) {
        let mut dash_pattern = self.line_style.as_unscaled_vector();
        match self.line_cap {
            LineCap::Straight => dash_pattern
                .iter_mut()
                .for_each(|e| e.mul_assign(stroke_width * 3.0)),
            LineCap::Rounded => dash_pattern.iter_mut().enumerate().for_each(|(idx, e)| {
                e.mul_assign(stroke_width * 3.0);
                if idx % 2 == 1 {
                    e.add_assign(stroke_width)
                }
            }),
        }
        info!("dash pattern = {:?}", dash_pattern);
        self.inner.set_dash_pattern(dash_pattern);
    }
    pub(crate) fn from_precursor(precursor: ShapeStylePrecursor, stroke_width: f64) -> Self {
        let mut shape_style = Self {
            line_cap: precursor.line_cap,
            line_style: precursor.line_style,
            inner: piet::StrokeStyle {
                line_join: piet::LineJoin::default(),
                line_cap: precursor.line_cap.into(),
                dash_pattern: piet::StrokeDash::default(),
                dash_offset: 0.0,
            },
        };
        shape_style.update_inner_strokedash(stroke_width);
        shape_style
    }
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            line_cap: LineCap::Straight,
            line_style: LineStyle::Solid,
            inner: piet::StrokeStyle::new()
                .line_join(piet::LineJoin::default())
                .line_cap(piet::LineCap::Butt)
                .dash_pattern(&[])
                .dash_offset(0.0),
        }
    }
}

impl Default for ShapeStylePrecursor {
    fn default() -> Self {
        Self {
            line_cap: LineCap::Straight,
            line_style: LineStyle::Solid,
        }
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
pub enum LineCap {
    Straight,
    Rounded,
}

impl TryFrom<u32> for LineCap {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("LineCap try_from::<u32>() for value {value} failed"))
    }
}

impl Into<piet::LineCap> for LineCap {
    fn into(self) -> piet::LineCap {
        match self {
            Self::Straight => piet::LineCap::Butt,
            Self::Rounded => piet::LineCap::Round,
        }
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
pub enum LineStyle {
    Solid,
    Dotted,
    DashedEquidistant,
}

impl LineStyle {
    /// Returns the baseline dash pattern
    fn as_unscaled_vector(&self) -> Vec<f64> {
        match self {
            Self::Solid => Vec::new(),
            // LineCap must be set to 'Rounded'
            Self::Dotted => vec![0.0, 1.0],
            Self::DashedEquidistant => vec![1.0, 1.0],
        }
    }
    /// Indicates whether or not the LineStyle is dotted
    pub fn is_dotted(&self) -> bool {
        match self {
            Self::Solid => false,
            Self::Dotted => true,
            Self::DashedEquidistant => false,
        }
    }
}

impl TryFrom<u32> for LineStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("LineStyle try_from::<u32>() for value {value} failed"))
    }
}
