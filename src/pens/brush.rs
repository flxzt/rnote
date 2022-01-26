use std::collections::VecDeque;

use crate::compose::color::Color;
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
        let mut brush = Self {
            style: BrushStyle::default(),
            smooth_options: SmoothOptions::default(),
            textured_options: TexturedOptions::default(),
            current_stroke: None,
        };
        brush.set_width(Self::WIDTH_DEFAULT);

        brush
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
        if let Some(current_stroke_key) = appwindow.canvas().pens().borrow().brush.current_stroke {
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
            .brush
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

// specifying shared behaviour of all options
impl Brush {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 3.6;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;

    /// Gets the width for the current brush style
    pub fn width(&self) -> f64 {
        match self.style {
            BrushStyle::Solid => self.smooth_options.width(),
            BrushStyle::Textured => self.textured_options.width(),
        }
    }

    /// Sets the width for the current brush style
    pub fn set_width(&mut self, width: f64) {
        let width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);

        match self.style {
            BrushStyle::Solid => self.smooth_options.set_width(width),
            BrushStyle::Textured => self.textured_options.set_width(width),
        }
    }

    /// Gets the color for the current brush style
    pub fn color(&self) -> Option<Color> {
        match self.style {
            BrushStyle::Solid => self.smooth_options.color(),
            BrushStyle::Textured => self.textured_options.color(),
        }
    }

    /// Sets the color for the current brush style
    pub fn set_color(&mut self, color: Option<Color>) {
        match self.style {
            BrushStyle::Solid => self.smooth_options.set_color(color),
            BrushStyle::Textured => self.textured_options.set_color(color),
        }
    }
}
