use std::collections::VecDeque;

use crate::compose::geometry;
use crate::strokes::strokestyle::InputData;
use crate::ui::appwindow::RnoteAppWindow;
use crate::utils;

use gtk4::{gdk, graphene, gsk, prelude::*, Snapshot};

use super::penbehaviour::PenBehaviour;

#[derive(Clone, Debug)]
pub struct Eraser {
    width: f64,
    current_input: Option<InputData>,
}

impl Default for Eraser {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            current_input: None,
        }
    }
}

impl PenBehaviour for Eraser {
    fn begin(&mut self, mut data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        appwindow
            .canvas()
            .set_cursor(gdk::Cursor::from_name("none", None).as_ref());

        self.current_input = data_entries.pop_back();
    }

    fn motion(&mut self, mut data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        self.current_input = data_entries.pop_back();

        appwindow
            .canvas()
            .sheet()
            .strokes_state()
            .borrow_mut()
            .trash_colliding_strokes(self, Some(appwindow.canvas().viewport_in_sheet_coords()));

        if appwindow.canvas().sheet().resize_endless() {
            appwindow.canvas().update_background_rendernode(false);
        }
    }

    fn end(&mut self, _data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().cursor()));

        self.current_input = None;
    }

    fn draw(
        &self,
        _sheet_bounds: p2d::bounding_volume::AABB,
        _renderer: &crate::render::Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        if let Some(bounds) = self.gen_bounds(zoom) {
            let border_color = Self::OUTLINE_COLOR.to_gdk();
            let border_width = 2.0;

            snapshot.append_color(
                &Self::FILL_COLOR.to_gdk(),
                &geometry::aabb_to_graphene_rect(bounds),
            );

            snapshot.append_border(
                &gsk::RoundedRect::new(
                    geometry::aabb_to_graphene_rect(bounds),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                ),
                &[border_width, border_width, border_width, border_width],
                &[border_color, border_color, border_color, border_color],
            );
        }
        Ok(())
    }
}

impl Eraser {
    pub const OUTLINE_COLOR: utils::Color = utils::Color {
        r: 0.8,
        g: 0.1,
        b: 0.0,
        a: 0.5,
    };
    pub const FILL_COLOR: utils::Color = utils::Color {
        r: 0.7,
        g: 0.2,
        b: 0.1,
        a: 0.5,
    };
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 30.0;

    pub fn new(width: f64) -> Self {
        Self {
            width,
            current_input: None,
        }
    }

    pub fn current_input(&self) -> Option<InputData> {
        self.current_input
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }

    pub fn gen_bounds(&self, zoom: f64) -> Option<p2d::bounding_volume::AABB> {
        self.current_input.map_or_else(
            || None,
            |current_input| {
                Some(p2d::bounding_volume::AABB::new(
                    na::point![
                        ((current_input.pos()[0]) - self.width / 2.0) * zoom,
                        ((current_input.pos()[1]) - self.width / 2.0) * zoom
                    ],
                    na::point![
                        ((current_input.pos()[0]) + self.width / 2.0) * zoom,
                        ((current_input.pos()[1]) + self.width / 2.0) * zoom
                    ],
                ))
            },
        )
    }
}
