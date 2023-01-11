use crate::style::PressureCurve;
use crate::Color;

use serde::{Deserialize, Serialize};

/// Options for shapes that can be drawn smoothly
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "smooth_options")]
pub struct SmoothOptions {
    /// The stroke width
    #[serde(rename = "stroke_width")]
    pub stroke_width: f64,
    /// The stroke color
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    /// The fill color
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    /// Pressure curve
    #[serde(rename = "pressure_curve")]
    pub pressure_curve: PressureCurve,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        Self {
            stroke_width: 2.0,
            stroke_color: Some(Color::BLACK),
            fill_color: None,
            pressure_curve: PressureCurve::default(),
        }
    }
}
