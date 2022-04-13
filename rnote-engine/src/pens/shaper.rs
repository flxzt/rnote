use super::penbehaviour::PenBehaviour;
use super::AudioPlayer;
use crate::sheet::Sheet;
use crate::store::StrokeKey;
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{Camera, DrawOnSheetBehaviour, StrokeStore, SurfaceFlags};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rnote_compose::shapes::ShapeType;
use rnote_compose::style::rough::RoughOptions;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::PenEvent;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "shaper_style")]
pub enum ShaperStyle {
    #[serde(rename = "smooth")]
    Smooth,
    #[serde(rename = "rough")]
    Rough,
}

impl Default for ShaperStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "shaper")]
pub struct Shaper {
    #[serde(rename = "shape_type")]
    pub shape_type: ShapeType,
    #[serde(rename = "style")]
    pub style: ShaperStyle,
    #[serde(rename = "smooth_options")]
    pub smooth_options: SmoothOptions,
    #[serde(rename = "rough_options")]
    pub rough_options: RoughOptions,

    #[serde(skip)]
    current_stroke_key: Option<StrokeKey>,
    #[serde(skip)]
    pub rect_start: na::Vector2<f64>,
    #[serde(skip)]
    pub rect_current: na::Vector2<f64>,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            shape_type: ShapeType::default(),
            style: ShaperStyle::default(),
            smooth_options: SmoothOptions::default(),
            rough_options: RoughOptions::default(),
            current_stroke_key: None,
            rect_start: na::vector![0.0, 0.0],
            rect_current: na::vector![0.0, 0.0],
        }
    }
}

impl PenBehaviour for Shaper {
    fn handle_event(
        &mut self,
        event: PenEvent,
        sheet: &mut Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) -> SurfaceFlags {
        let surface_flags = SurfaceFlags::default();

        match (self.current_stroke_key, event) {
            (
                None,
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                if !element.filter_by_bounds(sheet.bounds().loosened(Self::INPUT_OVERSHOOT)) {
                    self.rect_start = element.pos;
                    self.rect_current = element.pos;

                    let shapestroke = Stroke::ShapeStroke(ShapeStroke::new(element, self));
                    let current_stroke_key = store.insert_stroke(shapestroke);

                    store.regenerate_rendering_for_stroke_threaded(
                        current_stroke_key,
                        camera.image_scale(),
                    );

                    self.current_stroke_key = Some(current_stroke_key);
                }
            }
            (
                Some(current_stroke_key),
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                if !element.filter_by_bounds(sheet.bounds().loosened(Self::INPUT_OVERSHOOT)) {
                    store.update_shapestroke(current_stroke_key, self, element);

                    store.regenerate_rendering_for_stroke_threaded(
                        current_stroke_key,
                        camera.image_scale(),
                    );
                }
            }
            (None, PenEvent::Up { .. }) => {}
            (
                Some(current_stroke_key),
                PenEvent::Up {
                    element,
                    shortcut_key: _,
                },
            ) => {
                store.update_shapestroke(current_stroke_key, self, element);

                finish_current_stroke(current_stroke_key, sheet, store, camera);
                self.current_stroke_key = None;
            }
            (None, PenEvent::Proximity { .. }) => {}
            (Some(current_stroke_key), PenEvent::Proximity { .. }) => {
                finish_current_stroke(current_stroke_key, sheet, store, camera);
                self.current_stroke_key = None;
            }
            (None, PenEvent::Cancel) => {}
            (Some(current_stroke_key), PenEvent::Cancel) => {
                finish_current_stroke(current_stroke_key, sheet, store, camera);
                self.current_stroke_key = None;
            }
        }

        surface_flags
    }
}

impl DrawOnSheetBehaviour for Shaper {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        None
    }

    fn draw_on_sheet(
        &self,
        _cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        _camera: &Camera,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

fn finish_current_stroke(
    current_stroke_key: StrokeKey,
    _sheet: &mut Sheet,
    store: &mut StrokeStore,
    camera: &Camera,
) {
    store.update_geometry_for_stroke(current_stroke_key);

    store.regenerate_rendering_for_stroke_threaded(current_stroke_key, camera.image_scale());
}

impl Shaper {
    pub const INPUT_OVERSHOOT: f64 = 30.0;
}
