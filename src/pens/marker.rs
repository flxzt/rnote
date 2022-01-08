use gtk4::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::strokes::markerstroke::MarkerStroke;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokes::StrokeKey;
use crate::{input, utils};

use super::penbehaviour::PenBehaviour;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Marker {
    width: f64,
    pub color: utils::Color,
    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
}

impl Default for Marker {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            color: Self::COLOR_DEFAULT,
            current_stroke: None,
        }
    }
}

impl PenBehaviour for Marker {
    fn begin(
        &mut self,
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        self.current_stroke = None;
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().motion_cursor()));

        let filter_bounds = p2d::bounding_volume::AABB::new(
            na::point![-input::INPUT_OVERSHOOT, -input::INPUT_OVERSHOOT],
            na::point![
                (appwindow.canvas().sheet().width()) as f64 + input::INPUT_OVERSHOOT,
                (appwindow.canvas().sheet().height()) as f64 + input::INPUT_OVERSHOOT
            ],
        );
        input::filter_mapped_inputdata(filter_bounds, &mut data_entries);

        if let Some(inputdata) = data_entries.pop_back() {
            let element = Element::new(inputdata);
            let markerstroke = StrokeStyle::MarkerStroke(MarkerStroke::new(element, self.clone()));

            self.current_stroke = Some(
                appwindow
                    .canvas()
                    .sheet()
                    .strokes_state()
                    .borrow_mut()
                    .insert_stroke(markerstroke),
            );
        }
    }

    fn motion(
        &mut self,
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        if let Some(current_stroke_key) = self.current_stroke {
            let filter_bounds = p2d::bounding_volume::AABB::new(
                na::point![-input::INPUT_OVERSHOOT, -input::INPUT_OVERSHOOT],
                na::point![
                    (appwindow.canvas().sheet().width()) as f64 + input::INPUT_OVERSHOOT,
                    (appwindow.canvas().sheet().height()) as f64 + input::INPUT_OVERSHOOT
                ],
            );
            input::filter_mapped_inputdata(filter_bounds, &mut data_entries);

            for inputdata in data_entries {
                appwindow
                    .canvas()
                    .sheet()
                    .strokes_state()
                    .borrow_mut()
                    .add_to_stroke(current_stroke_key, Element::new(inputdata));
            }
        }
    }

    fn end(
        &mut self,
        _data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().cursor()));

        if let Some(current_stroke) = self.current_stroke.take() {
            appwindow
                .canvas()
                .sheet()
                .strokes_state()
                .borrow_mut()
                .update_geometry_for_stroke(current_stroke);

            appwindow
                .canvas()
                .sheet()
                .strokes_state()
                .borrow_mut()
                .regenerate_rendering_for_stroke_threaded(current_stroke);
        }
    }
}

impl Marker {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 2.0;

    pub const COLOR_DEFAULT: utils::Color = utils::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }
}
