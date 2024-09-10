// Imports
use super::shapestyle::ShapeStyle;
use crate::style::PressureCurve;
use crate::Color;
use serde::{Deserialize, Serialize};

/// Options for shapes that can be drawn in a smooth style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "smooth_options")]
pub struct SmoothOptions {
    /// Stroke width.
    #[serde(rename = "stroke_width", with = "crate::serialize::f64_dp3")]
    pub stroke_width: f64,
    /// Stroke color. When set to None, the stroke outline is not drawn.
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    /// Fill color. When set to None, the fill is not drawn.
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    /// Pressure curve.
    #[serde(rename = "pressure_curve")]
    pub pressure_curve: PressureCurve,
    /// Shape style.
    #[serde(rename = "shape_style")]
    pub shape_style: Option<ShapeStyle>,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        Self {
            stroke_width: 2.0,
            stroke_color: Some(Color::BLACK),
            fill_color: None,
            pressure_curve: PressureCurve::default(),
            shape_style: None,
        }
    }
}
