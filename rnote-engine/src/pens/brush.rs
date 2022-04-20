use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use crate::{Camera, DrawOnSheetBehaviour, Sheet, StrokeStore, SurfaceFlags};
use rnote_compose::builders::{PenPathBuilder, ShapeBuilderBehaviour};
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::penpath::Segment;
use rnote_compose::Style;

use p2d::bounding_volume::{BoundingVolume, AABB};
use rand::{Rng, SeedableRng};
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::style::textured::TexturedOptions;
use rnote_compose::style::Composer;
use serde::{Deserialize, Serialize};

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::AudioPlayer;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "brush_style")]
pub enum BrushStyle {
    #[serde(rename = "marker")]
    Marker,
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "textured")]
    Textured,
}

impl Default for BrushStyle {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Debug, Clone)]
enum BrushState {
    Idle,
    Drawing {
        path_builder: PenPathBuilder,
        current_stroke_key: StrokeKey,
    },
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
    state: BrushState,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            style: BrushStyle::default(),
            smooth_options: SmoothOptions::default(),
            textured_options: TexturedOptions::default(),
            state: BrushState::Idle,
        }
    }
}

impl PenBehaviour for Brush {
    fn handle_event(
        &mut self,
        event: PenEvent,
        sheet: &mut Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
        audioplayer: Option<&mut AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags) {
        let mut surface_flags = SurfaceFlags::default();
        let style = self.style;

        let pen_progress = match (&mut self.state, event) {
            (
                BrushState::Idle,
                pen_event @ PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                if !element.filter_by_bounds(sheet.bounds().loosened(Self::INPUT_OVERSHOOT)) {
                    Self::start_audio(style, audioplayer);

                    // A new seed for a new brush stroke
                    let seed = Some(rand_pcg::Pcg64::from_entropy().gen());
                    self.textured_options.seed = seed;

                    let brushstroke = Stroke::BrushStroke(BrushStroke::new(
                        Segment::Dot { element },
                        self.gen_style_for_current_options(),
                    ));
                    let current_stroke_key = store.insert_stroke(brushstroke);

                    let mut path_builder = PenPathBuilder::start(element);

                    if let Some(new_segments) = path_builder.handle_event(pen_event) {
                        for new_segment in new_segments {
                            store.add_segment_to_brushstroke(current_stroke_key, new_segment);
                        }

                        surface_flags.sheet_changed = true;
                    }

                    if let Err(e) = store
                        .regenerate_rendering_for_stroke(current_stroke_key, camera.image_scale())
                    {
                        log::error!("regenerate_rendering_for_stroke() failed after inserting brush stroke, Err {}", e);
                    }

                    self.state = BrushState::Drawing {
                        path_builder,
                        current_stroke_key,
                    };

                    surface_flags.redraw = true;
                    surface_flags.hide_scrollbars = Some(true);

                    PenProgress::InProgress
                } else {
                    PenProgress::Idle
                }
            }
            (BrushState::Idle, _) => PenProgress::Idle,
            (
                BrushState::Drawing {
                    path_builder,
                    current_stroke_key,
                },
                pen_event @ PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                if !element.filter_by_bounds(sheet.bounds().loosened(Self::INPUT_OVERSHOOT)) {
                    if let Some(new_segments) = path_builder.handle_event(pen_event) {
                        let n_segments = new_segments.len();

                        for new_segment in new_segments {
                            store.add_segment_to_brushstroke(*current_stroke_key, new_segment);
                        }

                        if let Err(e) = store.append_rendering_last_segments(
                            *current_stroke_key,
                            n_segments,
                            camera.image_scale(),
                        ) {
                            log::error!("append_rendering_last_segments() for penevent down in brush failed with Err {}", e);
                        }

                        surface_flags.sheet_changed = true;
                    }

                    surface_flags.redraw = true;
                }

                PenProgress::InProgress
            }
            (
                BrushState::Drawing {
                    ref mut path_builder,
                    current_stroke_key,
                },
                pen_event @ PenEvent::Up {
                    element: _,
                    shortcut_key: _,
                },
            ) => {
                Self::stop_audio(style, audioplayer);

                if let Some(new_segments) = path_builder.handle_event(pen_event) {
                    for new_segment in new_segments {
                        store.add_segment_to_brushstroke(*current_stroke_key, new_segment);
                    }
                }

                // Finish up the last stroke
                store.update_geometry_for_stroke(*current_stroke_key);
                store.regenerate_rendering_for_stroke_threaded(
                    *current_stroke_key,
                    camera.image_scale(),
                );

                self.state = BrushState::Idle;

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.sheet_changed = true;
                surface_flags.hide_scrollbars = Some(false);

                PenProgress::Finished
            }
            (BrushState::Drawing { .. }, PenEvent::Proximity { .. }) => PenProgress::InProgress,
            (
                BrushState::Drawing {
                    current_stroke_key, ..
                },
                PenEvent::Cancel,
            ) => {
                Self::stop_audio(style, audioplayer);

                // Finish up the last stroke
                store.update_geometry_for_stroke(*current_stroke_key);
                store.regenerate_rendering_for_stroke_threaded(
                    *current_stroke_key,
                    camera.image_scale(),
                );

                self.state = BrushState::Idle;

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.sheet_changed = true;
                surface_flags.hide_scrollbars = Some(false);

                PenProgress::Finished
            }
        };

        (pen_progress, surface_flags)
    }
}

impl DrawOnSheetBehaviour for Brush {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        match (&self.state, self.style) {
            (BrushState::Idle, _) => None,
            (BrushState::Drawing { path_builder, .. }, BrushStyle::Marker) => {
                Some(path_builder.composed_bounds(&self.smooth_options))
            }
            (BrushState::Drawing { path_builder, .. }, BrushStyle::Solid) => {
                Some(path_builder.composed_bounds(&self.smooth_options))
            }
            (BrushState::Drawing { path_builder, .. }, BrushStyle::Textured) => {
                Some(path_builder.composed_bounds(&self.textured_options))
            }
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        _camera: &Camera,
    ) -> anyhow::Result<()> {
        match (&self.state, self.style) {
            (BrushState::Drawing { path_builder, .. }, BrushStyle::Marker) => {
                let mut smooth_options = self.smooth_options.clone();
                smooth_options.segment_constant_width = true;

                path_builder.draw_composed(cx, &smooth_options);
            }
            (BrushState::Drawing { path_builder, .. }, BrushStyle::Solid) => {
                path_builder.draw_composed(cx, &self.smooth_options);
            }
            (BrushState::Drawing { path_builder, .. }, BrushStyle::Textured) => {
                path_builder.draw_composed(cx, &self.textured_options);
            }
            _ => {}
        }

        Ok(())
    }
}

impl Brush {
    pub const INPUT_OVERSHOOT: f64 = 30.0;

    fn start_audio(style: BrushStyle, audioplayer: Option<&mut AudioPlayer>) {
        if let Some(audioplayer) = audioplayer {
            match style {
                BrushStyle::Marker => {
                    audioplayer.play_random_marker_sound();
                }
                BrushStyle::Solid | BrushStyle::Textured => {
                    audioplayer.start_random_brush_sound();
                }
            }
        }
    }

    fn stop_audio(_style: BrushStyle, audioplayer: Option<&mut AudioPlayer>) {
        if let Some(audioplayer) = audioplayer {
            audioplayer.stop_random_brush_sond();
        }
    }

    pub fn gen_style_for_current_options(&self) -> Style {
        match &self.style {
            BrushStyle::Marker => {
                let mut options = self.smooth_options.clone();
                options.segment_constant_width = true;

                Style::Smooth(options)
            }
            BrushStyle::Solid => {
                let options = self.smooth_options.clone();

                Style::Smooth(options)
            }
            BrushStyle::Textured => {
                let options = self.textured_options.clone();

                Style::Textured(options)
            }
        }
    }
}
