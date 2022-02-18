use gtk4::{glib, prelude::*};
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::compose::rough::roughoptions::RoughOptions;
use crate::compose::smooth::SmoothOptions;
use crate::input;
use crate::strokes::shapestroke::ShapeStroke;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokesstate::StrokeKey;

use super::penbehaviour::PenBehaviour;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, glib::Enum)]
#[serde(rename = "shaperstyle")]
#[enum_type(name = "ShaperStyle")]
pub enum ShaperStyle {
    #[serde(rename = "line")]
    #[enum_value(name = "Line", nick = "line")]
    Line,
    #[serde(rename = "rectangle")]
    #[enum_value(name = "Rectangle", nick = "rectangle")]
    Rectangle,
    #[serde(rename = "ellipse")]
    #[enum_value(name = "Ellipse", nick = "ellipse")]
    Ellipse,
}

impl Default for ShaperStyle {
    fn default() -> Self {
        Self::Line
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, glib::Enum)]
#[enum_type(name = "ShaperDrawStyle")]
#[serde(rename = "shaper_drawstyle")]
pub enum ShaperDrawStyle {
    #[enum_value(name = "Smooth", nick = "smooth")]
    #[serde(rename = "smooth")]
    Smooth,
    #[enum_value(name = "Rough", nick = "rough")]
    #[serde(rename = "rough")]
    Rough,
}

impl Default for ShaperDrawStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

impl ShaperDrawStyle {
    pub const SMOOTH_MARGIN: f64 = 1.0;
    pub const ROUGH_MARGIN: f64 = 20.0;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shaper")]
pub struct Shaper {
    #[serde(rename = "style")]
    pub style: ShaperStyle,
    #[serde(rename = "drawstyle")]
    pub drawstyle: ShaperDrawStyle,
    #[serde(rename = "smooth_options")]
    pub smooth_options: SmoothOptions,
    #[serde(rename = "rough_options")]
    pub rough_options: RoughOptions,

    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
    #[serde(skip)]
    pub rect_start: na::Vector2<f64>,
    #[serde(skip)]
    pub rect_current: na::Vector2<f64>,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            style: ShaperStyle::default(),
            drawstyle: ShaperDrawStyle::default(),
            smooth_options: SmoothOptions::default(),
            rough_options: RoughOptions::default(),
            current_stroke: None,
            rect_start: na::vector![0.0, 0.0],
            rect_current: na::vector![0.0, 0.0],
        }
    }
}

impl PenBehaviour for Shaper {
    fn begin(
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        appwindow.canvas().pens().borrow_mut().shaper.current_stroke = None;
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

        if let Some(inputdata) = data_entries.pop_back() {
            let element = Element::new(inputdata);

            let shapestroke = StrokeStyle::ShapeStroke(ShapeStroke::new(
                element,
                &appwindow.canvas().pens().borrow().shaper.clone(),
            ));
            appwindow.canvas().pens().borrow_mut().shaper.rect_start = element.inputdata.pos();
            appwindow.canvas().pens().borrow_mut().shaper.rect_current = element.inputdata.pos();

            let current_stroke_key = Some(
                appwindow
                    .canvas()
                    .sheet()
                    .borrow_mut()
                    .strokes_state
                    .insert_stroke(shapestroke),
            );
            appwindow.canvas().pens().borrow_mut().shaper.current_stroke = current_stroke_key;
        }
    }

    fn motion(
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        let current_stroke_key = appwindow.canvas().pens().borrow().shaper.current_stroke;
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

            appwindow.canvas().resize_sheet_autoexpand();
            appwindow.canvas().update_background_rendernode(true);
        }
    }
}
