use gtk4::prelude::*;
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::compose::rough::roughoptions;
use crate::strokes::shapestroke::ShapeStroke;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokesstate::StrokeKey;
use crate::{compose, input};

use super::penbehaviour::PenBehaviour;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "shapestyle")]
pub enum ShapeStyle {
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "rectangle")]
    Rectangle,
    #[serde(rename = "ellipse")]
    Ellipse,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self::Rectangle
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "drawstyle")]
pub enum DrawStyle {
    #[serde(rename = "smooth")]
    Smooth,
    #[serde(rename = "rough")]
    Rough,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

impl DrawStyle {
    pub const ROUGH_MARGIN: f64 = 20.0;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shaper")]
pub struct Shaper {
    #[serde(rename = "shapestyle")]
    shapestyle: ShapeStyle,
    #[serde(rename = "drawstyle")]
    drawstyle: DrawStyle,
    #[serde(rename = "width")]
    width: f64,
    #[serde(rename = "color")]
    color: Option<compose::Color>,
    #[serde(rename = "fill")]
    fill: Option<compose::Color>,
    #[serde(rename = "rough_config")]
    pub rough_config: roughoptions::Options,
    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            shapestyle: ShapeStyle::default(),
            drawstyle: DrawStyle::default(),
            width: Shaper::WIDTH_DEFAULT,
            color: Shaper::COLOR_DEFAULT,
            fill: Shaper::FILL_DEFAULT,
            rough_config: roughoptions::Options::default(),
            current_stroke: None,
        }
    }
}

impl PenBehaviour for Shaper {
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
            let shapestroke = StrokeStyle::ShapeStroke(ShapeStroke::new(element, self.clone()));

            self.current_stroke = Some(
                appwindow
                    .canvas()
                    .sheet()
                    .strokes_state()
                    .borrow_mut()
                    .insert_stroke(shapestroke),
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

impl Shaper {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 2.0;

    pub const COLOR_DEFAULT: Option<compose::Color> = Some(compose::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    pub const FILL_DEFAULT: Option<compose::Color> = None;

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Shaper::WIDTH_MIN, Shaper::WIDTH_MAX);
    }

    pub fn shapestyle(&self) -> ShapeStyle {
        self.shapestyle
    }

    pub fn set_shapestyle(&mut self, shapestyle: ShapeStyle) {
        self.shapestyle = shapestyle;
    }

    pub fn drawstyle(&self) -> DrawStyle {
        self.drawstyle
    }

    pub fn set_drawstyle(&mut self, drawstyle: DrawStyle) {
        self.drawstyle = drawstyle;
    }

    pub fn color(&self) -> Option<compose::Color> {
        self.color
    }

    pub fn set_color(&mut self, color: Option<compose::Color>) {
        self.color = color;
    }

    pub fn fill(&self) -> Option<compose::Color> {
        self.fill
    }

    pub fn set_fill(&mut self, fill: Option<compose::Color>) {
        self.fill = fill;
    }
}
