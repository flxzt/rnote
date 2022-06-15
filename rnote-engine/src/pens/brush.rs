use super::penbehaviour::{PenBehaviour, PenProgress};
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use crate::AudioPlayer;
use crate::{DrawOnDocBehaviour, WidgetFlags};
use rnote_compose::builders::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use rnote_compose::builders::Constraints;
use rnote_compose::builders::{PenPathBuilder, ShapeBuilderBehaviour};
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::penpath::Segment;
use rnote_compose::style::textured::TexturedOptions;
use rnote_compose::style::PressureCurve;
use rnote_compose::{Shape, Style};

use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use rand::{Rng, SeedableRng};
use rnote_compose::style::smooth::SmoothOptions;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "brush_style")]
pub enum BrushStyle {
    #[serde(rename = "marker")]
    Marker = 0,
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

impl TryFrom<u32> for BrushStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("BrushStyle try_from::<u32>() for value {} failed", value)
        })
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
        let mut smooth_options = SmoothOptions::default();
        let mut textured_options = TexturedOptions::default();
        smooth_options.stroke_width = Self::STROKE_WIDTH_DEFAULT;
        textured_options.stroke_width = Self::STROKE_WIDTH_DEFAULT;

        Self {
            style: BrushStyle::default(),
            smooth_options,
            textured_options,
            state: BrushState::Idle,
        }
    }
}

impl PenBehaviour for Brush {
    fn handle_event(
        &mut self,
        event: PenEvent,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();
        let style = self.style;

        let pen_progress = match (&mut self.state, event) {
            (
                BrushState::Idle,
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                if !element
                    .filter_by_bounds(engine_view.doc.bounds().loosened(Self::INPUT_OVERSHOOT))
                {
                    widget_flags.merge_with_other(engine_view.store.record());

                    Self::start_audio(style, engine_view.audioplayer);

                    // A new seed for a new brush stroke
                    let seed = Some(rand_pcg::Pcg64::from_entropy().gen());
                    self.textured_options.seed = seed;

                    let brushstroke = Stroke::BrushStroke(BrushStroke::new(
                        Segment::Dot { element },
                        self.gen_style_for_current_options(),
                    ));
                    let current_stroke_key = engine_view.store.insert_stroke(brushstroke);

                    let path_builder = PenPathBuilder::start(element);

                    if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                        current_stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    ) {
                        log::error!("regenerate_rendering_for_stroke() failed after inserting brush stroke, Err {}", e);
                    }

                    self.state = BrushState::Drawing {
                        path_builder,
                        current_stroke_key,
                    };

                    widget_flags.redraw = true;
                    widget_flags.hide_scrollbars = Some(true);

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
                Self::stop_audio(style, engine_view.audioplayer);

                // Finish up the last stroke
                engine_view
                    .store
                    .update_geometry_for_stroke(*current_stroke_key);
                engine_view.store.regenerate_rendering_for_stroke_threaded(
                    engine_view.tasks_tx.clone(),
                    *current_stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.state = BrushState::Idle;

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;
                widget_flags.hide_scrollbars = Some(false);

                PenProgress::Finished
            }
            (
                BrushState::Drawing {
                    path_builder,
                    current_stroke_key,
                },
                pen_event,
            ) => {
                match path_builder.handle_event(pen_event, Constraints::default()) {
                    BuilderProgress::InProgress => {
                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    BuilderProgress::EmitContinue(shapes) => {
                        let mut n_segments = 0;

                        for shape in shapes {
                            match shape {
                                Shape::Segment(new_segment) => {
                                    engine_view.store.add_segment_to_brushstroke(
                                        *current_stroke_key,
                                        new_segment,
                                    );
                                    n_segments += 1;
                                    widget_flags.indicate_changed_store = true;
                                }
                                _ => {
                                    // not reachable, pen builder should only produce segments
                                }
                            }
                        }

                        if let Err(e) = engine_view.store.append_rendering_last_segments(
                            engine_view.tasks_tx.clone(),
                            *current_stroke_key,
                            n_segments,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        ) {
                            log::error!("append_rendering_last_segments() for penevent down in brush failed with Err {}", e);
                        }
                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    BuilderProgress::Finished(shapes) => {
                        for shape in shapes {
                            match shape {
                                Shape::Segment(new_segment) => {
                                    engine_view.store.add_segment_to_brushstroke(
                                        *current_stroke_key,
                                        new_segment,
                                    );
                                    widget_flags.indicate_changed_store = true;
                                }
                                _ => {
                                    // not reachable, pen builder should only produce segments
                                }
                            }
                        }

                        // Finish up the last stroke
                        engine_view
                            .store
                            .update_geometry_for_stroke(*current_stroke_key);
                        engine_view.store.regenerate_rendering_for_stroke_threaded(
                            engine_view.tasks_tx.clone(),
                            *current_stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        Self::stop_audio(style, engine_view.audioplayer);

                        self.state = BrushState::Idle;

                        engine_view
                            .doc
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        widget_flags.redraw = true;
                        widget_flags.resize = true;
                        widget_flags.indicate_changed_store = true;
                        widget_flags.hide_scrollbars = Some(false);

                        PenProgress::Finished
                    }
                }
            }
        };

        (pen_progress, widget_flags)
    }
}

impl DrawOnDocBehaviour for Brush {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<AABB> {
        let style = self.gen_style_for_current_options();

        match &self.state {
            BrushState::Idle => None,
            BrushState::Drawing { path_builder, .. } => {
                path_builder.bounds(&style, engine_view.camera.zoom())
            }
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        match &self.state {
            BrushState::Idle => {}
            BrushState::Drawing { path_builder, .. } => {
                let style = self.gen_style_for_current_options();
                path_builder.draw_styled(cx, &style, engine_view.camera.total_zoom());
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

impl Brush {
    const INPUT_OVERSHOOT: f64 = 30.0;

    pub const STROKE_WIDTH_MIN: f64 = 1.0;
    pub const STROKE_WIDTH_MAX: f64 = 500.0;
    pub const STROKE_WIDTH_DEFAULT: f64 = 2.0;

    fn start_audio(style: BrushStyle, audioplayer: &mut Option<AudioPlayer>) {
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

    fn stop_audio(_style: BrushStyle, audioplayer: &mut Option<AudioPlayer>) {
        if let Some(audioplayer) = audioplayer {
            audioplayer.stop_random_brush_sond();
        }
    }

    pub fn gen_style_for_current_options(&self) -> Style {
        match &self.style {
            BrushStyle::Marker => {
                let mut options = self.smooth_options.clone();
                options.pressure_curve = PressureCurve::Const;

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
