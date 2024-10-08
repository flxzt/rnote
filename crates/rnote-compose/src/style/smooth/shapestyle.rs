// Imports
use anyhow::Context;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::ops::{AddAssign, MulAssign};

/// Describes the style of shapes/lines using concise presets
#[derive(Debug, Clone, Serialize)]
#[serde(into = "ShapeStylePrecursor")]
pub struct ShapeStyle {
    /// Line cap
    pub line_cap: LineCap,
    /// Line style
    pub line_style: LineStyle,
    /// Represents the dash pattern
    inner: Vec<f64>,
}

/// This struct is a subset of ShapeStyle, its goal is to keep the filesize increase to a minimum
/// as the dash pattern (inner) is calculated from the width of the stroke, the line_cap, and the line_style
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

impl ShapeStyle {
    /// The ratio between the length of a dash and the width of the stroke
    const DASH_LENGTH_TO_WIDTH_RATIO: f64 = 3.0;
    /// Updates the line cap
    pub fn update_line_cap(&mut self, line_cap: LineCap, stroke_width: f64) {
        // Dotted style requires a round LineCap
        if self.line_style.is_dotted() && line_cap != LineCap::Rounded {
            self.line_style = LineStyle::Solid;
        }
        self.line_cap = line_cap;
        self.update_inner(stroke_width);
    }
    /// Updates the line style
    pub fn update_line_style(&mut self, line_style: LineStyle, stroke_width: f64) {
        // Dotted style requires a round LineCap
        if line_style.is_dotted() {
            self.line_cap = LineCap::Rounded;
        }
        self.line_style = line_style;
        self.update_inner(stroke_width);
    }
    /// Updates the inner strokedash
    pub fn update_inner(&mut self, stroke_width: f64) {
        let mut dash_pattern = self.line_style.as_unscaled_vector();
        match self.line_cap {
            LineCap::Straight => dash_pattern
                .iter_mut()
                .for_each(|e| e.mul_assign(stroke_width * Self::DASH_LENGTH_TO_WIDTH_RATIO)),
            LineCap::Rounded => dash_pattern.iter_mut().enumerate().for_each(|(idx, e)| {
                if !self.line_style.is_dotted() {
                    e.mul_assign(stroke_width * Self::DASH_LENGTH_TO_WIDTH_RATIO);
                }
                // round edges add a half-disk with a radius equal to the stroke width to each edge of a line
                // this increases the length of each line by the width of the stroke, and is not taken into account by DashStroke
                // therefore we must manually account for it twice
                if idx % 2 == 1 {
                    e.add_assign(2.0 * stroke_width)
                }
            }),
        }
        self.inner = dash_pattern;
    }
    pub(crate) fn from_precursor(precursor: ShapeStylePrecursor, stroke_width: f64) -> Self {
        let mut shape_style = Self {
            line_cap: precursor.line_cap,
            line_style: precursor.line_style,
            inner: Vec::with_capacity(0),
        };
        shape_style.update_inner(stroke_width);
        shape_style
    }
    pub(crate) fn get(&self) -> piet::StrokeStyle {
        let mut style = piet::StrokeStyle::new().line_cap(self.line_cap.into());
        style.set_dash_pattern(self.inner.as_slice());
        style
    }
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            line_cap: LineCap::Straight,
            line_style: LineStyle::Solid,
            inner: Vec::with_capacity(0),
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

/// Line cap present at the start and end of a line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
pub enum LineCap {
    /// Straight line cap
    Straight,
    /// Rounded line cap
    Rounded,
}

impl TryFrom<u32> for LineCap {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("LineCap try_from::<u32>() for value {value} failed"))
    }
}

impl From<LineCap> for piet::LineCap {
    fn from(value: LineCap) -> Self {
        match value {
            LineCap::Straight => piet::LineCap::Butt,
            LineCap::Rounded => piet::LineCap::Round,
        }
    }
}

/// The overall style of the line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
pub enum LineStyle {
    /// Solid line style
    Solid,
    /// Dotted line style, the dots are equidistant
    Dotted,
    /// Dashed line style, the dashes have less space between them
    DashedNarrow,
    /// Dashed line style, the dashes are equidistant
    DashedEquidistant,
    /// Dashed line style, the dashes have more space between them
    DashedWide,
}

impl LineStyle {
    /// Returns the baseline dash pattern
    fn as_unscaled_vector(&self) -> Vec<f64> {
        match self {
            Self::Solid => Vec::new(),
            // LineCap must be set to 'Rounded'
            Self::Dotted => vec![0.0, 0.0],
            Self::DashedNarrow => vec![1.0, 0.618],
            Self::DashedEquidistant => vec![1.0, 1.0],
            Self::DashedWide => vec![1.0, 1.618],
        }
    }
    /// Indicates whether or not the LineStyle is dotted
    pub fn is_dotted(&self) -> bool {
        match self {
            Self::Solid => false,
            Self::Dotted => true,
            Self::DashedNarrow => false,
            Self::DashedEquidistant => false,
            Self::DashedWide => false,
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
