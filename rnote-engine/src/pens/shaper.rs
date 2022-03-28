use crate::render::Renderer;
use crate::strokes::inputdata::InputData;
use crate::utils;
use gtk4::glib;
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use std::sync::{Arc, RwLock};

use crate::compose::rough::roughoptions::RoughOptions;
use crate::compose::smooth::SmoothOptions;
use crate::sheet::Sheet;
use crate::strokes::element::Element;
use crate::strokes::shapestroke::ShapeStroke;
use crate::strokes::strokestyle::StrokeStyle;
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, glib::Enum)]
#[enum_type(name = "ShaperConstraintRatio")]
#[serde(rename = "shaper_constraint_ratio")]
pub enum ShaperConstraintRatio {
    #[enum_value(name = "Disabled", nick = "disabled")]
    #[serde(rename = "disabled")]
    Disabled,
    #[enum_value(name = "1:1", nick = "one_to_one")]
    #[serde(rename = "one_to_one")]
    OneToOne,
    #[enum_value(name = "3:2", nick = "three_to_two")]
    #[serde(rename = "three_to_two")]
    ThreeToTwo,
    #[enum_value(name = "Golden ratio", nick = "golden")]
    #[serde(rename = "golden")]
    Golden,
}

impl From<glib::GString> for ShaperConstraintRatio {
    fn from(nick: glib::GString) -> Self {
        match nick.to_string().as_str() {
            "disabled" => ShaperConstraintRatio::Disabled,
            "one_to_one" => ShaperConstraintRatio::OneToOne,
            "three_to_two" => ShaperConstraintRatio::ThreeToTwo,
            "golden" => ShaperConstraintRatio::Golden,
            _ => unreachable!(),
        }
    }
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
    #[serde(skip)]
    pub ratio: ShaperConstraintRatio,
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
            ratio: ShaperConstraintRatio::Disabled,
        }
    }
}

impl PenBehaviour for Shaper {
    fn begin(
        &mut self,
        mut data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        _viewport: Option<AABB>,
        _zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) {
        self.current_stroke = None;

        let filter_bounds = sheet.bounds().loosened(utils::INPUT_OVERSHOOT);

        utils::filter_mapped_inputdata(filter_bounds, &mut data_entries);

        if let Some(inputdata) = data_entries.pop_back() {
            let element = Element::new(inputdata);

            let shapestroke = StrokeStyle::ShapeStroke(ShapeStroke::new(element, self));
            self.rect_start = element.inputdata.pos();
            self.rect_current = element.inputdata.pos();

            let current_stroke_key = Some(sheet.strokes_state.insert_stroke(shapestroke));
            self.current_stroke = current_stroke_key;
        }
    }

    fn motion(
        &mut self,
        mut data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        _viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) {
        let current_stroke_key = self.current_stroke;
        if let Some(current_stroke_key) = current_stroke_key {
            let filter_bounds = sheet.bounds().loosened(utils::INPUT_OVERSHOOT);

            utils::filter_mapped_inputdata(filter_bounds, &mut data_entries);

            for inputdata in data_entries {
                sheet.strokes_state.add_to_shapestroke(
                    current_stroke_key,
                    self,
                    Element::new(inputdata),
                    renderer.clone(),
                    zoom,
                );
            }
        }
    }

    fn end(
        &mut self,
        data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        _viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) {
        let current_stroke_key = self.current_stroke.take();
        if let Some(current_stroke_key) = current_stroke_key {
            sheet
                .strokes_state
                .update_geometry_for_stroke(current_stroke_key);

            for inputdata in data_entries {
                sheet.strokes_state.add_to_shapestroke(
                    current_stroke_key,
                    self,
                    Element::new(inputdata),
                    renderer.clone(),
                    zoom,
                );
            }
        }
    }
}
