use std::collections::VecDeque;

use crate::compose::{color::Color, textured};
use crate::input;
use crate::strokes::brushstroke::BrushStroke;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokesstate::StrokeKey;

use gtk4::prelude::*;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "brushstyle")]
pub enum BrushStyle {
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "textured")]
    Textured,
    #[serde(rename = "experimental")]
    Experimental,
}

impl Default for BrushStyle {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "brush")]
pub struct Brush {
    #[serde(rename = "width")]
    width: f64,
    #[serde(rename = "sensitivity")]
    sensitivity: f64,
    #[serde(rename = "color")]
    color: Color,
    #[serde(rename = "style")]
    style: BrushStyle,
    #[serde(rename = "textured_config")]
    pub textured_config: textured::TexturedOptions,
    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            sensitivity: Self::SENSITIVITY_DEFAULT,
            color: Color::BLACK,
            style: BrushStyle::default(),
            textured_config: textured::TexturedOptions::default(),
            current_stroke: None,
        }
    }
}

impl PenBehaviour for Brush {
    fn begin(
        &mut self,
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        self.current_stroke = None;
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().motion_cursor()));

        let filter_bounds = appwindow
            .canvas()
            .sheet()
            .bounds()
            .loosened(input::INPUT_OVERSHOOT);

        input::filter_mapped_inputdata(filter_bounds, &mut data_entries);

        if let Some(inputdata) = data_entries.pop_back() {
            let element = Element::new(inputdata);
            let brushstroke = StrokeStyle::BrushStroke(BrushStroke::new(element, self.clone()));

            self.current_stroke = Some(
                appwindow
                    .canvas()
                    .sheet()
                    .strokes_state()
                    .borrow_mut()
                    .insert_stroke(brushstroke),
            );
        }
    }

    fn motion(
        &mut self,
        mut data_entries: VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        if let Some(current_stroke_key) = self.current_stroke {
            let filter_bounds = appwindow
                .canvas()
                .sheet()
                .bounds()
                .loosened(input::INPUT_OVERSHOOT);

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

impl Brush {
    pub const WIDTH_MIN: f64 = 0.1;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 6.0;
    pub const SENSITIVITY_MIN: f64 = 0.0;
    pub const SENSITIVITY_MAX: f64 = 1.0;
    pub const SENSITIVITY_DEFAULT: f64 = 0.5;

    pub const TEMPLATE_BOUNDS_PADDING: f64 = 50.0;

    pub const COLOR_DEFAULT: Color = Color {
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

    pub fn sensitivity(&self) -> f64 {
        self.sensitivity
    }

    pub fn set_sensitivity(&mut self, sensitivity: f64) {
        self.sensitivity = sensitivity.clamp(Self::SENSITIVITY_MIN, Self::SENSITIVITY_MAX);
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn style(&self) -> BrushStyle {
        self.style
    }

    pub fn set_style(&mut self, style: BrushStyle) {
        self.style = style;
    }
}
