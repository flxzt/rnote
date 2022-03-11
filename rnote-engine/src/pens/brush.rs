use std::collections::VecDeque;

use crate::compose::smooth::SmoothOptions;
use crate::compose::textured::TexturedOptions;
use crate::render::Renderer;
use crate::sheet::Sheet;
use crate::strokes::brushstroke::BrushStroke;
use crate::strokes::element::Element;
use crate::strokes::inputdata::InputData;
use crate::strokes::strokestyle::StrokeStyle;
use crate::strokesstate::StrokeKey;
use crate::utils;

use gtk4::glib;
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use super::penbehaviour::PenBehaviour;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, glib::Enum)]
#[repr(u32)]
#[enum_type(name = "BrushStyle")]
#[serde(rename = "brushstyle")]
pub enum BrushStyle {
    #[enum_value(name = "Marker", nick = "marker")]
    #[serde(rename = "marker")]
    Marker,
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

        let elements_iter = data_entries
            .into_iter()
            .map(|inputdata| Element::new(inputdata));

        let brushstroke = BrushStroke::new_w_elements(elements_iter, &self);

        if let Some(brushstroke) = brushstroke {
            let brushstroke = StrokeStyle::BrushStroke(brushstroke);

            let current_stroke_key = Some(sheet.strokes_state.insert_stroke(brushstroke));
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
                sheet.strokes_state.add_to_brushstroke(
                    current_stroke_key,
                    Element::new(inputdata),
                    renderer.clone(),
                    zoom,
                );
            }
        }
    }

    fn end(
        &mut self,
        _data_entries: VecDeque<InputData>,
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

            sheet
                .strokes_state
                .regenerate_rendering_for_stroke_threaded(current_stroke_key, renderer, zoom);
        }
    }
}
