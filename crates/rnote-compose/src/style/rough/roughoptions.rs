// Imports
use crate::Color;
use anyhow::Context;
use serde::{Deserialize, Serialize};

/// Options for shapes that can be drawn in a rough style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "rough_options")]
pub struct RoughOptions {
    /// Stroke color. When set to None, the stroke outline is not drawn.
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    /// Stroke width.
    #[serde(rename = "stroke_width", with = "crate::serialize::f64_dp3")]
    pub stroke_width: f64,
    /// Fill color. When set to None the fill is not drawn.
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    /// Fill style.
    #[serde(rename = "fill_style")]
    pub fill_style: FillStyle,
    /// Hachure angle (in radians).
    #[serde(rename = "hachure_angle", with = "crate::serialize::f64_dp3")]
    pub hachure_angle: f64,
    /// An optional seed to generate reproducible shapes.
    #[serde(rename = "seed")]
    pub seed: Option<u64>,
}

impl Default for RoughOptions {
    fn default() -> Self {
        Self {
            stroke_color: Some(Color::BLACK),
            stroke_width: 2.4,
            fill_color: None,
            fill_style: FillStyle::Hachure,
            // Default hachure angle (in rad). is -41 degrees
            hachure_angle: -0.715585,
            seed: None,
        }
    }
}

impl RoughOptions {
    /// The margin for the bounds of composed rough shapes.
    // TODO: make this not a fixed value, but dependent on the shape size, roughness, etc.
    pub const ROUGH_BOUNDS_MARGIN: f64 = 20.0;

    /// Advance the seed, if it is set to `Some()`.
    pub fn advance_seed(&mut self) {
        self.seed = self.seed.map(crate::utils::seed_advance)
    }
}

/// Available fill styles.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "fill_style")]
pub enum FillStyle {
    /// Solid.
    // pre v0.5.9 the fill style was always set to `Hachure` (capitalized), even though the app rendered a solid fill.
    #[serde(rename = "solid", alias = "Hachure")]
    Solid,
    /// Hachure.
    #[default]
    #[serde(rename = "hachure")]
    Hachure,
    /// Zig zag.
    #[serde(rename = "zig_zag")]
    ZigZag,
    /// Zig zag line.
    #[serde(rename = "zig_zag_line")]
    ZigZagLine,
    /// Crosshatch.
    #[serde(rename = "crosshatch")]
    Crosshatch,
    /// Dots.
    #[serde(rename = "dots")]
    Dots,
    /// Dashed.
    #[serde(rename = "dashed")]
    Dashed,
}

impl From<roughr::core::FillStyle> for FillStyle {
    fn from(s: roughr::core::FillStyle) -> Self {
        match s {
            roughr::core::FillStyle::Solid => Self::Solid,
            roughr::core::FillStyle::Hachure => Self::Hachure,
            roughr::core::FillStyle::ZigZag => Self::ZigZag,
            roughr::core::FillStyle::CrossHatch => Self::Crosshatch,
            roughr::core::FillStyle::Dots => Self::Dots,
            roughr::core::FillStyle::Dashed => Self::Dashed,
            roughr::core::FillStyle::ZigZagLine => Self::ZigZag,
        }
    }
}

impl From<FillStyle> for roughr::core::FillStyle {
    fn from(s: FillStyle) -> Self {
        match s {
            FillStyle::Solid => roughr::core::FillStyle::Solid,
            FillStyle::Hachure => roughr::core::FillStyle::Hachure,
            FillStyle::ZigZag => roughr::core::FillStyle::ZigZag,
            FillStyle::Crosshatch => roughr::core::FillStyle::CrossHatch,
            FillStyle::Dots => roughr::core::FillStyle::Dots,
            FillStyle::Dashed => roughr::core::FillStyle::Dashed,
            FillStyle::ZigZagLine => roughr::core::FillStyle::ZigZagLine,
        }
    }
}

impl TryFrom<u32> for FillStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("FillStyle try_from::<u32>() for value {value} failed"))
    }
}
