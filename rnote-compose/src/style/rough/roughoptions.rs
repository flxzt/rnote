use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::Color;

/// The rough options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "rough_options")]
pub struct RoughOptions {
    /// the stroke color. When set to None, no stroke outline is produced
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    /// the stroke width
    #[serde(rename = "stroke_width")]
    pub stroke_width: f64,
    /// an optional fill color. When set to None no fill is produced.
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    /// the fill style
    #[serde(rename = "fill_style")]
    pub fill_style: FillStyle,
    /// the hachure angle (in rad)
    #[serde(rename = "hachure_angle")]
    pub hachure_angle: f64,
    /// an optional seed for creating random values used in shape generation.
    /// When using the same seed the generator produces the same shape.
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
    /// The margin for the bounds of composed rough shapes
    ///
    /// TODO: make this not a fixed value, but dependent on the shape size, roughness, etc.
    pub const ROUGH_BOUNDS_MARGIN: f64 = 20.0;

    /// Advances the seed
    pub fn advance_seed(&mut self) {
        self.seed = self.seed.map(crate::utils::seed_advance)
    }
}

/// available Fill styles
#[derive(
    Debug,
    Clone,
    Copy,
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
    /// Solid
    // pre v0.5.9 the fill style was always set to `Hachure` (capitalized), even though the app rendered a solid fill.
    // For compatibility reasons we need set this alias.
    #[serde(rename = "solid", alias = "Hachure")]
    Solid,
    /// Hachure
    #[serde(rename = "hachure")]
    Hachure,
    /// Zig zag
    #[serde(rename = "zig_zag")]
    ZigZag,
    /// Zig zag line
    #[serde(rename = "zig_zag_line")]
    ZigZagLine,
    /// Crosshatch
    #[serde(rename = "crosshatch")]
    Crosshatch,
    /// Dots
    #[serde(rename = "dots")]
    Dots,
    /// Dashed
    #[serde(rename = "dashed")]
    Dashed,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self::Hachure
    }
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
