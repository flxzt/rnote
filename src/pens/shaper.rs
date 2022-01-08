use gtk4::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::strokes::shapestroke::ShapeStroke;
use crate::strokes::strokestyle::{Element, StrokeStyle};
use crate::strokes::StrokeKey;
use crate::{input, utils};

use super::penbehaviour::PenBehaviour;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CurrentShape {
    Line,
    Rectangle,
    Ellipse,
}

impl Default for CurrentShape {
    fn default() -> Self {
        Self::Rectangle
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DrawStyle {
    Smooth,
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
#[serde(default)]
pub struct Shaper {
    pub current_shape: CurrentShape,
    pub drawstyle: DrawStyle,
    width: f64,
    color: Option<utils::Color>,
    fill: Option<utils::Color>,
    pub roughconfig: rough_rs::options::Options,
    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            current_shape: CurrentShape::default(),
            drawstyle: DrawStyle::default(),
            width: Shaper::WIDTH_DEFAULT,
            color: Shaper::COLOR_DEFAULT,
            fill: Shaper::FILL_DEFAULT,
            roughconfig: rough_rs::options::Options::default(),
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
            let shapestroke = StrokeStyle::ShapeStroke(ShapeStroke::new(element, self.clone()));

            self.current_stroke = Some(
                appwindow
                    .canvas()
                    .sheet()
                    .strokes_state()
                    .borrow_mut()
                    .insert_stroke(shapestroke),
            );

            appwindow
                .canvas()
                .sheet()
                .strokes_state()
                .borrow_mut()
                .regenerate_rendering_newest_stroke_threaded();
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

impl Shaper {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 2.0;

    pub const COLOR_DEFAULT: Option<utils::Color> = Some(utils::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    pub const FILL_DEFAULT: Option<utils::Color> = None;

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Shaper::WIDTH_MIN, Shaper::WIDTH_MAX);
    }

    pub fn color(&self) -> Option<utils::Color> {
        self.color
    }

    pub fn set_color(&mut self, color: Option<utils::Color>) {
        self.color = color;
    }

    pub fn fill(&self) -> Option<utils::Color> {
        self.fill
    }

    pub fn set_fill(&mut self, fill: Option<utils::Color>) {
        self.fill = fill;
    }

    pub fn apply_roughconfig_onto(&self, options: &mut rough_rs::options::Options) {
        options.roughness = self.roughconfig.roughness();
        options.bowing = self.roughconfig.bowing();
    }
}
