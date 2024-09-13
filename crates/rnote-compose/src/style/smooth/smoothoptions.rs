// Imports
use super::shapestyle::{ShapeStyle, ShapeStylePrecursor};
use crate::style::PressureCurve;
use crate::Color;
use serde::{Deserialize, Serialize};

/// Options for shapes that can be drawn in a smooth style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SmoothOptionsPrecursor", into = "SmoothOptionsPrecursor")]
pub struct SmoothOptions {
    /// Stroke width.
    pub stroke_width: f64,
    /// Stroke color. When set to None, the stroke outline is not drawn.
    pub stroke_color: Option<Color>,
    /// Fill color. When set to None, the fill is not drawn.
    pub fill_color: Option<Color>,
    /// Pressure curve.
    pub pressure_curve: PressureCurve,
    /// Shape style.
    pub shape_style: ShapeStyle,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "smooth_options")]
pub struct SmoothOptionsPrecursor {
    #[serde(rename = "stroke_width", with = "crate::serialize::f64_dp3")]
    pub stroke_width: f64,
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    #[serde(rename = "pressure_curve")]
    pub pressure_curve: PressureCurve,
    #[serde(rename = "shape_style")]
    pub shape_style: ShapeStylePrecursor,
}

impl From<SmoothOptions> for SmoothOptionsPrecursor {
    fn from(value: SmoothOptions) -> Self {
        Self {
            stroke_width: value.stroke_width,
            stroke_color: value.stroke_color,
            fill_color: value.fill_color,
            pressure_curve: value.pressure_curve,
            shape_style: value.shape_style.into(),
        }
    }
}

impl From<SmoothOptionsPrecursor> for SmoothOptions {
    fn from(value: SmoothOptionsPrecursor) -> Self {
        Self {
            stroke_width: value.stroke_width,
            stroke_color: value.stroke_color,
            fill_color: value.fill_color,
            pressure_curve: value.pressure_curve,
            shape_style: ShapeStyle::from_precursor(value.shape_style, value.stroke_width),
        }
    }
}

impl Default for SmoothOptions {
    fn default() -> Self {
        Self {
            stroke_width: 2.0,
            stroke_color: Some(Color::BLACK),
            fill_color: None,
            pressure_curve: PressureCurve::default(),
            shape_style: ShapeStyle::default(),
        }
    }
}

impl Default for SmoothOptionsPrecursor {
    fn default() -> Self {
        Self {
            stroke_width: 2.0,
            stroke_color: Some(Color::BLACK),
            fill_color: None,
            pressure_curve: PressureCurve::default(),
            shape_style: ShapeStylePrecursor::default(),
        }
    }
}
