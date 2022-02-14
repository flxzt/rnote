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
        Self {
            options: SmoothOptions::default(),
            current_stroke: None,
        }
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
        let current_stroke_key = appwindow.canvas().pens().borrow().marker.current_stroke;
        if let Some(current_stroke_key) = current_stroke_key {
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
                        &mut appwindow.canvas().pens().borrow_mut(),
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

        let current_stroke_key = appwindow
            .canvas()
            .pens()
            .borrow_mut()
            .marker
            .current_stroke
            .take();
        if let Some(current_stroke_key) = current_stroke_key {
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

            appwindow.canvas().update_size_autoexpand();
        }
    }
}
