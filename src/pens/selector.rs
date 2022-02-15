use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::compose::color::Color;
use crate::strokes::strokestyle::InputData;
use crate::ui::appwindow::RnoteAppWindow;
use crate::{compose, render};

use anyhow::Context;
use gtk4::{gdk, glib, prelude::*, Snapshot};
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
    fn begin(mut data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        appwindow
            .canvas()
            .set_cursor(gdk::Cursor::from_name("cell", None).as_ref());

        appwindow.canvas().pens().borrow_mut().selector.path.clear();

        if let Some(inputdata) = data_entries.pop_back() {
            appwindow
                .canvas()
                .pens()
                .borrow_mut()
                .selector
                .path
                .push(inputdata);
        }
    }

    fn motion(mut data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        if let Some(inputdata) = data_entries.pop_back() {
            let style = appwindow.canvas().pens().borrow().selector.style;

            match style {
                SelectorStyle::Polygon => {
                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .selector
                        .path
                        .push(inputdata);
                }
                SelectorStyle::Rectangle => {
                    if appwindow.canvas().pens().borrow().selector.path.len() > 2 {
                        appwindow
                            .canvas()
                            .pens()
                            .borrow_mut()
                            .selector
                            .path
                            .resize(2, InputData::default());
                    }
                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .selector
                        .path
                        .insert(1, inputdata)
                }
            }
        }
    }

    fn end(_data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().cursor()));

        appwindow
            .canvas()
            .sheet()
            .borrow_mut()
            .strokes_state
            .update_selection_for_selector(
                &appwindow.canvas().pens().borrow().selector,
                Some(appwindow.canvas().viewport_in_sheet_coords()),
            );
        let selection_keys = appwindow
            .canvas()
            .sheet()
            .borrow()
            .strokes_state
            .selection_keys_in_order_rendered();
        appwindow
            .canvas()
            .sheet()
            .borrow_mut()
            .strokes_state
            .regenerate_rendering_for_strokes(
                &selection_keys,
                appwindow.canvas().renderer(),
                appwindow.canvas().zoom(),
            );

        appwindow.canvas().pens().borrow_mut().selector.path.clear();
    }

    fn draw(
        &self,
        _sheet_bounds: AABB,
        zoom: f64,
        snapshot: &Snapshot,
        renderer: Arc<RwLock<render::Renderer>>,
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
            let images = renderer.read().unwrap().gen_images(zoom, vec![svg], bounds)?;
            if let Some(rendernode) =
                render::images_to_rendernode(&images, zoom).context("images_to_rendernode() failed in selector.draw()")?
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
