use serde::{Deserialize, Serialize};
use svg::node::element;

use crate::utils::{self};

/// The options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    /// limits the maximum offset the randomness is allowed to create.
    pub max_randomness_offset: f64,
    /// indicating how rough the drawing is. Good values are between 1 and 10
    pub roughness: f64,
    /// how curvy the lines are when drawing a sketch. 0 is a straight line.
    pub bowing: f64,
    /// an optional seed for creating random values used in shape generation.
    /// When using the same seed the generator produces the same shape.
    pub seed: Option<u64>,
    /// An optional stroke color. When set to None, no stroke outline is produced
    pub stroke: Option<utils::Color>,
    /// the stroke width
    pub stroke_width: f64,
    /// an optional fill color. When set to None no fill is produced.
    pub fill: Option<utils::Color>,
    /// the fill style
    pub fill_style: FillStyle,
    /// the fill weight. When the fill style produces lines, this is the width.
    /// with dots this is the diameter
    pub fill_weight: f64,
    /// The angle of the hachure lines in degrees.
    pub hachure_angle: f64,
    /// The gap between to hachure lines.
    pub hachure_gap: f64,
    /// The number of points when estimating curved shapes.
    pub curve_stepcount: f64,
    /// when drawing ellipses, circles and arcs this sets the generated dimensions in comparison to the specified dimensions
    /// A value of 1.0 means the generated dimensions are almost 100% accurate.
    pub curve_fitting: f64,
    /// the tightness of the curve
    pub curve_tightness: f64,
    /// If this vector has values, the strokes are dashed.
    pub stroke_line_dash: Vec<f64>,
    /// The offset of the dashs, when they exist
    pub stroke_line_dash_offset: f64,
    /// like stroke line dash, but for the fill
    pub fill_line_dash: Vec<f64>,
    /// like stroke line dash offset, but for the fill
    pub fill_line_dash_offset: f64,
    /// disables multiple stroke generation for a sketched look
    pub disable_multistroke: bool,
    /// disables multiple fill stroke generation for a sketched look
    pub disable_multistroke_fill: bool,
    /// When generating paths this simplifies the shape.
    /// Values should be between 0.0 and 1.0, meaning 0.0 is no simplification.
    /// a value of 0.5 means the number of generated points is halved.
    pub simplification: f64,
    /// when filling the shape with the FillStyle::Dashed style this is the offset of the dashes
    pub dash_offset: f64,
    /// when filling the shape with the FillStyle::Dashed style this is the gaps between the dashes
    pub dash_gap: f64,
    /// when filling the shape with the FillStyle::Zigzag style this is the width of the zig-zag triangle.
    pub zigzag_offset: f64,
    /// Enables the preservation of the end points when generating a shape.
    pub preserve_vertices: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_randomness_offset: 2.0,
            roughness: Self::ROUGHNESS_DEFAULT,
            bowing: Self::BOWING_DEFAULT,
            seed: None,
            stroke: Some(utils::Color::black()),
            stroke_width: 1.0,
            fill: None,
            fill_style: FillStyle::Hachure,
            fill_weight: 0.5,
            hachure_angle: -41.0,
            hachure_gap: 4.0,
            curve_stepcount: 9.0,
            curve_fitting: 0.95,
            curve_tightness: 0.0,
            stroke_line_dash: Vec::new(),
            stroke_line_dash_offset: 0.0,
            fill_line_dash: Vec::new(),
            fill_line_dash_offset: 0.0,
            disable_multistroke: false,
            disable_multistroke_fill: false,
            simplification: 0.0,
            dash_offset: 0.0,
            dash_gap: 4.0,
            zigzag_offset: 4.0,
            preserve_vertices: false,
        }
    }
}

impl Options {
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

    pub(crate) fn apply_to_path(&self, mut path: element::Path) -> element::Path {
        path = if let Some(stroke) = self.stroke {
            path.set("stroke", stroke.to_css_color())
        } else {
            path.set("stroke", "none")
        };
        path = path.set("stroke-width", self.stroke_width);
        path = if let Some(fill) = self.fill {
            path.set("fill", fill.to_css_color())
        } else {
            path.set("fill", "none")
        };
        path = path.set(
            "stroke-dasharray",
            self.stroke_line_dash
                .iter()
                .map(|&no| {
                    format! {"{}", no}
                })
                .collect::<Vec<String>>()
                .join(" "),
        );
        path = path.set("stroke-dashoffset", self.stroke_line_dash_offset);

        path
    }

    /// Returns the roughness
    pub fn roughness(&self) -> f64 {
        self.roughness
    }

    /// Sets the roughness
    pub fn set_roughness(&mut self, roughness: f64) {
        self.roughness = roughness.clamp(Self::ROUGHNESS_MIN, Self::ROUGHNESS_MAX);
    }

    /// Returns the bowing
    pub fn bowing(&self) -> f64 {
        self.roughness
    }

    /// Sets the bowing
    pub fn set_bowing(&mut self, bowing: f64) {
        self.bowing = bowing.clamp(Self::BOWING_MIN, Self::BOWING_MAX);
    }

    /// Returns the bowing
    pub fn multistroke(&self) -> bool {
        !self.disable_multistroke
    }

    /// Sets the bowing
    pub fn set_multistroke(&mut self, multistroke: bool) {
        self.disable_multistroke = !multistroke;
    }
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
