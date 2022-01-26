use gtk4::{glib, prelude::*};
use p2d::bounding_volume::BoundingVolume;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::compose::color::Color;
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
        Self::Rectangle
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
    pub const ROUGH_MARGIN: f64 = 20.0;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shaper")]
pub struct Shaper {
    #[serde(rename = "shapestyle")]
    shaperstyle: ShaperStyle,
    #[serde(rename = "drawstyle")]
    drawstyle: ShaperDrawStyle,
    #[serde(rename = "smooth_options")]
    pub smooth_options: SmoothOptions,
    #[serde(rename = "rough_options")]
    pub rough_options: RoughOptions,

    #[serde(skip)]
    pub current_stroke: Option<StrokeKey>,
}

impl Default for Shaper {
    fn default() -> Self {
        let mut shaper = Self {
            shaperstyle: ShaperStyle::default(),
            drawstyle: ShaperDrawStyle::default(),
            smooth_options: SmoothOptions::default(),
            rough_options: RoughOptions::default(),
            current_stroke: None,
        };
        shaper.set_width(Self::WIDTH_DEFAULT);

        shaper
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
        if let Some(current_stroke_key) = appwindow.canvas().pens().borrow().shaper.current_stroke {
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
            .shaper
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

impl Shaper {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 2.0;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;

    pub fn shaperstyle(&self) -> ShaperStyle {
        self.shaperstyle
    }

    pub fn set_shaperstyle(&mut self, shapestyle: ShaperStyle) {
        self.shaperstyle = shapestyle;
    }

    pub fn drawstyle(&self) -> ShaperDrawStyle {
        self.drawstyle
    }

    pub fn set_drawstyle(&mut self, drawstyle: ShaperDrawStyle) {
        self.drawstyle = drawstyle;
    }

    /// Gets the width for the current shaper drawstyle
    pub fn width(&self) -> f64 {
        match self.drawstyle {
            ShaperDrawStyle::Smooth => self.smooth_options.width(),
            ShaperDrawStyle::Rough => self.rough_options.stroke_width(),
        }
    }

    /// Sets the width for the current shaper drawstyle
    pub fn set_width(&mut self, width: f64) {
        match self.drawstyle {
            ShaperDrawStyle::Smooth => self.smooth_options.set_width(width),
            ShaperDrawStyle::Rough => self.rough_options.set_stroke_width(width),
        }
    }

    /// Gets the color for the current shaper drawstyle
    pub fn color(&self) -> Option<Color> {
        match self.drawstyle {
            ShaperDrawStyle::Smooth => self.smooth_options.color(),
            ShaperDrawStyle::Rough => self.rough_options.stroke,
        }
    }

    /// Sets the color for the current shaper drawstyle
    pub fn set_color(&mut self, color: Option<Color>) {
        match self.drawstyle {
            ShaperDrawStyle::Smooth => self.smooth_options.set_color(color),
            ShaperDrawStyle::Rough => self.rough_options.stroke = color,
        }
    }

    /// Gets the fill color for the current shaper drawstyle
    pub fn fill(&self) -> Option<Color> {
        match self.drawstyle {
            ShaperDrawStyle::Smooth => self.smooth_options.fill(),
            ShaperDrawStyle::Rough => self.rough_options.fill,
        }
    }

    /// Sets the fill color for the current shaper drawstyle
    pub fn set_fill(&mut self, fill: Option<Color>) {
        match self.drawstyle {
            ShaperDrawStyle::Smooth => self.smooth_options.set_fill(fill),
            ShaperDrawStyle::Rough => self.rough_options.stroke = fill,
        }
    }
}
