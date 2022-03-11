use std::sync::{Arc, RwLock};

use anyhow::Context;
use gtk4::{glib, gsk, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};
use svg::node::element;

use crate::compose;
use crate::compose::color::Color;
use crate::compose::geometry::AABBHelpers;
use crate::render::{self, Renderer};

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "PatternStyle")]
#[serde(rename = "pattern_style")]
pub enum PatternStyle {
    #[enum_value(name = "None", nick = "none")]
    #[serde(rename = "none")]
    None = 0,
    #[enum_value(name = "Lines", nick = "lines")]
    #[serde(rename = "lines")]
    Lines,
    #[enum_value(name = "Grid", nick = "grid")]
    #[serde(rename = "grid")]
    Grid,
    #[enum_value(name = "Dots", nick = "dots")]
    #[serde(rename = "dots")]
    Dots,
}

impl Default for PatternStyle {
    fn default() -> Self {
        Self::Dots
    }
}

pub fn gen_hline_pattern(
    bounds: AABB,
    spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = compose::random_id_prefix() + "_bg_hline_pattern";

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
                    .set("stroke", color.to_css_color())
                    .set("x1", 0_f64)
                    .set("y1", 0_f64)
                    .set("x2", bounds.extents()[0])
                    .set("y2", 0_f64),
            ),
    );

    let rect = element::Rectangle::new()
        .set("x", bounds.mins[0])
        .set("y", bounds.mins[1])
        .set("width", bounds.extents()[0])
        .set("height", bounds.extents()[1])
        .set("fill", format!("url(#{})", pattern_id));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

pub fn gen_grid_pattern(
    bounds: AABB,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = compose::random_id_prefix() + "_bg_grid_pattern";

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
                    .set("stroke", color.to_css_color())
                    .set("x1", 0_f64)
                    .set("y1", 0_f64)
                    .set("x2", column_spacing)
                    .set("y2", 0_f64),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color())
                    .set("x1", 0_f64)
                    .set("y1", 0_f64)
                    .set("x2", 0_f64)
                    .set("y2", row_spacing),
            ),
    );

    let rect = element::Rectangle::new()
        .set("x", bounds.mins[0])
        .set("y", bounds.mins[1])
        .set("width", bounds.extents()[0])
        .set("height", bounds.extents()[1])
        .set("fill", format!("url(#{})", pattern_id));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

pub fn gen_dots_pattern(
    bounds: AABB,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    dots_width: f64,
) -> svg::node::element::Element {
    let pattern_id = compose::random_id_prefix() + "_bg_dots_pattern";

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
                    .set("fill", color.to_css_color())
                    .set("x", 0_f64)
                    .set("y", 0_f64)
                    .set("width", dots_width)
                    .set("height", dots_width),
            ),
    );

    let rect = element::Rectangle::new()
        .set("x", bounds.mins[0])
        .set("y", bounds.mins[1])
        .set("width", bounds.extents()[0])
        .set("height", bounds.extents()[1])
        .set("fill", format!("url(#{})", pattern_id));

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
    #[serde(skip)]
    pub image: Option<render::Image>,
    #[serde(skip)]
    rendernode: Option<gsk::RenderNode>,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Self::COLOR_DEFAULT,
            pattern: PatternStyle::default(),
            pattern_size: Self::PATTERN_SIZE_DEFAULT,
            pattern_color: Self::PATTERN_COLOR_DEFAULT,
            image: None,
            rendernode: None,
        }
    }
}

impl Background {
    pub const TILE_MAX_SIZE: f64 = 192.0;
    pub const COLOR_DEFAULT: Color = Color::WHITE;
    pub const PATTERN_SIZE_DEFAULT: na::Vector2<f64> = na::vector![32.0, 32.0];
    pub const PATTERN_COLOR_DEFAULT: Color = Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 1.0,
    };

    pub fn tile_size(&self) -> na::Vector2<f64> {
        // Calculate tile size as multiple of pattern_size with max size TITLE_MAX_SIZE
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
        let tile_size = na::vector![tile_width, tile_height];

        tile_size
    }

    /// Generates the background svg, without xml header or svg root
    pub fn gen_svg(&self, bounds: AABB) -> Result<render::Svg, anyhow::Error> {
        let mut group = element::Group::new();

        // background color
        let color_rect = element::Rectangle::new()
            .set("x", bounds.mins[0])
            .set("y", bounds.mins[1])
            .set("width", bounds.extents()[0])
            .set("height", bounds.extents()[1])
            .set("fill", self.color.to_css_color());
        group = group.add(color_rect);

        match self.pattern {
            PatternStyle::None => {}
            PatternStyle::Lines => {
                group = group.add(gen_hline_pattern(
                    bounds,
                    self.pattern_size[1],
                    self.pattern_color,
                    1.0,
                ));
            }
            PatternStyle::Grid => {
                group = group.add(gen_grid_pattern(
                    bounds,
                    self.pattern_size[1],
                    self.pattern_size[0],
                    self.pattern_color,
                    1.0,
                ));
            }
            PatternStyle::Dots => {
                group = group.add(gen_dots_pattern(
                    bounds,
                    self.pattern_size[1],
                    self.pattern_size[0],
                    self.pattern_color,
                    2.0,
                ));
            }
        }
        let svg_data = compose::svg_node_to_string(&group)
            .map_err(|e| anyhow::anyhow!("node_to_string() failed for background, {}", e))?;

        Ok(render::Svg { svg_data, bounds })
    }

    pub fn gen_image(
        &self,
        renderer: Arc<RwLock<Renderer>>,
        zoom: f64,
        bounds: AABB,
    ) -> Result<Option<render::Image>, anyhow::Error> {
        let svg = self.gen_svg(bounds)?;
        Ok(Some(render::concat_images(
            renderer
                .read()
                .unwrap()
                .gen_images(zoom, vec![svg], bounds)?,
            bounds,
            zoom,
        )?))
    }

    pub fn regenerate_background(
        &mut self,
        zoom: f64,
        sheet_bounds: AABB,
        viewport: Option<AABB>,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        let tile_size = self.tile_size();
        let tile_bounds = AABB::new(na::point![0.0, 0.0], na::point![tile_size[0], tile_size[1]]);

        self.image = self.gen_image(renderer, zoom, tile_bounds)?;

        self.update_rendernode(zoom, sheet_bounds, viewport)?;
        Ok(())
    }

    pub fn gen_rendernode(
        &mut self,
        zoom: f64,
        sheet_bounds: AABB,
        viewport: Option<AABB>,
    ) -> Result<Option<gsk::RenderNode>, anyhow::Error> {
        let snapshot = Snapshot::new();
        let tile_size = self.tile_size();

        snapshot.push_clip(
            &sheet_bounds
                .scale(na::Vector2::from_element(zoom))
                .to_graphene_rect(),
        );

        // Fill with background color just in case there is any space left between the tiles
        snapshot.append_color(
            &self.color.to_gdk(),
            &sheet_bounds
                .scale(na::Vector2::from_element(zoom))
                .to_graphene_rect(),
        );

        if let Some(image) = &self.image {
            let new_texture = render::image_to_memtexture(image)
                .context("image_to_memtexture() failed in gen_rendernode().")?;
            for aabb in sheet_bounds.split_extended_origin_aligned(tile_size) {
                if let Some(viewport) = viewport {
                    if !aabb.intersects(&viewport) {
                        continue;
                    }
                }
                snapshot.append_texture(
                    &new_texture,
                    &aabb
                        .scale(na::Vector2::from_element(zoom))
                        .to_graphene_rect(),
                );
            }
        }

        snapshot.pop();

        Ok(snapshot.to_node())
    }

    pub fn update_rendernode(
        &mut self,
        zoom: f64,
        sheet_bounds: AABB,
        viewport: Option<AABB>,
    ) -> Result<(), anyhow::Error> {
        match self.gen_rendernode(zoom, sheet_bounds, viewport) {
            Ok(new_rendernode) => {
                self.rendernode = new_rendernode;
            }
            Err(e) => {
                log::error!(
                    "gen_rendernode() failed in update_rendernode() of background with Err: {}",
                    e
                );
            }
        }

        Ok(())
    }

    pub fn draw(&self, snapshot: &Snapshot) {
        self.rendernode.iter().for_each(|rendernode| {
            snapshot.append_node(rendernode);
        });
    }
}
