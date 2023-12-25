// Imports
use crate::render;
use anyhow::Context;
use p2d::bounding_volume::Aabb;
use rnote_compose::Color;
use serde::{Deserialize, Serialize};
use svg::node::element;
use svg::Node;

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
    #[serde(rename = "isometric_grid")]
    IsometricGrid,
    #[serde(rename = "isometric_dots")]
    IsometricDots,
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

/// 3_f64.sqrt()
const SQRT_THREE: f64 = 1.7320508075688772;
/// 3_f64.sqrt() / 2_f64
const HALF_SQRT_THREE: f64 = SQRT_THREE / 2_f64;
/// 3_f64.sqrt() / 4_f64
const QUARTER_SQRT_THREE: f64 = SQRT_THREE / 4_f64;

fn gen_hline_pattern(
    bounds: Aabb,
    spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_hline_pattern";

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

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn gen_grid_pattern(
    bounds: Aabb,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_grid_pattern";

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

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn gen_dots_pattern(
    bounds: Aabb,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    dots_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_dots_pattern";

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

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));
    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn calc_width_iso_pattern(spacing: f64) -> f64 {
    spacing * SQRT_THREE
}

fn gen_iso_grid_pattern(
    bounds: Aabb,
    spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    // spacing: side length of the equilateral triangle
    // pattern_width: two times the height of the equilateral triangle

    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_iso_grid_pattern";
    let pattern_width = calc_width_iso_pattern(spacing);

    let line_offset = line_width * 0.5;

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", pattern_width)
            .set("height", spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", 0_f64)
                    .set("x2", line_offset + pattern_width)
                    .set("y2", spacing),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", spacing)
                    .set("x2", line_offset + pattern_width)
                    .set("y2", 0_f64),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset + pattern_width * 0.5)
                    .set("y1", 0_f64)
                    .set("x2", line_offset + pattern_width * 0.5)
                    .set("y2", spacing),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", 0_f64)
                    .set("x2", line_offset)
                    .set("y2", spacing),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn gen_iso_dots_pattern(
    bounds: Aabb,
    spacing: f64,
    color: Color,
    hexagon_height: f64,
) -> svg::node::element::Element {
    // spacing: side length of the equilateral triangle
    // pattern_width: two times the height of the equilateral triangle

    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_iso_dots_pattern";
    let pattern_width = calc_width_iso_pattern(spacing);

    let hexagon_path = |x_offset: f64, y_offset: f64| {
        element::path::Data::new()
            .move_to((
                x_offset + QUARTER_SQRT_THREE * hexagon_height,
                y_offset + hexagon_height,
            ))
            .line_to((
                x_offset + HALF_SQRT_THREE * hexagon_height,
                y_offset + 0.75 * hexagon_height,
            ))
            .line_to((
                x_offset + HALF_SQRT_THREE * hexagon_height,
                y_offset + 0.25 * hexagon_height,
            ))
            .line_to((x_offset + QUARTER_SQRT_THREE * hexagon_height, y_offset))
            .line_to((x_offset, y_offset + 0.25 * hexagon_height))
            .line_to((x_offset, y_offset + 0.75 * hexagon_height))
            .close()
    };

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", pattern_width)
            .set("height", spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Path::new()
                    .set("stroke", "none")
                    .set("fill", color.to_css_color_attr())
                    .set("d", hexagon_path(0.0, 0.0)),
            )
            .add(
                element::Path::new()
                    .set("stroke", "none")
                    .set("fill", color.to_css_color_attr())
                    .set("d", hexagon_path(pattern_width * 0.5, spacing * 0.5)),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));
    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "background")]
pub struct Background {
    #[serde(rename = "color")]
    pub color: Color,
    #[serde(rename = "pattern")]
    pub pattern: PatternStyle,
    #[serde(
        rename = "pattern_size",
        with = "rnote_compose::serialize::na_vector2_f64_dp3"
    )]
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
    const LINE_WIDTH: f64 = 0.5;
    const DOTS_WIDTH: f64 = 1.5;
    const HEXAGON_HEIGHT: f64 = 2.0;

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
    pub(crate) fn tile_size(&self) -> na::Vector2<f64> {
        let pattern_size = match self.pattern {
            PatternStyle::None => {
                na::vector![Self::TILE_MAX_SIZE, Self::TILE_MAX_SIZE]
            }
            PatternStyle::Lines => {
                na::vector![Self::TILE_MAX_SIZE, self.pattern_size[1]]
            }
            PatternStyle::IsometricGrid | PatternStyle::IsometricDots => {
                na::vector![
                    calc_width_iso_pattern(self.pattern_size[1]),
                    self.pattern_size[1]
                ]
            }
            _ => self.pattern_size,
        };

        let tile_factor =
            na::Vector2::from_element(Self::TILE_MAX_SIZE).component_div(&pattern_size);

        let tile_width = if tile_factor[0] > 1.0 {
            tile_factor[0].floor() * pattern_size[0]
        } else {
            pattern_size[0]
        };
        let tile_height = if tile_factor[1] > 1.0 {
            tile_factor[1].floor() * pattern_size[1]
        } else {
            pattern_size[1]
        };

        na::vector![tile_width, tile_height]
    }

    /// Generate the background svg, without Xml header or Svg root.
    pub(crate) fn gen_svg(
        &self,
        bounds: Aabb,
        with_pattern: bool,
        optimize_printing: bool,
    ) -> Result<render::Svg, anyhow::Error> {
        let (color, pattern_color) = if optimize_printing {
            if self.color.luma() > 0.5 {
                // original background color is bright, don't invert pattern color
                (Color::WHITE, self.pattern_color)
            } else {
                // original background color is dark, invert pattern color
                (
                    Color::WHITE,
                    self.pattern_color.to_inverted_brightness_color(),
                )
            }
        } else {
            (self.color, self.pattern_color)
        };

        // background color
        let mut color_rect = element::Rectangle::new().set("fill", color.to_css_color_attr());
        color_rect.assign("x", format!("{}px", bounds.mins[0]));
        color_rect.assign("y", format!("{}px", bounds.mins[1]));
        color_rect.assign("width", format!("{}px", bounds.extents()[0]));
        color_rect.assign("height", format!("{}px", bounds.extents()[1]));

        let mut svg_group = element::Group::new();
        svg_group = svg_group.add(color_rect);

        if with_pattern {
            match self.pattern {
                PatternStyle::None => {}
                PatternStyle::Lines => {
                    svg_group = svg_group.add(gen_hline_pattern(
                        bounds,
                        self.pattern_size[1],
                        pattern_color,
                        Self::LINE_WIDTH,
                    ));
                }
                PatternStyle::Grid => {
                    svg_group = svg_group.add(gen_grid_pattern(
                        bounds,
                        self.pattern_size[1],
                        self.pattern_size[0],
                        pattern_color,
                        Self::LINE_WIDTH,
                    ));
                }
                PatternStyle::Dots => {
                    svg_group = svg_group.add(gen_dots_pattern(
                        bounds,
                        self.pattern_size[1],
                        self.pattern_size[0],
                        pattern_color,
                        Self::DOTS_WIDTH,
                    ));
                }
                PatternStyle::IsometricGrid => {
                    svg_group = svg_group.add(gen_iso_grid_pattern(
                        bounds,
                        self.pattern_size[1],
                        pattern_color,
                        Self::LINE_WIDTH,
                    ));
                }
                PatternStyle::IsometricDots => {
                    svg_group = svg_group.add(gen_iso_dots_pattern(
                        bounds,
                        self.pattern_size[1],
                        pattern_color,
                        Self::HEXAGON_HEIGHT,
                    ));
                }
            }
        }

        let svg_data = rnote_compose::utils::svg_node_to_string(&svg_group)
            .context("Converting Svg group node to String failed.")?;

        Ok(render::Svg { svg_data, bounds })
    }

    pub(crate) fn gen_tile_image(&self, image_scale: f64) -> Result<render::Image, anyhow::Error> {
        let tile_bounds = Aabb::new(na::point![0.0, 0.0], self.tile_size().into());
        self.gen_svg(tile_bounds, true, false)?
            .gen_image(image_scale)
    }

    pub(crate) fn draw_to_cairo(
        &self,
        cx: &cairo::Context,
        bounds: Aabb,
        with_pattern: bool,
        optimize_printing: bool,
    ) -> anyhow::Result<()> {
        let mut background_svg = self.gen_svg(bounds, with_pattern, optimize_printing)?;
        background_svg.wrap_svg_root(Some(bounds), Some(bounds), false);
        background_svg.draw_to_cairo(cx)
    }
}
