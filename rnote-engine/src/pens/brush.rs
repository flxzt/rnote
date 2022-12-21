use std::time::Instant;

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::pensconfig::brushconfig::BrushStyle;
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use crate::{DrawOnDocBehaviour, WidgetFlags};
use rnote_compose::builders::{
    Constraints, PenPathBuilderBehaviour, PenPathBuilderCreator, PenPathBuilderProgress,
    PenPathModeledBuilder,
};
use rnote_compose::builders::{PenPathCurvedBuilder, PenPathSimpleBuilder};
use rnote_compose::penevents::PenEvent;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::builders::PenPathBuilderType;

#[derive(Debug)]
enum BrushState {
    Idle,
    Drawing {
        path_builder: Box<dyn PenPathBuilderBehaviour>,
        current_stroke_key: StrokeKey,
    },
}

#[derive(Debug)]
pub struct Brush {
    state: BrushState,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            state: BrushState::Idle,
        }
    }
}

impl PenBehaviour for Brush {
    fn style(&self) -> PenStyle {
        PenStyle::Brush
    }

    fn update_state(&mut self, _engine_view: &mut EngineViewMut) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

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
                    widget_flags.merge(engine_view.store.record(Instant::now()));
                    self.start_audio(engine_view);
                    engine_view.pens_config.brush_config.new_style_seeds();

                    let brushstroke = Stroke::BrushStroke(BrushStroke::new(
                        element,
                        engine_view
                            .pens_config
                            .brush_config
                            .style_for_current_options(),
                    ));
                    let current_stroke_key = engine_view.store.insert_stroke(
                        brushstroke,
                        Some(
                            engine_view
                                .pens_config
                                .brush_config
                                .layer_for_current_options(),
                        ),
                    );

                    let path_builder: Box<dyn PenPathBuilderBehaviour> =
                        match engine_view.pens_config.brush_config.builder_type {
                            PenPathBuilderType::Simple => {
                                Box::new(PenPathSimpleBuilder::start(element, now))
                            }
                            PenPathBuilderType::Curved => {
                                Box::new(PenPathCurvedBuilder::start(element, now))
                            }
                            PenPathBuilderType::Modeled => {
                                Box::new(PenPathModeledBuilder::start(element, now))
                            }
                        };

                    engine_view.store.regenerate_rendering_for_stroke(
                        current_stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    );

                    self.state = BrushState::Drawing {
                        path_builder,
                        current_stroke_key,
                    };

                    widget_flags.redraw = true;

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

                self.stop_audio(engine_view);

                PenProgress::Finished
            }
            (
                BrushState::Drawing {
                    path_builder,
                    current_stroke_key,
                },
                pen_event,
            ) => {
                match path_builder.handle_event(pen_event, now, Constraints::default()) {
                    PenPathBuilderProgress::InProgress => {
                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    PenPathBuilderProgress::EmitContinue(segments) => {
                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.indicate_changed_store = true;
                            }

                            engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    PenPathBuilderProgress::Finished(segments) => {
                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.indicate_changed_store = true;
                            }

                            // First we draw the last segments immediately,
                            engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        // but then regenerate the entire stroke rendering because it gets rid of some artifacts
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

                        self.stop_audio(engine_view);

                        self.state = BrushState::Idle;

                        engine_view
                            .doc
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        widget_flags.redraw = true;
                        widget_flags.resize = true;
                        widget_flags.indicate_changed_store = true;

                        PenProgress::Finished
                    }
                }
            }
        };

        (pen_progress, widget_flags)
    }
}

impl DrawOnDocBehaviour for Brush {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let style = engine_view
            .pens_config
            .brush_config
            .style_for_current_options();

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
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        match &self.state {
            BrushState::Idle => {}
            BrushState::Drawing { path_builder, .. } => {
                match engine_view.pens_config.brush_config.style {
                    BrushStyle::Marker => {
                        // Don't draw the marker, as the pen would render on top of other strokes, while the stroke itself would render underneath them.
                    }
                    BrushStyle::Solid | BrushStyle::Textured => {
                        let style = engine_view
                            .pens_config
                            .brush_config
                            .style_for_current_options();

                        /*
                                               // Change color for debugging
                                               match &mut style {
                                                   Style::Smooth(options) => {
                                                       options.stroke_color = Some(rnote_compose::Color::RED)
                                                   }
                                                   Style::Rough(_) | Style::Textured(_) => {}
                                               }
                        */

                        path_builder.draw_styled(cx, &style, engine_view.camera.total_zoom());
                    }
                }
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Brush {
    const INPUT_OVERSHOOT: f64 = 30.0;

    fn start_audio(&self, engine_view: &mut EngineViewMut) {
        if let Some(audioplayer) = engine_view.audioplayer {
            match engine_view.pens_config.brush_config.style {
                BrushStyle::Marker => {
                    audioplayer.play_random_marker_sound();
                }
                BrushStyle::Solid | BrushStyle::Textured => {
                    audioplayer.start_random_brush_sound();
                }
            }
        }
    }

    fn stop_audio(&self, engine_view: &mut EngineViewMut) {
        if let Some(audioplayer) = engine_view.audioplayer {
            audioplayer.stop_random_brush_sond();
        }
    }
}
