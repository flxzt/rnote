use anyhow::Context;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};
use svg::node::element;
use svg::Node;

use crate::render;
use rnote_compose::Color;

#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "pattern_style")]
pub enum PatternStyle {
    #[serde(rename = "none")]
    None = 0,
    #[serde(rename = "lines")]
    Lines,
    #[serde(rename = "grid")]
    Grid,
    #[serde(rename = "dots")]
    Dots,
}

impl Default for PatternStyle {
    fn default() -> Self {
        Self::Dots
    }
}

impl TryFrom<u32> for PatternStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("PatternStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

pub fn gen_hline_pattern(
    bounds: Aabb,
    spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::random_id_prefix() + "_bg_hline_pattern";

    let line_offset = line_width * 0.5;

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", bounds.extents()[0])
            .set("height", spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", 0_f64)
                    .set("y1", line_offset)
                    .set("x2", bounds.extents()[0])
                    .set("y2", line_offset),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{})", pattern_id));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

pub fn gen_grid_pattern(
    bounds: Aabb,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::random_id_prefix() + "_bg_grid_pattern";

    let line_offset = line_width * 0.5;

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", column_spacing)
            .set("height", row_spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", 0_f64)
                    .set("y1", line_offset)
                    .set("x2", column_spacing)
                    .set("y2", line_offset),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", 0_f64)
                    .set("x2", line_offset)
                    .set("y2", row_spacing),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{})", pattern_id));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

pub fn gen_dots_pattern(
    bounds: Aabb,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    dots_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::random_id_prefix() + "_bg_dots_pattern";

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", column_spacing)
            .set("height", row_spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Rectangle::new()
                    .set("stroke", "none")
                    .set("fill", color.to_css_color_attr())
                    .set("x", 0_f64)
                    .set("y", 0_f64)
                    .set("width", dots_width)
                    .set("height", dots_width)
                    .set("rx", dots_width / 3.0)
                    .set("ry", dots_width / 3.0),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{})", pattern_id));
    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "background")]
pub struct Background {
    #[serde(rename = "color")]
    pub color: Color,
    #[serde(rename = "pattern")]
    pub pattern: PatternStyle,
    #[serde(rename = "pattern_size")]
    pub pattern_size: na::Vector2<f64>,
    #[serde(rename = "pattern_color")]
    pub pattern_color: Color,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Self::COLOR_DEFAULT,
            pattern: PatternStyle::default(),
            pattern_size: Self::PATTERN_SIZE_DEFAULT,
            pattern_color: Self::PATTERN_COLOR_DEFAULT,
        }
    }
}

impl Background {
    const TILE_MAX_SIZE: f64 = 128.0;
    const COLOR_DEFAULT: Color = Color::WHITE;
    const PATTERN_SIZE_DEFAULT: na::Vector2<f64> = na::vector![32.0, 32.0];
    const PATTERN_COLOR_DEFAULT: Color = Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 1.0,
    };

    /// Calculates the tile size as multiple of pattern_size with max size TITLE_MAX_SIZE
    pub fn tile_size(&self) -> na::Vector2<f64> {
        let tile_factor =
            na::Vector2::from_element(Self::TILE_MAX_SIZE).component_div(&self.pattern_size);

        let tile_width = if tile_factor[0] > 1.0 {
            tile_factor[0].floor() * self.pattern_size[0]
        } else {
            self.pattern_size[0]
        };
        let tile_height = if tile_factor[1] > 1.0 {
            tile_factor[1].floor() * self.pattern_size[1]
        } else {
            self.pattern_size[1]
        };

        na::vector![tile_width, tile_height]
    }

    /// Generates the background svg, without xml header or svg root
    pub fn gen_svg(&self, bounds: Aabb, with_pattern: bool) -> Result<render::Svg, anyhow::Error> {
        let mut svg_group = element::Group::new();

        // background color
        let mut color_rect = element::Rectangle::new().set("fill", self.color.to_css_color_attr());

        color_rect.assign("x", format!("{}px", bounds.mins[0]));
        color_rect.assign("y", format!("{}px", bounds.mins[1]));
        color_rect.assign("width", format!("{}px", bounds.extents()[0]));
        color_rect.assign("height", format!("{}px", bounds.extents()[1]));

        svg_group = svg_group.add(color_rect);

        if with_pattern {
            match self.pattern {
                PatternStyle::None => {}
                PatternStyle::Lines => {
                    svg_group = svg_group.add(gen_hline_pattern(
                        bounds,
                        self.pattern_size[1],
                        self.pattern_color,
                        0.5,
                    ));
                }
                PatternStyle::Grid => {
                    svg_group = svg_group.add(gen_grid_pattern(
                        bounds,
                        self.pattern_size[1],
                        self.pattern_size[0],
                        self.pattern_color,
                        0.5,
                    ));
                }
                PatternStyle::Dots => {
                    svg_group = svg_group.add(gen_dots_pattern(
                        bounds,
                        self.pattern_size[1],
                        self.pattern_size[0],
                        self.pattern_color,
                        1.5,
                    ));
                }
            }
        }

        let svg_data = rnote_compose::utils::svg_node_to_string(&svg_group)
            .context("svg_node_to_string() failed for background.")?;

        Ok(render::Svg { svg_data, bounds })
    }

    pub fn gen_tile_image(&self, image_scale: f64) -> Result<Option<render::Image>, anyhow::Error> {
        let tile_size = self.tile_size();
        let tile_bounds = Aabb::new(na::point![0.0, 0.0], na::point![tile_size[0], tile_size[1]]);
        let svg = self.gen_svg(tile_bounds, true)?;

        Ok(Some(render::Image::gen_image_from_svg(
            svg,
            tile_bounds,
            image_scale,
        )?))
    }
}
