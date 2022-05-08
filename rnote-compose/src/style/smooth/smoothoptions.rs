use crate::Color;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "smooth_options")]
/// Options for shapes that can be drawn smoothly (plain)
pub struct SmoothOptions {
    #[serde(rename = "stroke_width")]
    /// The stroke width
    pub stroke_width: f64,
    #[serde(rename = "stroke_color")]
    /// The stroke color
    pub stroke_color: Option<Color>,
    #[serde(rename = "fill_color")]
    /// The fill color
    pub fill_color: Option<Color>,
    #[serde(rename = "segment_constant_width")]
    /// True if segments should have a constant width ( ignoring pen pressures )
    pub segment_constant_width: bool,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        Self {
            stroke_width: Self::WIDTH_DEFAULT,
            stroke_color: Some(Color::BLACK),
            fill_color: None,
            segment_constant_width: false,
        }
    }
}

impl SmoothOptions {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 1.0;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;
}
