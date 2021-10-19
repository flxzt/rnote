use std::error::Error;

use gtk4::gsk::IsRenderNode;
use gtk4::{gdk, glib, gsk, Snapshot, Widget};
use serde::{Deserialize, Serialize};
use svg::node::element;

use crate::strokes::{compose, render};
use crate::utils;

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
}

impl Default for PatternStyle {
    fn default() -> Self {
        Self::None
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

    while y_offset < bounds.maxs[1] {
        group = group.add(
            element::Line::new()
                .set("stroke-width", line_width)
                .set("stroke", color.to_css_color())
                .set("x1", bounds.mins[0])
                .set("y1", y_offset - line_width / 2.0)
                .set("x2", bounds.maxs[0])
                .set("y2", y_offset - line_width / 2.0),
        );

        y_offset += spacing
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

    let mut x_offset = bounds.mins[0] + column_spacing;
    while x_offset < bounds.maxs[0] {
        // vertical lines
        group = group.add(
            element::Line::new()
                .set("stroke-width", line_width)
                .set("stroke", color.to_css_color())
                .set("x1", x_offset - line_width / 2.0)
                .set("y1", bounds.mins[1])
                .set("x2", x_offset - line_width / 2.0)
                .set("y2", bounds.maxs[1]),
        );

        x_offset += column_spacing
    }

    let mut y_offset = bounds.mins[1] + row_spacing;
    while y_offset < bounds.maxs[1] {
        // horizontal lines
        group = group.add(
            element::Line::new()
                .set("stroke-width", line_width)
                .set("stroke", color.to_css_color())
                .set("x1", bounds.mins[0])
                .set("y1", y_offset - line_width / 2.0)
                .set("x2", bounds.maxs[0])
                .set("y2", y_offset - line_width / 2.0),
        );

        y_offset += row_spacing
    }
    group.into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Background {
    color: utils::Color,
    pattern: PatternStyle,
    #[serde(skip, default = "render::default_rendernode")]
    rendernode: gsk::RenderNode,
    #[serde(skip)]
    current_scalefactor: f64,
    #[serde(skip)]
    current_bounds: p2d::bounding_volume::AABB,
    #[serde(skip)]
    texture_buffer: Option<gdk::Texture>,
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
            rendernode: render::default_rendernode(),
            current_scalefactor: 1.0,
            current_bounds: p2d::bounding_volume::AABB::new_invalid(),
            texture_buffer: None,
        }
    }
}

impl Background {
    pub const PATTERN_SIZE_DEFAULT: f64 = 20.0;

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

    pub fn draw(&self, snapshot: &Snapshot) {
        snapshot.append_node(&self.rendernode);
    }

    pub fn update_rendernode(
        &mut self,
        scalefactor: f64,
        sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &render::Renderer,
        active_widget: &Widget,
        force_regenerate: bool,
    ) -> Result<(), Box<dyn Error>> {
        // use texture_buffer if bounds and scale havent changed
        if !force_regenerate
            && sheet_bounds == self.current_bounds
            && self.texture_buffer.is_some()
            && scalefactor == self.current_scalefactor
        {
            if let Some(texture_buffer) = &self.texture_buffer {
                self.rendernode = gsk::TextureNode::new(
                    texture_buffer,
                    &utils::aabb_to_graphene_rect(self.current_bounds),
                )
                .upcast();
                return Ok(());
            }
        }

        if let Ok(new_rendernode) = self.gen_rendernode(scalefactor, sheet_bounds, renderer) {
            let new_texture = render::render_node_to_texture(active_widget, &new_rendernode)?;
            self.rendernode = new_rendernode;
            self.texture_buffer = new_texture;
            self.current_scalefactor = scalefactor;
            self.current_bounds = sheet_bounds;
        } else {
            log::error!("failed to gen_rendernode() in update_rendernode() of background");
            return Ok(());
        }

        Ok(())
    }

    pub fn gen_rendernode(
        &self,
        scalefactor: f64,
        sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
        renderer.gen_rendernode(
            sheet_bounds,
            scalefactor,
            compose::add_xml_header(self.gen_svg_data(sheet_bounds)?.as_str()).as_str(),
        )
    }

    pub fn gen_svg_data(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
    ) -> Result<String, Box<dyn Error>> {
        let mut svg = String::from("");

        let mut group = element::Group::new();

        // background color
        let color_rect = element::Rectangle::new()
            .set("x", sheet_bounds.mins[0])
            .set("y", sheet_bounds.mins[1])
            .set("width", sheet_bounds.maxs[0] - sheet_bounds.mins[0])
            .set("height", sheet_bounds.maxs[1] - sheet_bounds.mins[1])
            .set("fill", self.color.to_css_color());
        group = group.add(color_rect);

        match self.pattern {
            PatternStyle::None => {}
            PatternStyle::Lines => {
                group = group.add(gen_horizontal_line_pattern(
                    sheet_bounds,
                    64.0,
                    utils::Color::new(0.3, 0.4, 0.9, 0.5),
                    1.0,
                ));
            }
            PatternStyle::Grid => {
                group = group.add(gen_grid_pattern(
                    sheet_bounds,
                    32.0,
                    32.0,
                    utils::Color::new(0.3, 0.4, 0.9, 0.5),
                    1.0,
                ));
            }
        }
        svg.push_str(rough_rs::node_to_string(&group)?.as_str());

        let svg = compose::wrap_svg(svg.as_str(), Some(sheet_bounds), None, false, false);
        //println!("{}", svg);
        Ok(svg)
    }
}
