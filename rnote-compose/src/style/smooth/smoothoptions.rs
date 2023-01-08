use crate::style::strokeoptions::StrokeOptions;
use crate::style::PressureCurve;
use crate::Color;

use serde::{Deserialize, Serialize};

/// Options for shapes that can be drawn smoothly
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "smooth_options")]
pub struct SmoothOptions {
    /// The stroke
    #[serde(rename = "stroke_options")]
    pub stroke_options: StrokeOptions,
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
            stroke_options: StrokeOptions::default(),
            fill_color: None,
            pressure_curve: PressureCurve::default(),
        }
    }
}
