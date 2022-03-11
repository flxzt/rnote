use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::compose::{self, color::Color};
use crate::render::{self, Renderer};
use crate::sheet::Sheet;
use crate::strokes::inputdata::InputData;

use anyhow::Context;
use gtk4::{glib, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};
use svg::node::element;

use super::penbehaviour::PenBehaviour;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, glib::Enum)]
#[serde(rename = "selector_style")]
#[enum_type(name = "SelectorStyle")]
pub enum SelectorStyle {
    #[serde(rename = "polygon")]
    #[enum_value(name = "Polygon", nick = "polygon")]
    Polygon,
    #[serde(rename = "rectangle")]
    #[enum_value(name = "Rectangle", nick = "rectangle")]
    Rectangle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "selector")]
pub struct Selector {
    #[serde(rename = "style")]
    pub style: SelectorStyle,
    #[serde(skip)]
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
    fn begin(
        &mut self,
        mut data_entries: VecDeque<InputData>,
        _sheet: &mut Sheet,
        _viewport: Option<AABB>,
        _zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) {
        self.path.clear();

        if let Some(inputdata) = data_entries.pop_back() {
            self.path.push(inputdata);
        }
    }

    fn motion(
        &mut self,
        mut data_entries: VecDeque<InputData>,
        _sheet: &mut Sheet,
        _viewport: Option<AABB>,
        _zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) {
        if let Some(inputdata) = data_entries.pop_back() {
            let style = self.style;

            match style {
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
    }

    fn end(
        &mut self,
        _data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) {
        sheet
            .strokes_state
            .update_selection_for_selector(&self, viewport);

        let selection_keys = sheet.strokes_state.selection_keys_as_rendered();
        sheet
            .strokes_state
            .regenerate_rendering_for_strokes(&selection_keys, renderer, zoom);

        self.path.clear();
    }

    fn draw(
        &self,
        snapshot: &Snapshot,
        _sheet: &Sheet,
        _viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        if let Some(bounds) = self.gen_bounds() {
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

            let svg_data = compose::svg_node_to_string(&svg_path).map_err(|e| {
                anyhow::anyhow!(
                    "node_to_string() failed in gen_svg_path() for selector, {}",
                    e
                )
            })?;

            let svg = render::Svg { bounds, svg_data };
            let images = renderer
                .read()
                .unwrap()
                .gen_images(zoom, vec![svg], bounds)?;
            if let Some(rendernode) = render::images_to_rendernode(&images, zoom)
                .context("images_to_rendernode() failed in selector.draw()")?
            {
                snapshot.append_node(&rendernode);
            }
        }
        Ok(())
    }
}

impl Selector {
    pub const PATH_WIDTH: f64 = 1.5;
    pub const PATH_COLOR: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 0.7,
    };
    pub const FILL_COLOR: Color = Color {
        r: 0.7,
        g: 0.7,
        b: 0.7,
        a: 0.15,
    };

    pub fn gen_bounds(&self) -> Option<AABB> {
        // Making sure bounds are always outside of coord + width
        let mut path_iter = self.path.iter();
        if let Some(first) = path_iter.next() {
            let mut new_bounds = AABB::new(
                na::Point2::from(first.pos() - na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH]),
                na::Point2::from(first.pos() + na::vector![Self::PATH_WIDTH, Self::PATH_WIDTH]),
            );

            path_iter.for_each(|inputdata| {
                let pos_bounds = AABB::new(
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
