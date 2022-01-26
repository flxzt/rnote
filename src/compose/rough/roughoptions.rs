use crate::compose::color::Color;
use serde::{Deserialize, Serialize};
use svg::node::element;

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
    /// an optional seed for creating random values used in shape generation.
    /// When using the same seed the generator produces the same shape.
    #[serde(rename = "seed")]
    pub seed: Option<u64>,
    /// An optional stroke color. When set to None, no stroke outline is produced
    #[serde(rename = "stroke")]
    pub stroke: Option<Color>,
    /// the stroke width
    #[serde(rename = "stroke_width")]
    pub stroke_width: f64,
    /// an optional fill color. When set to None no fill is produced.
    #[serde(rename = "fill")]
    pub fill: Option<Color>,
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
    /// The number of points when estimating curved shapes.
    #[serde(rename = "curve_stepcount")]
    pub curve_stepcount: f64,
    /// when drawing ellipses, circles and arcs this sets the generated dimensions in comparison to the specified dimensions
    /// A value of 1.0 means the generated dimensions are almost 100% accurate.
    #[serde(rename = "curve_fitting")]
    pub curve_fitting: f64,
    /// the tightness of the curve
    #[serde(rename = "curve_tightness")]
    pub curve_tightness: f64,
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
    /// Enables the preservation of the end points when generating a shape.
    #[serde(rename = "preserve_vertices")]
    pub preserve_vertices: bool,
}

impl Default for RoughOptions {
    fn default() -> Self {
        Self {
            max_randomness_offset: 2.0,
            roughness: Self::ROUGHNESS_DEFAULT,
            bowing: Self::BOWING_DEFAULT,
            seed: None,
            stroke: Some(Color::BLACK),
            stroke_width: Self::STROKE_WIDTH_DEFAULT,
            fill: None,
            fill_style: FillStyle::Hachure,
            fill_weight: 0.5,
            hachure_angle: -41.0,
            hachure_gap: 4.0,
            curve_stepcount: Self::CURVESTEPCOUNT_DEFAULT,
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

impl RoughOptions {
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

    pub(super) fn apply_to_line(&self, mut path: element::Path) -> element::Path {
        path = if let Some(stroke) = self.stroke {
            path.set("stroke", stroke.to_css_color())
        } else {
            path.set("stroke", "none")
        };
        path = path.set("stroke-width", self.stroke_width);

        // the fill is in generated with the fill_polygon
        path = path.set("fill", "none");

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

    pub(super) fn apply_to_fill_polygon_solid(&self, mut path: element::Path) -> element::Path {
        path = path.set("stroke", "none");

        path = if let Some(fill) = self.fill {
            path.set("fill", fill.to_css_color())
        } else {
            path.set("fill", "none")
        };

        path
    }

    pub(super) fn apply_to_rect(&self, mut rect: element::Path) -> element::Path {
        rect = if let Some(stroke) = self.stroke {
            rect.set("stroke", stroke.to_css_color())
        } else {
            rect.set("stroke", "none")
        };
        rect = rect.set("stroke-width", self.stroke_width);

        // the fill is in generated with the fill_polygon
        rect = rect.set("fill", "none");

        rect = rect.set(
            "stroke-dasharray",
            self.stroke_line_dash
                .iter()
                .map(|&no| {
                    format! {"{}", no}
                })
                .collect::<Vec<String>>()
                .join(" "),
        );
        rect = rect.set("stroke-dashoffset", self.stroke_line_dash_offset);

        rect
    }

    pub(super) fn apply_to_ellipse(&self, mut ellipse_path: element::Path) -> element::Path {
        ellipse_path = if let Some(stroke) = self.stroke {
            ellipse_path.set("stroke", stroke.to_css_color())
        } else {
            ellipse_path.set("stroke", "none")
        };
        ellipse_path = ellipse_path.set("stroke-width", self.stroke_width);

        // the fill is in generated with the fill_polygon
        ellipse_path = ellipse_path.set("fill", "none");

        ellipse_path = ellipse_path.set(
            "stroke-dasharray",
            self.stroke_line_dash
                .iter()
                .map(|&no| {
                    format! {"{}", no}
                })
                .collect::<Vec<String>>()
                .join(" "),
        );
        ellipse_path = ellipse_path.set("stroke-dashoffset", self.stroke_line_dash_offset);

        ellipse_path
    }

    /// stroke width
    pub fn stroke_width(&self) -> f64 {
        self.stroke_width
    }

    /// set stroke width
    pub fn set_stroke_width(&mut self, stroke_width: f64) {
        self.stroke_width = stroke_width.clamp(Self::STROKE_WIDTH_MIN, Self::STROKE_WIDTH_MAX);
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

    /// Returns the multistroke
    pub fn curve_stepcount(&self) -> f64 {
        self.curve_stepcount
    }

    /// Sets multistroke
    pub fn set_curve_stepcount(&mut self, curve_stepcount: f64) {
        self.curve_stepcount = curve_stepcount;
    }

    /// Returns multistroke
    pub fn multistroke(&self) -> bool {
        !self.disable_multistroke
    }

    /// Sets the multistroke
    pub fn set_multistroke(&mut self, multistroke: bool) {
        self.disable_multistroke = !multistroke;
    }

    /// Returns preserve_vertices
    pub fn preserve_vertices(&self) -> bool {
        !self.preserve_vertices
    }

    /// Sets preserve_vertices
    pub fn set_preserve_vertices(&mut self, preserve_vertices: bool) {
        self.preserve_vertices = !preserve_vertices;
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
