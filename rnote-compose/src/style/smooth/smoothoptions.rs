use crate::Color;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "smooth_options")]
pub struct SmoothOptions {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    #[serde(rename = "segment_constant_width")]
    pub segment_constant_width: bool,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            stroke_color: Some(Self::COLOR_DEFAULT),
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
    /// The default color
    pub const COLOR_DEFAULT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}
