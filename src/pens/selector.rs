use crate::strokes::strokestyle::InputData;
use crate::ui::appwindow::RnoteAppWindow;
use crate::{compose, render, utils};

use gtk4::Snapshot;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use svg::node::element;

use super::penbehaviour::PenBehaviour;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SelectorStyle {
    Polygon,
    Rectangle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Selector {
    style: SelectorStyle,
    pub path: Vec<InputData>,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            style: SelectorStyle::Polygon,
            path: vec![],
        }
    }
}

impl PenBehaviour for Selector {
    fn begin(&mut self, inputdata: InputData, _appwindow: &RnoteAppWindow) {
        self.path.clear();
        self.path.push(inputdata);
    }

    fn motion(&mut self, inputdata: InputData, _appwindow: &RnoteAppWindow) {
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
    }

    fn end(&mut self, _inputdata: InputData, appwindow: &RnoteAppWindow) {
        appwindow
            .canvas()
            .sheet()
            .strokes_state()
            .borrow_mut()
            .update_selection_for_selector(
                self,
                Some(appwindow.canvas().viewport_in_sheet_coords()),
            );

        self.path.clear();
    }

    fn draw(
        &self,
        _sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &render::Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        if let Some(bounds) = self.gen_bounds() {
            let mut svg_data = String::new();
            let mut data = element::path::Data::new();
            let offset = na::vector![0.0, 0.0];

            match self.style {
                SelectorStyle::Polygon => {
                    for (i, element) in self.path.iter().enumerate() {
                        if i == 0 {
                            data = data.move_to((
                                element.pos()[0] + offset[0],
                                element.pos()[1] + offset[1],
                            ));
                        } else {
                            data = data.line_to((
                                element.pos()[0] + offset[0],
                                element.pos()[1] + offset[1],
                            ));
                        }
                    }
                }
                SelectorStyle::Rectangle => {
                    if let (Some(first), Some(last)) = (self.path.first(), self.path.last()) {
                        data =
                            data.move_to((first.pos()[0] + offset[0], first.pos()[1] + offset[1]));
                        data =
                            data.line_to((last.pos()[0] + offset[0], first.pos()[1] + offset[1]));
                        data = data.line_to((last.pos()[0] + offset[0], last.pos()[1] + offset[1]));
                        data =
                            data.line_to((first.pos()[0] + offset[0], last.pos()[1] + offset[1]));
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

            svg_data += rough_rs::node_to_string(&svg_path)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "rough_rs::node_to_string failed in gen_svg_path() for selector, {}",
                        e
                    )
                })?
                .as_str();

            let svg = render::Svg {
                bounds,
                svg_data: compose::wrap_svg(svg_data.as_str(), None, Some(bounds), true, false),
            };
            let image = renderer.gen_image(zoom, &[svg], bounds)?;
            let rendernode = render::image_to_rendernode(&image, zoom);
            snapshot.append_node(&rendernode);
        }
        Ok(())
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
        a: 0.15,
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

    pub fn gen_bounds(&self) -> Option<p2d::bounding_volume::AABB> {
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
            Some(new_bounds)
        } else {
            None
        }
    }
}
