use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use crate::{Camera, DrawOnSheetBehaviour, Sheet, StrokeStore, SurfaceFlags};
use piet::RenderContext;
use rnote_compose::builders::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use rnote_compose::builders::{PenPathBuilder, ShapeBuilderBehaviour};
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::penpath::Segment;
use rnote_compose::{Shape, Style};

use p2d::bounding_volume::{BoundingVolume, AABB};
use rand::{Rng, SeedableRng};
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::style::textured::TexturedOptions;
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
                PenEvent::Down {
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

                    let path_builder = PenPathBuilder::start(element);

                    if let Err(e) = store.regenerate_rendering_for_stroke(
                        current_stroke_key,
                        camera.viewport_extended(),
                        camera.image_scale(),
                    ) {
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
                    current_stroke_key, ..
                },
                PenEvent::Cancel,
            ) => {
                Self::stop_audio(style, audioplayer);

                // Finish up the last stroke
                store.update_geometry_for_stroke(*current_stroke_key);
                store.regenerate_rendering_for_stroke_threaded(
                    *current_stroke_key,
                    camera.viewport_extended(),
                    camera.image_scale(),
                );

                self.state = BrushState::Idle;

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.sheet_changed = true;
                surface_flags.hide_scrollbars = Some(false);

                PenProgress::Finished
            }
            (
                BrushState::Drawing {
                    path_builder,
                    current_stroke_key,
                },
                pen_event,
            ) => {
                match path_builder.handle_event(pen_event) {
                    BuilderProgress::InProgress => {
                        surface_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    BuilderProgress::EmitContinue(shapes) => {
                        let mut n_segments = 0;

                        for shape in shapes {
                            match shape {
                                Shape::Segment(new_segment) => {
                                    store.add_segment_to_brushstroke(
                                        *current_stroke_key,
                                        new_segment,
                                    );
                                    n_segments += 1;
                                    surface_flags.sheet_changed = true;
                                }
                                _ => {
                                    // not reachable, pen builder should only produce segments
                                }
                            }
                        }

                        if let Err(e) = store.append_rendering_last_segments(
                            *current_stroke_key,
                            n_segments,
                            camera.viewport_extended(),
                            camera.image_scale(),
                        ) {
                            log::error!("append_rendering_last_segments() for penevent down in brush failed with Err {}", e);
                        }

                        PenProgress::InProgress
                    }
                    BuilderProgress::Finished(Some(shapes)) => {
                        for shape in shapes {
                            match shape {
                                Shape::Segment(new_segment) => {
                                    store.add_segment_to_brushstroke(
                                        *current_stroke_key,
                                        new_segment,
                                    );
                                    surface_flags.sheet_changed = true;
                                }
                                _ => {
                                    // not reachable, pen builder should only produce segments
                                }
                            }
                        }

                        // Finish up the last stroke
                        store.update_geometry_for_stroke(*current_stroke_key);
                        store.regenerate_rendering_for_stroke_threaded(
                            *current_stroke_key,
                            camera.viewport_extended(),
                            camera.image_scale(),
                        );

                        Self::stop_audio(style, audioplayer);

                        self.state = BrushState::Idle;

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                        surface_flags.hide_scrollbars = Some(false);

                        PenProgress::Finished
                    }
                    BuilderProgress::Finished(None) => {
                        // Finish up the last stroke
                        store.update_geometry_for_stroke(*current_stroke_key);
                        store.regenerate_rendering_for_stroke_threaded(
                            *current_stroke_key,
                            camera.viewport_extended(),
                            camera.image_scale(),
                        );

                        Self::stop_audio(style, audioplayer);

                        self.state = BrushState::Idle;

                        surface_flags.redraw = true;
                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                        surface_flags.hide_scrollbars = Some(false);

                        PenProgress::Finished
                    }
                }
            }
        };

        (pen_progress, surface_flags)
    }
}

impl DrawOnSheetBehaviour for Brush {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        let style = self.gen_style_for_current_options();

        match &self.state {
            BrushState::Idle => None,
            BrushState::Drawing { path_builder, .. } => {
                Some(path_builder.bounds(&style, camera.zoom()))
            }
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        _sheet_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        match &self.state {
            BrushState::Idle => {}
            BrushState::Drawing { path_builder, .. } => {
                let style = self.gen_style_for_current_options();
                path_builder.draw_styled(cx, &style, camera.total_zoom());
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
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
