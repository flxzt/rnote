use crate::strokes::strokestyle::InputData;
use crate::{compose, render, utils};

use gtk4::{gsk, Snapshot};
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use svg::node::element;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SelectorStyle {
    Polygon,
    Rectangle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Selector {
    style: SelectorStyle,
    pub path: Vec<InputData>,
    pub bounds: Option<p2d::bounding_volume::AABB>,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
    shown: bool,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            style: SelectorStyle::Polygon,
            path: vec![],
            bounds: None,
            rendernode: render::default_rendernode(),
            shown: false,
        }
    }
}

impl Selector {
    pub const STROKE_DASHARRAY: &'static str = "4 6";
    pub const PATH_WIDTH: f64 = 2.0;
    pub const PATH_COLOR: utils::Color = utils::Color {
        r: 0.7,
        g: 0.7,
        b: 0.7,
        a: 0.7,
    };
    pub const FILL_COLOR: utils::Color = utils::Color {
        r: 0.9,
        g: 0.9,
        b: 0.9,
        a: 0.05,
    };

    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(&self) -> SelectorStyle {
        self.style
    }

    pub fn set_style(&mut self, style: SelectorStyle) {
        self.style = style;
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn new_path(&mut self, inputdata: InputData) {
        self.clear_path();

        self.path.push(inputdata);
        self.update_bounds();
    }

    pub fn add_elem_to_path(&mut self, inputdata: InputData) {
        match self.style {
            SelectorStyle::Polygon => {
                self.path.push(inputdata);
            }
            SelectorStyle::Rectangle => {
                if self.path.len() > 2 {
                    self.path.resize(2, InputData::default());
                }
                self.path.insert(1, inputdata)
            }
        }
        self.update_bounds();
    }

    pub fn clear_path(&mut self) {
        self.bounds = None;
        self.path.clear();
    }

    pub fn update_rendernode(&mut self, zoom: f64, renderer: &render::Renderer) {
        match self.gen_image(zoom, renderer) {
            Ok(Some(image)) => self.rendernode = render::image_to_rendernode(&image, zoom),
            Ok(None) => {
                log::error!("gen_image() in update_rendernode() of selector returned None.");
            }
            Err(e) => log::error!(
                "gen_rendernode() in update_rendernode() for selector failed with Err {}",
                e
            ),
        };
    }

    pub fn gen_image(
        &self,
        zoom: f64,
        renderer: &render::Renderer,
    ) -> Result<Option<render::Image>, anyhow::Error> {
        if let Some(bounds) = self.bounds {
            let svg = render::Svg {
                bounds,
                svg_data: compose::wrap_svg(
                    self.gen_svg_path(na::vector![0.0, 0.0])?.as_str(),
                    None,
                    Some(bounds),
                    true,
                    false,
                ),
            };

            Ok(Some(renderer.gen_image(zoom, &vec![svg], bounds)?))
        } else {
            Ok(None)
        }
    }

    fn update_bounds(&mut self) {
        // Making sure bounds are always outside of coord + width
        let mut path_iter = self.path.iter();
        if let Some(first) = path_iter.next() {
            let mut new_bounds = p2d::bounding_volume::AABB::new(
                na::Point2::from(first.pos() - na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH]),
                na::Point2::from(first.pos() + na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH]),
            );

            path_iter.for_each(|inputdata| {
                let pos_bounds = p2d::bounding_volume::AABB::new(
                    na::Point2::from(
                        inputdata.pos() - na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH],
                    ),
                    na::Point2::from(
                        inputdata.pos() + na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH],
                    ),
                );
                new_bounds.merge(&pos_bounds);
            });
            self.bounds = Some(new_bounds);
        } else {
            self.bounds = None;
        }
    }

    pub fn gen_svg_path(&self, offset: na::Vector2<f64>) -> Result<String, anyhow::Error> {
        let mut svg = String::new();
        let mut data = element::path::Data::new();

        match self.style {
            SelectorStyle::Polygon => {
                for (i, element) in self.path.iter().enumerate() {
                    if i == 0 {
                        data = data
                            .move_to((element.pos()[0] + offset[0], element.pos()[1] + offset[1]));
                    } else {
                        data = data
                            .line_to((element.pos()[0] + offset[0], element.pos()[1] + offset[1]));
                    }
                }
            }
            SelectorStyle::Rectangle => {
                if let (Some(first), Some(last)) = (self.path.first(), self.path.last()) {
                    data = data.move_to((first.pos()[0] + offset[0], first.pos()[1] + offset[1]));
                    data = data.line_to((last.pos()[0] + offset[0], first.pos()[1] + offset[1]));
                    data = data.line_to((last.pos()[0] + offset[0], last.pos()[1] + offset[1]));
                    data = data.line_to((first.pos()[0] + offset[0], last.pos()[1] + offset[1]));
                }
            }
        }
        data = data.close();

        let svg_path = element::Path::new()
            .set("d", data)
            .set("stroke", Self::PATH_COLOR.to_css_color())
            .set("stroke-width", Self::PATH_WIDTH)
            .set("stroke-dasharray", "4 6")
            .set("fill", Self::FILL_COLOR.to_css_color());

        svg += rough_rs::node_to_string(&svg_path)
            .map_err(|e| {
                anyhow::anyhow!(
                    "rough_rs::node_to_string failed in gen_svg_path() for selector, {}",
                    e
                )
            })?
            .as_str();

        Ok(svg)
    }

    pub fn draw(&self, snapshot: &Snapshot) {
        if self.shown {
            snapshot.append_node(&self.rendernode);
        }
    }
}
