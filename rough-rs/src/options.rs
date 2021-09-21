use svg::node::element;

use crate::utils::{self, merge};


#[derive(Debug, Clone)]
pub struct Options {
    pub max_randomness_offset: Option<f64>,
    pub roughness: Option<f64>,
    pub bowing: Option<f64>,
    pub stroke: Option<utils::Color>,
    pub stroke_width: Option<f64>,
    pub curve_fitting: Option<f64>,
    pub curve_tightness: Option<f64>,
    pub curve_stepcount: Option<f64>,
    pub fill: Option<String>,
    pub fill_style: Option<FillStyle>,
    pub fill_weight: Option<f64>,
    pub hachure_angle: Option<f64>,
    pub hachure_gap: Option<f64>,
    pub simplification: Option<f64>,
    pub dash_offset: Option<f64>,
    pub dash_gap: Option<f64>,
    pub zigzag_offset: Option<f64>,
    pub combine_nested_svg_paths: Option<bool>,
    pub stroke_line_dash: Option<Vec<f64>>,
    pub stroke_line_dash_offset: Option<f64>,
    pub fill_line_dash: Option<Vec<f64>>,
    pub fill_line_dash_offset: Option<f64>,
    pub disable_multistroke: Option<bool>,
    pub disable_multistroke_fill: Option<bool>,
    pub preserve_vertices: Option<bool>,
    pub fixed_decimalplacedigits: Option<f64>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_randomness_offset: None,
            roughness: None,
            bowing: None,
            stroke: None,
            stroke_width: None,
            curve_fitting: None,
            curve_tightness: None,
            curve_stepcount: None,
            fill: None,
            fill_style: None,
            fill_weight: None,
            hachure_angle: None,
            hachure_gap: None,
            simplification: None,
            dash_offset: None,
            dash_gap: None,
            zigzag_offset: None,
            combine_nested_svg_paths: None,
            stroke_line_dash: None,
            stroke_line_dash_offset: None,
            fill_line_dash: None,
            fill_line_dash_offset: None,
            disable_multistroke: None,
            disable_multistroke_fill: None,
            preserve_vertices: None,
            fixed_decimalplacedigits: None,
        }
    }
}

impl Options {
    pub fn merge(self, other: Self) -> Self {
        Self {
            max_randomness_offset: merge(self.max_randomness_offset, other.max_randomness_offset),
            roughness: merge(self.roughness, other.roughness),
            bowing: merge(self.bowing, other.bowing),
            stroke: merge(self.stroke, other.stroke),
            stroke_width: merge(self.stroke_width, other.stroke_width),
            curve_fitting: merge(self.curve_fitting, other.curve_fitting),
            curve_tightness: merge(self.curve_tightness, other.curve_tightness),
            curve_stepcount: merge(self.curve_stepcount, other.curve_stepcount),
            fill: merge(self.fill, other.fill),
            fill_style: merge(self.fill_style, other.fill_style),
            fill_weight: merge(self.fill_weight, other.fill_weight),
            hachure_angle: merge(self.hachure_angle, other.hachure_angle),
            hachure_gap: merge(self.hachure_gap, other.hachure_gap),
            simplification: merge(self.simplification, other.simplification),
            dash_offset: merge(self.dash_offset, other.dash_offset),
            dash_gap: merge(self.dash_gap, other.dash_gap),
            zigzag_offset: merge(self.zigzag_offset, other.zigzag_offset),
            combine_nested_svg_paths: merge(
                self.combine_nested_svg_paths,
                other.combine_nested_svg_paths,
            ),
            stroke_line_dash: merge(self.stroke_line_dash.to_owned(), other.stroke_line_dash),
            stroke_line_dash_offset: merge(
                self.stroke_line_dash_offset,
                other.stroke_line_dash_offset,
            ),
            fill_line_dash: merge(self.fill_line_dash, other.fill_line_dash),
            fill_line_dash_offset: merge(self.fill_line_dash_offset, other.fill_line_dash_offset),
            disable_multistroke: merge(self.disable_multistroke, other.disable_multistroke),
            disable_multistroke_fill: merge(
                self.disable_multistroke_fill,
                other.disable_multistroke_fill,
            ),
            preserve_vertices: merge(self.preserve_vertices, other.preserve_vertices),
            fixed_decimalplacedigits: merge(
                self.fixed_decimalplacedigits,
                other.fixed_decimalplacedigits,
            ),
        }
    }

    pub fn apply_to_path(&self, mut path: element::Path) -> element::Path {
        if let Some(fill) = self.fill.clone() {
            path = path.set("fill", fill)
        };
        if let Some(stroke) = self.stroke.clone() {
            path = path.set("stroke", stroke.to_css_color())
        };
        if let Some(stroke_width) = self.stroke_width {
            path = path.set("stroke-width", stroke_width)
        };
        if let Some(stroke_line_dash) = &self.stroke_line_dash {
            path = path.set(
                "stroke-dasharray",
                stroke_line_dash
                    .iter()
                    .map(|&no| {
                        format! {"{}", no}
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
            )
        };
        if let Some(stroke_line_dash_offset) = self.fill_line_dash_offset {
            path = path.set("stroke-dashoffset", stroke_line_dash_offset)
        };

        path
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FillStyle {
    Solid,
    Hachure,
    Zigzag,
    ZigzagLine,
    Crosshatch,
    Dots,
    Sunburst,
    Dashed,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self::Hachure
    }
}