use std::collections::VecDeque;

use crate::compose::smooth::SmoothOptions;
use crate::compose::textured::TexturedOptions;
use crate::input;
use crate::strokes::brushstroke::BrushStroke;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokesstate::StrokeKey;

use gtk4::{glib, prelude::*};
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, glib::Enum)]
#[repr(u32)]
#[enum_type(name = "BrushStyle")]
#[serde(rename = "brushstyle")]
pub enum BrushStyle {
    #[enum_value(name = "Solid", nick = "solid")]
    #[serde(rename = "solid")]
    Solid,
    #[enum_value(name = "Textured", nick = "textured")]
    #[serde(rename = "textured")]
    Textured,
}

impl Default for BrushStyle {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "brush")]
pub struct Brush {
    #[serde(rename = "style")]
    pub style: BrushStyle,
    #[serde(rename = "smooth_options")]
    pub smooth_options: SmoothOptions,
    #[serde(rename = "textured_options")]
    pub textured_options: TexturedOptions,

    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            style: BrushStyle::default(),
            smooth_options: SmoothOptions::default(),
            textured_options: TexturedOptions::default(),
            current_stroke: None,
        }
    }
}

impl PenBehaviour for Brush {
    fn begin(
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        appwindow.canvas().pens().borrow_mut().brush.current_stroke = None;
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

        let brushstroke =
            BrushStroke::new_w_elements(elements_iter, &appwindow.canvas().pens().borrow().brush);

        if let Some(brushstroke) = brushstroke {
            let brushstroke = StrokeStyle::BrushStroke(brushstroke);

            let current_stroke_key = Some(
                appwindow
                    .canvas()
                    .sheet()
                    .borrow_mut()
                    .strokes_state
                    .insert_stroke(brushstroke),
            );
            appwindow.canvas().pens().borrow_mut().brush.current_stroke = current_stroke_key;
        }
    }

    fn motion(
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        let current_stroke_key = appwindow.canvas().pens().borrow().brush.current_stroke;
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
            .brush
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

            appwindow.canvas().resize_sheet_autoexpand();
            appwindow.canvas().update_background_rendernode(true);
        }
    }
}
