// Imports
use super::textureddotsdistribution::TexturedDotsDistribution;
use crate::Color;
use crate::style::PressureCurve;
use serde::{Deserialize, Serialize};

/// Options for shapes that can be drawn in a textured style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "textured_options")]
pub struct TexturedOptions {
    /// An optional seed to generate reproducible shapes.
    #[serde(rename = "seed")]
    pub seed: Option<u64>,
    /// Stroke width.
    #[serde(rename = "stroke_width", with = "crate::serialize::f64_dp3")]
    pub stroke_width: f64,
    /// Stroke color. When set to None, the stroke is not drawn.
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    /// Amount of dots of the texture per 10x10 area.
    #[serde(rename = "density", with = "crate::serialize::f64_dp3")]
    pub density: f64,
    /// Texture dots distribution type.
    #[serde(rename = "distribution")]
    pub distribution: TexturedDotsDistribution,
    /// Pressure curve.
    #[serde(rename = "pressure_curve")]
    pub pressure_curve: PressureCurve,
}

impl Default for TexturedOptions {
    fn default() -> Self {
        Self {
            seed: None,
            stroke_width: 6.0,
            density: 5.0,
            stroke_color: Some(Color::BLACK),
            distribution: TexturedDotsDistribution::default(),
            pressure_curve: PressureCurve::default(),
        }
    }
}

impl TexturedOptions {
    /// Dots dadii default, without width weight.
    pub(super) const DOTS_RADII_DEFAULT: na::Vector2<f64> = na::vector![1.2, 0.3];
    /// Weight factor the stroke width has to the radii of the dots.
    pub(super) const STROKE_WIDTH_RADII_WEIGHT: f64 = 0.1;
    /// Minimum dots density.
    pub const DENSITY_MIN: f64 = 0.1;
    /// Maximum dots density.
    pub const DENSITY_MAX: f64 = 100.0;

    /// Advances the seed.
    pub fn advance_seed(&mut self) {
        self.seed = self.seed.map(crate::utils::seed_advance)
    }
}
