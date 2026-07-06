// Imports
use super::PenBehaviour;
use super::PenStyle;
use super::pensconfig::brushconfig::BrushStyle;
use crate::engine::{EngineTask, EngineTaskSender, EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::tasks::OneOffTaskHandle;
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;
use piet::RenderContext;
use rnote_compose::Constraints;
use rnote_compose::Style;
use rnote_compose::builders::buildable::{Buildable, BuilderCreator, BuilderProgress};
use rnote_compose::builders::{
    PenPathBuilderType, PenPathCurvedBuilder, PenPathModeledBuilder, PenPathSimpleBuilder,
};
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::penevent::{PenEvent, PenProgress};
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::shaperecognition;
use std::time::{Duration, Instant};

#[derive(Debug)]
enum BrushState {
    Idle,
    Drawing {
        path_builder: Box<dyn Buildable<Emit = Segment>>,
        current_stroke_key: StrokeKey,
        preview_style: Style,
    },
    /// The drawn stroke was replaced by a recognized shape while the pen is still down.
    ///
    /// All events are swallowed until the pen is lifted.
    WaitForPenUp,
}

#[derive(Debug)]
pub struct Brush {
    state: BrushState,
    /// The position where the pen last came (approximately) to rest, while drawing.
    hold_anchor: Vector2,
    /// The time the pen came to rest at the current hold anchor.
    hold_begin: Instant,
    /// Task that fires when the pen was held still long enough to trigger shape recognition.
    hold_task_handle: Option<OneOffTaskHandle>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            state: BrushState::Idle,
            hold_anchor: Vector2::ZERO,
            hold_begin: Instant::now(),
            hold_task_handle: None,
        }
    }
}

impl PenBehaviour for Brush {
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        self.hold_task_handle = None;
        WidgetFlags::default()
    }

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
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let event_result = match (&mut self.state, event) {
            (BrushState::Idle, PenEvent::Down { element, .. }) => {
                if !element.filter_by_bounds(
                    engine_view
                        .document
                        .bounds()
                        .loosened(Self::INPUT_OVERSHOOT),
                ) {
                    #[cfg(feature = "ui")]
                    {
                        if engine_view.config.pens_config.brush_config.style == BrushStyle::Marker {
                            play_marker_sound(engine_view);
                        } else {
                            trigger_brush_sound(engine_view);
                        }
                    }

                    engine_view
                        .config
                        .pens_config
                        .brush_config
                        .new_style_seeds();

                    let preview_style = Self::get_preview_style(&engine_view.as_im());
                    let brushstroke =
                        Stroke::BrushStroke(BrushStroke::new(element, preview_style.clone()));

                    let current_stroke_key = engine_view.store.insert_stroke(
                        brushstroke,
                        Some(
                            engine_view
                                .config
                                .pens_config
                                .brush_config
                                .layer_for_current_options(),
                        ),
                    );

                    engine_view.store.regenerate_rendering_for_stroke(
                        current_stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    );

                    self.state = BrushState::Drawing {
                        path_builder: new_builder(
                            engine_view.config.pens_config.brush_config.builder_type,
                            element,
                            now,
                        ),
                        current_stroke_key,
                        preview_style,
                    };

                    self.hold_anchor = element.pos;
                    self.hold_begin = now;
                    if engine_view
                        .config
                        .pens_config
                        .brush_config
                        .shape_recognition_enabled
                    {
                        reset_hold_task(&mut self.hold_task_handle, engine_view.tasks_tx.clone());
                    } else {
                        self.hold_task_handle = None;
                    }

                    EventResult {
                        handled: true,
                        propagate: EventPropagation::Stop,
                        progress: PenProgress::InProgress,
                    }
                } else {
                    EventResult {
                        handled: false,
                        propagate: EventPropagation::Proceed,
                        progress: PenProgress::Idle,
                    }
                }
            }
            (BrushState::Idle, _) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (
                BrushState::Drawing {
                    current_stroke_key, ..
                },
                PenEvent::Cancel,
            ) => {
                if let Some(Stroke::BrushStroke(brushstroke)) =
                    engine_view.store.get_stroke_mut(*current_stroke_key)
                {
                    brushstroke.style = engine_view
                        .config
                        .pens_config
                        .brush_config
                        .style_for_current_options();
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
                widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                self.state = BrushState::Idle;
                self.hold_task_handle = None;

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.store_modified = true;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (
                BrushState::Drawing {
                    path_builder,
                    current_stroke_key,
                    ..
                },
                pen_event,
            ) => {
                // Track whether the pen is being held still,
                // which after a timeout triggers recognizing the drawn stroke as a shape.
                if engine_view
                    .config
                    .pens_config
                    .brush_config
                    .shape_recognition_enabled
                    && let PenEvent::Down { element, .. } = &pen_event
                {
                    let hold_radius = Self::HOLD_RADIUS_SURFACE / engine_view.camera.total_zoom();
                    if (element.pos - self.hold_anchor).length() > hold_radius {
                        self.hold_anchor = element.pos;
                        self.hold_begin = now;
                        reset_hold_task(&mut self.hold_task_handle, engine_view.tasks_tx.clone());
                    }
                }

                let builder_result =
                    path_builder.handle_event(pen_event, now, Constraints::default());
                let handled = builder_result.handled;
                let propagate = builder_result.propagate;

                let progress = match builder_result.progress {
                    BuilderProgress::InProgress => {
                        #[cfg(feature = "ui")]
                        {
                            if engine_view.config.pens_config.brush_config.style
                                != BrushStyle::Marker
                            {
                                trigger_brush_sound(engine_view);
                            }
                        }

                        PenProgress::InProgress
                    }
                    BuilderProgress::EmitContinue(segments) => {
                        #[cfg(feature = "ui")]
                        {
                            if engine_view.config.pens_config.brush_config.style
                                != BrushStyle::Marker
                            {
                                trigger_brush_sound(engine_view);
                            }
                        }

                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.store_modified = true;
                            }

                            engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        PenProgress::InProgress
                    }
                    BuilderProgress::Finished(segments) => {
                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.store_modified = true;
                            }

                            engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        if let Some(Stroke::BrushStroke(brushstroke)) =
                            engine_view.store.get_stroke_mut(*current_stroke_key)
                        {
                            brushstroke.style = engine_view
                                .config
                                .pens_config
                                .brush_config
                                .style_for_current_options();
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
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        self.state = BrushState::Idle;
                        self.hold_task_handle = None;

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;

                        PenProgress::Finished
                    }
                };

                EventResult {
                    handled,
                    propagate,
                    progress,
                }
            }
            (BrushState::WaitForPenUp, PenEvent::Up { .. } | PenEvent::Cancel) => {
                self.state = BrushState::Idle;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (BrushState::WaitForPenUp, _) => EventResult {
                handled: true,
                propagate: EventPropagation::Stop,
                progress: PenProgress::InProgress,
            },
        };

        (event_result, widget_flags)
    }
}

impl DrawableOnDoc for Brush {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let style = engine_view
            .config
            .pens_config
            .brush_config
            .style_for_current_options();

        match &self.state {
            BrushState::Idle | BrushState::WaitForPenUp => None,
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
            BrushState::Idle | BrushState::WaitForPenUp => {}
            BrushState::Drawing {
                path_builder,
                preview_style,
                ..
            } => {
                match engine_view.config.pens_config.brush_config.style {
                    BrushStyle::Marker => {
                        // Don't draw the marker, as the pen would render on top of other strokes, while the stroke itself would render underneath them.
                    }
                    BrushStyle::Solid | BrushStyle::Textured => {
                        path_builder.draw_styled(
                            cx,
                            preview_style,
                            engine_view.camera.total_zoom(),
                        );
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
    /// The duration the pen must be held still at the end of a drawn stroke
    /// to trigger recognizing it as a shape.
    const HOLD_DURATION: Duration = Duration::from_millis(700);
    /// The radius (in surface coordinates) the pen may wobble around the hold anchor
    /// while still being considered held still.
    const HOLD_RADIUS_SURFACE: f64 = 6.0;

    /// Attempt to recognize the currently drawn stroke as a shape and replace it,
    /// triggered when the pen was held still at the end of a drawn stroke.
    pub(crate) fn recognize_shape_on_hold(
        &mut self,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if !engine_view
            .config
            .pens_config
            .brush_config
            .shape_recognition_enabled
        {
            return widget_flags;
        }
        let BrushState::Drawing {
            current_stroke_key, ..
        } = self.state
        else {
            return widget_flags;
        };
        // Guard against the timeout task and the pen resuming movement racing each other:
        // only recognize when the pen actually rested at the hold anchor for the entire hold duration.
        if self.hold_begin.elapsed() < Self::HOLD_DURATION.mul_f64(0.9) {
            return widget_flags;
        }

        let recognized_shape = if let Some(Stroke::BrushStroke(brushstroke)) =
            engine_view.store.get_stroke_ref(current_stroke_key)
        {
            shaperecognition::recognize_shape(&brushstroke.path)
        } else {
            None
        };
        let Some(shape) = recognized_shape else {
            return widget_flags;
        };

        // Replace the drawn stroke with the recognized shape
        engine_view.store.remove_stroke(current_stroke_key);
        let shape_stroke_key = engine_view.store.insert_stroke(
            Stroke::ShapeStroke(ShapeStroke::new(
                shape,
                engine_view
                    .config
                    .pens_config
                    .brush_config
                    .style_for_recognized_shape(),
            )),
            Some(
                engine_view
                    .config
                    .pens_config
                    .brush_config
                    .layer_for_current_options(),
            ),
        );
        engine_view
            .store
            .update_geometry_for_stroke(shape_stroke_key);
        engine_view.store.regenerate_rendering_for_stroke_threaded(
            engine_view.tasks_tx.clone(),
            shape_stroke_key,
            engine_view.camera.viewport(),
            engine_view.camera.image_scale(),
        );
        widget_flags |= engine_view
            .document
            .resize_autoexpand(engine_view.store, engine_view.camera);
        widget_flags |= engine_view.store.record(Instant::now());
        widget_flags.store_modified = true;
        widget_flags.redraw = true;

        // Swallow all further events until the pen is lifted
        self.state = BrushState::WaitForPenUp;
        self.hold_task_handle = None;

        widget_flags
    }

    fn get_preview_style(engine_view: &EngineView) -> Style {
        let mut style = engine_view
            .config
            .pens_config
            .brush_config
            .style_for_current_options();

        if let Some(mut stroke_color) = style.stroke_color() {
            stroke_color.a = 1.0;
            style.set_stroke_color(stroke_color);
        }

        style
    }
}

#[cfg(feature = "ui")]
fn play_marker_sound(engine_view: &mut EngineViewMut) {
    if let Some(audioplayer) = engine_view.audioplayer {
        audioplayer.play_random_marker_sound();
    }
}

#[cfg(feature = "ui")]
fn trigger_brush_sound(engine_view: &mut EngineViewMut) {
    if let Some(audioplayer) = engine_view.audioplayer.as_mut() {
        audioplayer.trigger_random_brush_sound();
    }
}

/// (Re-)start the hold timeout, (re-)installing the one-off task when it already fired or is not installed yet.
///
/// When the timeout is reached, a task is sent to the engine
/// which triggers recognizing the currently drawn stroke as a shape.
fn reset_hold_task(handle: &mut Option<OneOffTaskHandle>, tasks_tx: EngineTaskSender) {
    if let Some(handle) = handle.as_mut()
        && handle.reset_timeout().is_ok()
    {
        return;
    }

    let hold_task = move || {
        tasks_tx.send(EngineTask::BrushRecognizeShape);
    };
    *handle = Some(OneOffTaskHandle::new(hold_task, Brush::HOLD_DURATION));
}

fn new_builder(
    builder_type: PenPathBuilderType,
    element: Element,
    now: Instant,
) -> Box<dyn Buildable<Emit = Segment>> {
    match builder_type {
        PenPathBuilderType::Simple => Box::new(PenPathSimpleBuilder::start(element, now)),
        PenPathBuilderType::Curved => Box::new(PenPathCurvedBuilder::start(element, now)),
        PenPathBuilderType::Modeled => Box::new(PenPathModeledBuilder::start(element, now)),
    }
}
