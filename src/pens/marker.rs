use gtk4::prelude::*;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::compose::smooth::SmoothOptions;
use crate::input;
use crate::strokes::markerstroke::MarkerStroke;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokesstate::StrokeKey;

use super::penbehaviour::PenBehaviour;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "marker")]
pub struct Marker {
    #[serde(rename = "options")]
    pub options: SmoothOptions,

    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
}

impl Default for Marker {
    fn default() -> Self {
        let mut marker = Self {
            options: SmoothOptions::default(),
            current_stroke: None,
        };
        marker.set_width(Self::WIDTH_DEFAULT);

        marker
    }
}

impl PenBehaviour for Marker {
    fn begin(
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        appwindow.canvas().pens().borrow_mut().marker.current_stroke = None;
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().motion_cursor()));

        let filter_bounds = appwindow
            .canvas()
            .sheet()
            .borrow()
            .bounds()
            .loosened(input::INPUT_OVERSHOOT);

        input::filter_mapped_inputdata(filter_bounds, &mut data_entries);

        let elements_iter = data_entries
            .into_iter()
            .map(|inputdata| Element::new(inputdata));

        let markerstroke =
            MarkerStroke::new_w_elements(elements_iter, &appwindow.canvas().pens().borrow().marker);

        if let Some(markerstroke) = markerstroke {
            let markerstroke = StrokeStyle::MarkerStroke(markerstroke);

            let current_stroke_key = Some(
                appwindow
                    .canvas()
                    .sheet()
                    .borrow_mut()
                    .strokes_state
                    .insert_stroke(markerstroke),
            );
            appwindow.canvas().pens().borrow_mut().marker.current_stroke = current_stroke_key;
        }
    }

    fn motion(
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        if let Some(current_stroke_key) = appwindow.canvas().pens().borrow().marker.current_stroke {
            let filter_bounds = appwindow
                .canvas()
                .sheet()
                .borrow()
                .bounds()
                .loosened(input::INPUT_OVERSHOOT);

            input::filter_mapped_inputdata(filter_bounds, &mut data_entries);

            for inputdata in data_entries {
                appwindow
                    .canvas()
                    .sheet()
                    .borrow_mut()
                    .strokes_state
                    .add_to_stroke(
                        current_stroke_key,
                        Element::new(inputdata),
                        appwindow.canvas().renderer(),
                        appwindow.canvas().zoom(),
                    );
            }
        }
    }

    fn end(
        _data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().cursor()));

        if let Some(current_stroke_key) = appwindow
            .canvas()
            .pens()
            .borrow_mut()
            .marker
            .current_stroke
            .take()
        {
            appwindow
                .canvas()
                .sheet()
                .borrow_mut()
                .strokes_state
                .update_geometry_for_stroke(current_stroke_key);

            appwindow
                .canvas()
                .sheet()
                .borrow_mut()
                .strokes_state
                .regenerate_rendering_for_stroke_threaded(
                    current_stroke_key,
                    appwindow.canvas().renderer(),
                    appwindow.canvas().zoom(),
                );

            appwindow.canvas().resize_endless();
        }
    }
}

impl Marker {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 6.0;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;

    pub fn width(&self) -> f64 {
        self.options.width()
    }

    pub fn set_width(&mut self, width: f64) {
        self.options
            .set_width(width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX));
    }
}
