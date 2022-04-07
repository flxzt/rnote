use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use crate::strokesstate::StrokeKey;
use crate::{Camera, DrawOnSheetBehaviour, Sheet, StrokesState};
use rnote_compose::builders::{PenPathBuilder, ShapeBuilderBehaviour};
use rnote_compose::penpath::Segment;
use rnote_compose::PenEvent;

use gtk4::glib;
use p2d::bounding_volume::{BoundingVolume, AABB};
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::style::textured::TexturedOptions;
use rnote_compose::style::Composer;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, glib::Enum)]
#[repr(u32)]
#[enum_type(name = "BrushStyle")]
#[serde(rename = "brush_style")]
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
    current_stroke_key: Option<StrokeKey>,
    #[serde(skip)]
    path_builder: PenPathBuilder,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            style: BrushStyle::default(),
            smooth_options: SmoothOptions::default(),
            textured_options: TexturedOptions::default(),
            current_stroke_key: None,
            path_builder: PenPathBuilder::default(),
        }
    }
}

impl PenBehaviour for Brush {
    fn handle_event(
        &mut self,
        event: PenEvent,
        sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        camera: &Camera,
    ) {
        match (self.current_stroke_key, event) {
            (
                None,
                pen_event @ PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                if !element.filter_by_bounds(sheet.bounds().loosened(Self::INPUT_OVERSHOOT)) {
                    let brushstroke =
                        Stroke::BrushStroke(BrushStroke::new(Segment::Dot { element }, &self));
                    let current_stroke_key = strokes_state.insert_stroke(brushstroke);
                    self.current_stroke_key = Some(current_stroke_key);

                    if let Some(new_segments) = self.path_builder.handle_event(pen_event) {
                        for new_segment in new_segments {
                            strokes_state
                                .add_segment_to_brushstroke(current_stroke_key, new_segment);
                        }
                    }

                    strokes_state.regenerate_rendering_for_stroke_threaded(
                        current_stroke_key,
                        camera.image_scale(),
                    );
                }
            }
            (
                Some(current_stroke_key),
                pen_event @ PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                if !element.filter_by_bounds(sheet.bounds().loosened(Self::INPUT_OVERSHOOT)) {
                    if let Some(new_segments) = self.path_builder.handle_event(pen_event) {
                        let no_segments = new_segments.len();

                        for new_segment in new_segments {
                            strokes_state
                                .add_segment_to_brushstroke(current_stroke_key, new_segment);
                        }

                        strokes_state.append_rendering_last_segments(
                            current_stroke_key,
                            no_segments,
                            camera.image_scale(),
                        );

                        /*                         strokes_state
                        .regenerate_rendering_for_stroke_threaded(current_stroke_key, zoom); */
                    }
                }
            }
            (None, PenEvent::Up { .. }) => {}
            (Some(current_stroke_key), pen_event @ PenEvent::Up { .. }) => {
                if let Some(new_segments) = self.path_builder.handle_event(pen_event) {
                    for new_segment in new_segments {
                        strokes_state.add_segment_to_brushstroke(current_stroke_key, new_segment);
                    }
                }
                // Finish up the last stroke
                strokes_state.update_geometry_for_stroke(current_stroke_key);
                strokes_state.regenerate_rendering_for_stroke_threaded(
                    current_stroke_key,
                    camera.image_scale(),
                );
                self.current_stroke_key = None;
            }
            (None, pen_event @ PenEvent::Cancel) => {
                self.path_builder.handle_event(pen_event);
            }
            (Some(current_stroke_key), pen_event @ PenEvent::Cancel) => {
                self.path_builder.handle_event(pen_event);

                // Finish up the last stroke
                strokes_state.update_geometry_for_stroke(current_stroke_key);
                strokes_state.regenerate_rendering_for_stroke_threaded(
                    current_stroke_key,
                    camera.image_scale(),
                );
                self.current_stroke_key = None;
            }
            (None, PenEvent::Proximity { .. }) => {}
            (Some(_), PenEvent::Proximity { .. }) => {}
        }
    }
}

impl DrawOnSheetBehaviour for Brush {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _viewport: AABB) -> Option<AABB> {
        let bounds = match self.style {
            BrushStyle::Marker => self.path_builder.composed_bounds(&self.smooth_options),
            BrushStyle::Solid => self.path_builder.composed_bounds(&self.smooth_options),
            BrushStyle::Textured => self.path_builder.composed_bounds(&self.textured_options),
        };

        Some(bounds)
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        _viewport: AABB,
    ) -> Result<(), anyhow::Error> {
        // Different color for debugging
        let smooth_options = self.smooth_options;
        /*         smooth_options.stroke_color = Some(rnote_compose::Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }); */

        match self.style {
            BrushStyle::Marker => {
                self.path_builder.draw_composed(cx, &smooth_options);
            }
            BrushStyle::Solid => {
                self.path_builder.draw_composed(cx, &smooth_options);
            }
            BrushStyle::Textured => {
                self.path_builder.draw_composed(cx, &self.textured_options);
            }
        }

        Ok(())
    }
}

impl Brush {
    pub const INPUT_OVERSHOOT: f64 = 30.0;
}