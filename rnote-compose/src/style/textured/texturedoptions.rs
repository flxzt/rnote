use serde::{Deserialize, Serialize};

use crate::style::PressureCurve;
use crate::Color;

use super::textureddotsdistribution::TexturedDotsDistribution;

/// The options for a textured shape

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "textured_options")]
pub struct TexturedOptions {
    /// An optional seed to generate reproducible strokes
    #[serde(rename = "seed")]
    pub seed: Option<u64>,
    /// The width
    #[serde(rename = "stroke_width")]
    pub stroke_width: f64,
    /// The color of the stroke
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    /// Amount dots per 10x10 area
    #[serde(rename = "density")]
    pub density: f64,
    /// the distribution type
    #[serde(rename = "distribution")]
    pub distribution: TexturedDotsDistribution,
    /// Pressure curve
    #[serde(rename = "pressure_curve")]
    pub pressure_curve: PressureCurve,
}

impl Default for TexturedOptions {
    fn default() -> Self {
        Self {
            seed: None,
            stroke_width: Self::WIDTH_DEFAULT,
            density: Self::DENSITY_DEFAULT,
            stroke_color: Some(Color::BLACK),
            distribution: TexturedDotsDistribution::default(),
            pressure_curve: PressureCurve::default(),
        }
    }
}

impl TexturedOptions {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 2.0;
    /// Density default
    pub const DENSITY_DEFAULT: f64 = 5.0;
    /// dots dadii default (without width weight)
    pub const RADII_DEFAULT: na::Vector2<f64> = na::vector![1.2, 0.3];
    /// The weight factor the stroke width has to the radii of the dots
    pub const STROKE_WIDTH_RADII_WEIGHT: f64 = 0.1;
}
