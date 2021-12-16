use gtk4::{glib, gsk, Snapshot};
use serde::{Deserialize, Serialize};
use svg::node::element;

use crate::render::Renderer;
use crate::{compose, geometry, render, utils};

#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    glib::GEnum,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
)]
#[repr(u32)]
#[genum(type_name = "PatternStyle")]
pub enum PatternStyle {
    #[genum(name = "None", nick = "none")]
    None = 0,
    #[genum(name = "Lines", nick = "lines")]
    Lines,
    #[genum(name = "Grid", nick = "grid")]
    Grid,
    #[genum(name = "Dots", nick = "dots")]
    Dots,
}

impl Default for PatternStyle {
    fn default() -> Self {
        Self::Dots
    }
}

pub fn gen_horizontal_line_pattern(
    bounds: p2d::bounding_volume::AABB,
    spacing: f64,
    color: utils::Color,
    line_width: f64,
) -> svg::node::element::Element {
    let mut group = element::Group::new();

    let mut y_offset = bounds.mins[1] + spacing;

    if spacing > 1.0 {
        while y_offset <= bounds.maxs[1] {
            group = group.add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color())
                    .set("x1", bounds.mins[0])
                    .set("y1", y_offset - line_width)
                    .set("x2", bounds.maxs[0])
                    .set("y2", y_offset - line_width),
            );

            y_offset += spacing
        }
    }

    group.into()
}

pub fn gen_grid_pattern(
    bounds: p2d::bounding_volume::AABB,
    row_spacing: f64,
    column_spacing: f64,
    color: utils::Color,
    line_width: f64,
) -> svg::node::element::Element {
    let mut group = element::Group::new();

    if column_spacing > 1.0 && row_spacing > 1.0 {
        let mut x_offset = bounds.mins[0] + column_spacing;
        while x_offset <= bounds.maxs[0] {
            // vertical lines
            group = group.add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color())
                    .set("x1", x_offset - line_width)
                    .set("y1", bounds.mins[1])
                    .set("x2", x_offset - line_width)
                    .set("y2", bounds.maxs[1]),
            );

            x_offset += column_spacing
        }

        let mut y_offset = bounds.mins[1] + row_spacing;
        while y_offset <= bounds.maxs[1] {
            // horizontal lines
            group = group.add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color())
                    .set("x1", bounds.mins[0])
                    .set("y1", y_offset - line_width)
                    .set("x2", bounds.maxs[0])
                    .set("y2", y_offset - line_width),
            );

            y_offset += row_spacing
        }
    }
    group.into()
}

pub fn gen_dots_pattern(
    bounds: p2d::bounding_volume::AABB,
    row_spacing: f64,
    column_spacing: f64,
    color: utils::Color,
    dots_width: f64,
) -> svg::node::element::Element {
    let mut group = element::Group::new();

    // Only generate pattern if spacings are sufficiently large
    if column_spacing > 1.0 && row_spacing > 1.0 {
        let mut x_offset = bounds.mins[0] + column_spacing;
        while x_offset <= bounds.maxs[0] {
            let mut y_offset = bounds.mins[1] + row_spacing;
            while y_offset <= bounds.maxs[1] {
                // row by row
                group = group.add(
                    element::Rectangle::new()
                        .set("stroke", "none")
                        .set("fill", color.to_css_color())
                        .set("x", x_offset - dots_width)
                        .set("y", y_offset - dots_width)
                        .set("width", dots_width)
                        .set("height", dots_width),
                );

                y_offset += row_spacing;
            }

            x_offset += column_spacing;
        }
    }

    group.into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Background {
    color: utils::Color,
    pattern: PatternStyle,
    pattern_size: na::Vector2<f64>,
    pattern_color: utils::Color,
    #[serde(skip, default = "render::default_rendernode")]
    rendernode: gsk::RenderNode,
    #[serde(skip)]
    current_zoom: f64,
    #[serde(skip)]
    current_bounds: p2d::bounding_volume::AABB,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: utils::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            pattern: PatternStyle::default(),
            pattern_size: na::Vector2::<f64>::from_element(Self::PATTERN_SIZE_DEFAULT),
            pattern_color: utils::Color {
                r: 0.3,
                g: 0.7,
                b: 1.0,
                a: 1.0,
            },
            rendernode: render::default_rendernode(),
            current_zoom: 1.0,
            current_bounds: p2d::bounding_volume::AABB::new_invalid(),
        }
    }
}

impl Background {
    pub const PATTERN_SIZE_DEFAULT: f64 = 20.0;
    pub const TILE_MAX_SIZE: f64 = 512.0;

    pub fn import_background(&mut self, background: &Self) {
        self.color = background.color;
        self.pattern = background.pattern;
        self.pattern_size = background.pattern_size;
        self.pattern_color = background.pattern_color;
        self.pattern_color = background.pattern_color;
    }

    pub fn color(&self) -> utils::Color {
        self.color
    }

    pub fn set_color(&mut self, color: utils::Color) {
        self.color = color;
    }

    pub fn pattern(&self) -> PatternStyle {
        self.pattern
    }

    pub fn set_pattern(&mut self, pattern: PatternStyle) {
        self.pattern = pattern;
    }

    pub fn pattern_color(&self) -> utils::Color {
        self.pattern_color
    }

    pub fn set_pattern_color(&mut self, pattern_color: utils::Color) {
        self.pattern_color = pattern_color;
    }

    pub fn pattern_size(&self) -> na::Vector2<f64> {
        self.pattern_size
    }

    pub fn set_pattern_size(&mut self, pattern_size: na::Vector2<f64>) {
        self.pattern_size = pattern_size;
    }

    pub fn draw(&self, snapshot: &Snapshot) {
        snapshot.append_node(&self.rendernode);
    }

    pub fn update_rendernode(
        &mut self,
        renderer: &Renderer,
        zoom: f64,
        sheet_bounds: p2d::bounding_volume::AABB,
        force_regenerate: bool,
    ) -> Result<(), anyhow::Error> {
        if force_regenerate || sheet_bounds != self.current_bounds || zoom != self.current_zoom {
            match self.gen_rendernode(renderer, zoom, sheet_bounds) {
                Ok(Some(new_rendernode)) => {
                    self.current_zoom = zoom;
                    self.current_bounds = sheet_bounds;
                    self.rendernode = new_rendernode;
                }
                Err(e) => {
                    log::error!(
                        "gen_rendernode() failed in update_rendernode() of background with Err: {}",
                        e
                    );
                }
                _ => {
                    log::error!(
                        "gen_rendernode() returned None in update_rendernode() of background"
                    );
                }
            }
        }

        Ok(())
    }

    pub fn gen_rendernode(
        &mut self,
        renderer: &Renderer,
        zoom: f64,
        sheet_bounds: p2d::bounding_volume::AABB,
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        let snapshot = Snapshot::new();

        // Calculate tile size as multiple of pattern_size with max size TITLE_MAX_SIZE
        let tile_factor =
            na::Vector2::<f64>::from_element(Self::TILE_MAX_SIZE).component_div(&self.pattern_size);

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
        let tile_size = na::vector![tile_width, tile_height];

        let tile_bounds = p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![tile_size[0], tile_size[1]],
        );
        let svg_string = compose::add_xml_header(self.gen_svg_data(tile_bounds)?.as_str());
        snapshot.push_clip(&geometry::aabb_to_graphene_rect(geometry::aabb_scale(
            sheet_bounds,
            zoom,
        )));

        // Fill with background color just in case there is any space left between the tiles
        snapshot.append_color(
            &self.color.to_gdk(),
            &geometry::aabb_to_graphene_rect(geometry::aabb_scale(sheet_bounds, zoom)),
        );

        let svg = render::Svg {
            bounds: tile_bounds,
            svg_data: svg_string,
        };

        match renderer.gen_image(zoom, &svg) {
            Ok(background_image) => {
                let new_texture = render::image_to_memtexture(&background_image);
                for aabb in geometry::split_aabb_extended(sheet_bounds, tile_size) {
                    snapshot.append_texture(
                        &new_texture,
                        &geometry::aabb_to_graphene_rect(geometry::aabb_scale(aabb, zoom)),
                    );
                }
            }
            Err(e) => {
                log::error!("gen_image_librsvg() for background failed, {}", e);
            }
        };

        snapshot.pop();

        Ok(snapshot.to_node())
    }

    pub fn gen_svg_data(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
    ) -> Result<String, anyhow::Error> {
        let mut svg = String::from("");

        let mut group = element::Group::new();

        // background color
        let color_rect = element::Rectangle::new()
            .set("x", sheet_bounds.mins[0])
            .set("y", sheet_bounds.mins[1])
            .set("width", sheet_bounds.extents()[0])
            .set("height", sheet_bounds.extents()[1])
            .set("fill", self.color.to_css_color());
        group = group.add(color_rect);

        match self.pattern {
            PatternStyle::None => {}
            PatternStyle::Lines => {
                group = group.add(gen_horizontal_line_pattern(
                    sheet_bounds,
                    self.pattern_size[1],
                    self.pattern_color,
                    1.0,
                ));
            }
            PatternStyle::Grid => {
                group = group.add(gen_grid_pattern(
                    sheet_bounds,
                    self.pattern_size[1],
                    self.pattern_size[0],
                    self.pattern_color,
                    1.0,
                ));
            }
            PatternStyle::Dots => {
                group = group.add(gen_dots_pattern(
                    sheet_bounds,
                    self.pattern_size[1],
                    self.pattern_size[0],
                    self.pattern_color,
                    2.0,
                ));
            }
        }
        svg.push_str(
            rough_rs::node_to_string(&group)
                .map_err(|e| {
                    anyhow::anyhow!("rough_rs::node_to_string() failed for background, {}", e)
                })?
                .as_str(),
        );

        let svg = compose::wrap_svg(svg.as_str(), Some(sheet_bounds), None, false, false);
        Ok(svg)
    }
}
