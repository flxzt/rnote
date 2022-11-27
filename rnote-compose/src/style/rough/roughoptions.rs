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
            stroke_width: Self::STROKE_WIDTH_DEFAULT,
            fill_color: None,
            fill_style: FillStyle::Hachure,
            hachure_angle: Self::HACHURE_ANGLE_DEFAULT,
            seed: None,
        }
    }
}

impl RoughOptions {
    /// The margin for the bounds of composed rough shapes
    /// TODO: make this not a const margin, but dependent on the shape size
    pub const ROUGH_BOUNDS_MARGIN: f64 = 20.0;

    /// Default stroke width
    pub const STROKE_WIDTH_DEFAULT: f64 = 1.0;
    /// min stroke width
    pub const STROKE_WIDTH_MIN: f64 = 0.1;
    /// max stroke width
    pub const STROKE_WIDTH_MAX: f64 = 1000.0;
    /// Default hachure angle (in rad)
    pub const HACHURE_ANGLE_DEFAULT: f64 = std::f64::consts::PI / 2.0;
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
    #[serde(rename = "solid")]
    Solid,
    /// Hachure
    #[serde(rename = "hachure")]
    Hachure,
    /// Zig zag
    #[serde(rename = "zig_zag")]
    ZigZag,
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
            // These are not implemented yet in roughr, but already exist in the struct
            _ => Self::Solid,
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
        }
    }
}

impl TryFrom<u32> for FillStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("FillStyle try_from::<u32>() for value {} failed", value)
        })
    }
}
