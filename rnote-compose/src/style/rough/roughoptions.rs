use serde::{Deserialize, Serialize};

use crate::Color;

/// The rough options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "rough_options")]
pub struct RoughOptions {
    /// limits the maximum offset the randomness is allowed to create.
    #[serde(rename = "max_randomness_offset")]
    pub max_randomness_offset: f64,
    /// indicating how rough the drawing is. Good values are between 1 and 10
    #[serde(rename = "roughness")]
    pub roughness: f64,
    /// how curvy the lines are when drawing a sketch. 0 is a straight line.
    #[serde(rename = "bowing")]
    pub bowing: f64,
    /// An optional stroke color. When set to None, no stroke outline is produced
    #[serde(rename = "stroke")]
    pub stroke_color: Option<Color>,
    /// the stroke width
    #[serde(rename = "stroke_width")]
    pub stroke_width: f64,
    /// when drawing ellipses, circles and arcs this sets the generated dimensions in comparison to the specified dimensions
    /// A value of 1.0 means the generated dimensions are almost 100% accurate.
    #[serde(rename = "curve_fitting")]
    pub curve_fitting: f64,
    /// the tightness of the curve
    #[serde(rename = "curve_tightness")]
    pub curve_tightness: f64,
    /// The number of points when estimating curved shapes.
    #[serde(rename = "curve_stepcount")]
    pub curve_stepcount: f64,
    /// an optional fill color. When set to None no fill is produced.
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
    /// the fill style
    #[serde(rename = "fill_style")]
    pub fill_style: FillStyle,
    /// the fill weight. When the fill style produces lines, this is the width.
    /// with dots this is the diameter
    #[serde(rename = "fill_weight")]
    pub fill_weight: f64,
    /// The angle of the hachure lines in degrees.
    #[serde(rename = "hachure_angle")]
    pub hachure_angle: f64,
    /// The gap between the hachure lines.
    #[serde(rename = "hachure_gap")]
    pub hachure_gap: f64,
    /// When generating paths this simplifies the shape.
    /// Values should be between 0.0 and 1.0, meaning 0.0 is no simplification.
    /// a value of 0.5 means the number of generated points is halved.
    #[serde(rename = "simplification")]
    pub simplification: f64,
    /// when filling the shape with the FillStyle::Dashed style this is the offset of the dashes
    #[serde(rename = "dash_offset")]
    pub dash_offset: f64,
    /// when filling the shape with the FillStyle::Dashed style this is the gaps between the dashes
    #[serde(rename = "dash_gap")]
    pub dash_gap: f64,
    /// when filling the shape with the FillStyle::Zigzag style this is the width of the zig-zag triangle.
    #[serde(rename = "zigzag_offset")]
    pub zigzag_offset: f64,
    /// an optional seed for creating random values used in shape generation.
    /// When using the same seed the generator produces the same shape.
    #[serde(rename = "seed")]
    pub seed: Option<u64>,
    /// If this vector has values, the strokes are dashed.
    #[serde(rename = "stroke_line_dash")]
    pub stroke_line_dash: Vec<f64>,
    /// The offset of the dashs, when they exist
    #[serde(rename = "stroke_line_dash_offset")]
    pub stroke_line_dash_offset: f64,
    /// like stroke line dash, but for the fill
    #[serde(rename = "fill_line_dash")]
    pub fill_line_dash: Vec<f64>,
    /// like stroke line dash offset, but for the fill
    #[serde(rename = "fill_line_dash_offset")]
    pub fill_line_dash_offset: f64,
    /// disables multiple stroke generation for a sketched look
    #[serde(rename = "disable_multistroke")]
    pub disable_multistroke: bool,
    /// disables multiple fill stroke generation for a sketched look
    #[serde(rename = "disable_multistroke_fill")]
    pub disable_multistroke_fill: bool,
    /// Enables the preservation of the end points when generating a shape.
    #[serde(rename = "preserve_vertices")]
    pub preserve_vertices: bool,
    #[serde(rename = "fixed_decimal_place_digits")]
    /// TODO: explain
    pub fixed_decimal_place_digits: f64,
}

impl Default for RoughOptions {
    fn default() -> Self {
        Self {
            max_randomness_offset: 2.0,
            roughness: Self::ROUGHNESS_DEFAULT,
            bowing: Self::BOWING_DEFAULT,
            stroke_color: Some(Color::BLACK),
            stroke_width: Self::STROKE_WIDTH_DEFAULT,
            curve_fitting: 0.95,
            curve_tightness: 0.0,
            curve_stepcount: Self::CURVESTEPCOUNT_DEFAULT,
            fill_color: None,
            fill_style: FillStyle::Hachure,
            fill_weight: -1.0,
            hachure_angle: -41.0,
            hachure_gap: -1.0,
            simplification: 0.0,
            dash_offset: -1.0,
            dash_gap: -1.0,
            zigzag_offset: -1.0,
            seed: None,
            stroke_line_dash: Vec::new(),
            stroke_line_dash_offset: 0.0,
            fill_line_dash: Vec::new(),
            fill_line_dash_offset: 0.0,
            disable_multistroke: false,
            disable_multistroke_fill: false,
            preserve_vertices: false,
            fixed_decimal_place_digits: 0.0,
        }
    }
}

impl RoughOptions {
    /// The margin for the bounds of shapes composed with RoughOptions
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
    pub const CURVESTEPCOUNT_DEFAULT: f64 = 9.0;
}

/// available Fill styles
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FillStyle {
    /// Solid
    Solid,
    /// Hachure
    Hachure,
    /// Zigzag
    Zigzag,
    /// Zigzagline
    ZigzagLine,
    /// Crosshatch
    Crosshatch,
    /// Dots
    Dots,
    /// Sunburst
    Sunburst,
    /// Dashed
    Dashed,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self::Hachure
    }
}
