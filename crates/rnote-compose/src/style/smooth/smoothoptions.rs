// Imports
use crate::Color;
use crate::style::PressureCurve;
use anyhow::Context;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::{
    f64,
    ops::{AddAssign, MulAssign},
};

/// Options for shapes that can be drawn in a smooth style.
#[derive(Debug, Clone, Serialize)]
pub struct SmoothOptions {
    /// Stroke width.
    pub stroke_width: f64,
    /// Stroke color. When set to None, the stroke outline is not drawn.
    pub stroke_color: Option<Color>,
    /// Fill color. When set to None, the fill is not drawn.
    pub fill_color: Option<Color>,
    /// Pressure curve.
    pub pressure_curve: PressureCurve,
    /// Line style.
    pub line_style: LineStyle,
    /// Line cap.
    pub line_cap: LineCap,
    #[serde(skip)]
    pub piet_stroke_style: piet::StrokeStyle,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        let stroke_width: f64 = 2.0;
        let line_style = LineStyle::default();
        let line_cap = LineCap::default();
        Self {
            stroke_width,
            stroke_color: Some(Color::BLACK),
            fill_color: None,
            pressure_curve: PressureCurve::default(),
            line_style,
            line_cap,
            piet_stroke_style: Self::generate_piet_stroke_style(stroke_width, line_style, line_cap),
        }
    }
}

impl SmoothOptions {
    /// The ratio between the length of a dash and the width of the stroke
    const DASH_LENGTH_TO_WIDTH_RATIO: f64 = f64::consts::E;

    fn generate_piet_stroke_style(
        stroke_width: f64,
        line_style: LineStyle,
        line_cap: LineCap,
    ) -> piet::StrokeStyle {
        let mut dash_pattern = line_style.as_unscaled_vector();
        match line_cap {
            LineCap::Straight => dash_pattern
                .iter_mut()
                .for_each(|e| e.mul_assign(stroke_width * Self::DASH_LENGTH_TO_WIDTH_RATIO)),
            LineCap::Rounded => dash_pattern.iter_mut().enumerate().for_each(|(idx, e)| {
                if !line_style.is_dotted() {
                    e.mul_assign(stroke_width * Self::DASH_LENGTH_TO_WIDTH_RATIO);
                }
                // If the stroke has a rounded linecap, a half-disk with radius equal to the stroke width is added both ends of a stroke, this increases the length of each line by the width of the stroke, and is not taken into account by DashStroke, it has to be manually accounted for
                if idx % 2 == 1 {
                    e.add_assign(2.0 * stroke_width)
                }
            }),
        };
        let mut stroke_style = piet::StrokeStyle::new();
        stroke_style.set_dash_pattern(dash_pattern);
        stroke_style.set_line_cap(line_cap.into());
        stroke_style
    }

    pub fn update_piet_stroke_style(&mut self) {
        self.piet_stroke_style =
            Self::generate_piet_stroke_style(self.stroke_width, self.line_style, self.line_cap);
    }

    /// Updates the line cap
    pub fn update_line_cap(&mut self, line_cap: LineCap) {
        // Dotted style requires a round LineCap
        if self.line_style.is_dotted() && line_cap != LineCap::Rounded {
            self.line_style = LineStyle::Solid;
        }
        self.line_cap = line_cap;
        self.update_piet_stroke_style();
    }

    /// Updates the line style
    pub fn update_line_style(&mut self, line_style: LineStyle) {
        // Dotted style requires a round LineCap
        if line_style.is_dotted() {
            self.line_cap = LineCap::Rounded;
        }
        self.line_style = line_style;
        self.update_piet_stroke_style();
    }
}

impl<'de> Deserialize<'de> for SmoothOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(default, rename = "smooth_options")]
        struct SmoothOptionsPrecursor {
            pub stroke_width: f64,
            pub stroke_color: Option<Color>,
            pub fill_color: Option<Color>,
            pub pressure_curve: PressureCurve,
            pub line_style: LineStyle,
            pub line_cap: LineCap,
        }

        impl From<SmoothOptions> for SmoothOptionsPrecursor {
            fn from(value: SmoothOptions) -> Self {
                Self {
                    stroke_width: value.stroke_width,
                    stroke_color: value.stroke_color,
                    fill_color: value.fill_color,
                    pressure_curve: value.pressure_curve,
                    line_style: value.line_style,
                    line_cap: value.line_cap,
                }
            }
        }

        impl Default for SmoothOptionsPrecursor {
            fn default() -> Self {
                SmoothOptions::default().into()
            }
        }

        let precursor = SmoothOptionsPrecursor::deserialize(deserializer)?;

        Ok(SmoothOptions {
            stroke_width: precursor.stroke_width,
            stroke_color: precursor.stroke_color,
            fill_color: precursor.fill_color,
            pressure_curve: precursor.pressure_curve,
            line_style: precursor.line_style,
            line_cap: precursor.line_cap,
            piet_stroke_style: Self::generate_piet_stroke_style(
                precursor.stroke_width,
                precursor.line_style,
                precursor.line_cap,
            ),
        })
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

impl Default for LineCap {
    fn default() -> Self {
        Self::Straight
    }
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
            Self::Dotted => vec![0.0, 0.0], // LineCap must be set to 'Rounded'
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

impl Default for LineStyle {
    fn default() -> Self {
        Self::Solid
    }
}

impl TryFrom<u32> for LineStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("LineStyle try_from::<u32>() for value {value} failed"))
    }
}
