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
    /// Roughness min
    pub const ROUGHNESS_MIN: f64 = 0.0;
    /// Roughness max
    pub const ROUGHNESS_MAX: f64 = 10.0;
    /// Roughness default
    pub const ROUGHNESS_DEFAULT: f64 = 1.0;
    /// Bowing min
    pub const BOWING_MIN: f64 = 0.0;
    /// Bowing max
    pub const BOWING_MAX: f64 = 20.0;
    /// Bowing default
    pub const BOWING_DEFAULT: f64 = 1.0;
    /// Curve stepcount min
    pub const CURVESTEPCOUNT_MIN: f64 = 3.0;
    /// Curve stepcount max
    pub const CURVESTEPCOUNT_MAX: f64 = 1000.0;
    /// Curve stepcount default
    pub const CURVESTEPCOUNT_DEFAULT: f64 = 12.0;
}

/// available Fill styles
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "fill_style")]
pub enum FillStyle {
    /// Solid
    #[serde(rename = "solid")]
    Solid,
    /// Hachure
    #[serde(rename = "hachure")]
    Hachure,
    /// Zigzag
    #[serde(rename = "zigzag")]
    Zigzag,
    /// Zigzagline
    #[serde(rename = "zigzag_line")]
    ZigzagLine,
    /// Crosshatch
    #[serde(rename = "crosshatch")]
    Crosshatch,
    /// Dots
    #[serde(rename = "dots")]
    Dots,
    /// Sunburst
    #[serde(rename = "sunburst")]
    Sunburst,
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
            roughr::core::FillStyle::ZigZag => Self::Zigzag,
            roughr::core::FillStyle::CrossHatch => Self::Crosshatch,
            roughr::core::FillStyle::Dots => Self::Dots,
            roughr::core::FillStyle::Dashed => Self::Dashed,
            roughr::core::FillStyle::ZigZagLine => Self::ZigzagLine,
        }
    }
}

impl From<FillStyle> for roughr::core::FillStyle {
    fn from(s: FillStyle) -> Self {
        match s {
            FillStyle::Solid => roughr::core::FillStyle::Solid,
            FillStyle::Hachure => roughr::core::FillStyle::Hachure,
            FillStyle::Zigzag => roughr::core::FillStyle::ZigZag,
            FillStyle::ZigzagLine => roughr::core::FillStyle::ZigZagLine,
            FillStyle::Crosshatch => roughr::core::FillStyle::CrossHatch,
            FillStyle::Dots => roughr::core::FillStyle::Dots,
            // FIXME: Not implemented yet, defaulting to Hachure
            FillStyle::Sunburst => roughr::core::FillStyle::Hachure,
            FillStyle::Dashed => roughr::core::FillStyle::Dashed,
        }
    }
}
