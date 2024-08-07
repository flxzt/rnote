// Imports
use super::pensconfig::brushconfig::BrushStyle;
use super::PenBehaviour;
use super::PenStyle;
use crate::engine::EngineTask;
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::builders::buildable::{Buildable, BuilderCreator, BuilderProgress};
use rnote_compose::builders::{
    PenPathBuilderType, PenPathCurvedBuilder, PenPathModeledBuilder, PenPathSimpleBuilder,
};
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::penevent::{PenEvent, PenProgress};
use rnote_compose::penpath::{Element, Segment};
use rnote_compose::Constraints;
use rnote_compose::PenPath;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
enum BrushState {
    Idle,
    Drawing {
        path_builder: Box<dyn Buildable<Emit = Segment>>,
        current_stroke_key: StrokeKey,
    },
}

#[derive(Debug)]
pub struct Brush {
    state: BrushState,
    /// if we have a long hold of the pen, we save the
    /// penpath for recognition one level upper
    pub pen_path_recognition: Option<PenPath>,
    longpress_handle: Option<crate::tasks::PeriodicTaskHandle>,
    pub current_stroke_key: Option<StrokeKey>,

    // dumb thing : take the start time of the stroke and transform to a line after 1 second
    pub time_start: Option<Instant>, // maybe we can do that as a speed based detection
                                     // if the speed is lower than ... in the timeout
                                     // calculate here the state
                                     // for a pen that's down and not moving
                                     // vecdecque of the last position + distance to the previous element
                                     // for all events inside the timeout
                                     // and a distance on the side

                                     // then test inside handle event to trigger the long press

                                     // but we still need the task if we hold the pen down
                                     // So what we need to do is to have a periodic task
                                     // where we send times of relevant pen events

                                     // when read, it sleeps until the time + timeout is done
                                     // then tests if the channel is empty
                                     // if it is, then calls the task (and the "does the task need to happen" test is done)
                                     // if not, reads the next messages

                                     // we need to keep the state of whether a long hold has occured or not as well
                                     // to not trigger the same code twice

                                     // we'll start with the internal state bookkeeping then try the other part after

                                     // another thing is to not trigger this too soon ? for a pen stuck on the start position
                                     // maybe not good ?
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            state: BrushState::Idle,
            pen_path_recognition: None,
            time_start: None,
            current_stroke_key: None,
            longpress_handle: None,
        }
    }
}

impl PenBehaviour for Brush {
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        self.longpress_handle = None;
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

        // we should have a special task on a separate thread that sends event
        // with the channel the strategy is the following
        // - CANCEL
        //      the state is idle
        //      pen cancel event
        // - if we start with drawing and the pen is down, start signal
        // - as long as we stay in drawing mode :
        //      - send pen down events
        //      -

        let event_result = match (&mut self.state, event) {
            (BrushState::Idle, PenEvent::Down { element, .. }) => {
                if !element.filter_by_bounds(
                    engine_view
                        .document
                        .bounds()
                        .loosened(Self::INPUT_OVERSHOOT),
                ) {
                    if engine_view.pens_config.brush_config.style == BrushStyle::Marker {
                        play_marker_sound(engine_view);
                    } else {
                        trigger_brush_sound(engine_view);
                    }

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

                    engine_view.store.regenerate_rendering_for_stroke(
                        current_stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    );

                    self.state = BrushState::Drawing {
                        path_builder: new_builder(
                            engine_view.pens_config.brush_config.builder_type,
                            element,
                            now,
                        ),
                        current_stroke_key,
                    };
                    self.time_start = Some(now);
                    let tasks_tx = engine_view.tasks_tx.clone();
                    let longpress_reminder = move || -> crate::tasks::PeriodicTaskResult {
                        tasks_tx.send(EngineTask::LongPressStatic);
                        crate::tasks::PeriodicTaskResult::Continue
                    };
                    self.longpress_handle = Some(crate::tasks::PeriodicTaskHandle::new(
                        longpress_reminder,
                        Duration::from_secs(2),
                    ));

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
                self.time_start = None;

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
                },
                pen_event,
            ) => {
                let builder_result =
                    path_builder.handle_event(pen_event, now, Constraints::default());
                let handled = builder_result.handled;
                let propagate = builder_result.propagate;

                let progress = match builder_result.progress {
                    BuilderProgress::InProgress => {
                        if engine_view.pens_config.brush_config.style != BrushStyle::Marker {
                            trigger_brush_sound(engine_view);
                        }

                        PenProgress::InProgress
                    }
                    BuilderProgress::EmitContinue(segments) => {
                        if engine_view.pens_config.brush_config.style != BrushStyle::Marker {
                            trigger_brush_sound(engine_view);
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

                        // then test
                        let delta = now - self.time_start.unwrap();
                        if delta > Duration::from_secs(1) {
                            //triger the actual change
                            // we need to save the stroke data before deleting it
                            println!("saving the current stroke data");
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_ref(*current_stroke_key)
                            {
                                let path = brushstroke.path.clone();
                                println!("the path is {:?}", path);

                                // save to a location
                                self.pen_path_recognition = Some(path);
                            }
                            println!("cancelled stroke");
                            // this HAS to happen AFTER the recognition is done and successful
                            // dummy test : do this half the time
                            self.current_stroke_key = Some(current_stroke_key.clone());
                            // can't do that here : need to do this two steps higher
                            //engine_view.store.remove_stroke(*current_stroke_key);
                            widget_flags.long_hold = true;
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

                        // the normal way this would happen would be at the previous step : for an in progress penprogress
                        if false {
                            // we need to save the stroke data before deleting it
                            println!("saving the current stroke data");
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_ref(*current_stroke_key)
                            {
                                let path = brushstroke.path.clone();
                                println!("the path is {:?}", path);

                                // save to a location
                                self.pen_path_recognition = Some(path);
                            }
                            println!("cancelled stroke");
                            engine_view.store.remove_stroke(*current_stroke_key);
                            widget_flags.long_hold = true;
                        }
                        // change to a shaper : need to use the widget flags higher up

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
        };

        (event_result, widget_flags)
    }
}

impl DrawableOnDoc for Brush {
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

    // reset the long press
    pub fn reset_long_press(&mut self) {
        self.time_start = Some(Instant::now());
        // this resets the time to the current one (dummy test)
    }
}

fn play_marker_sound(engine_view: &mut EngineViewMut) {
    if let Some(audioplayer) = engine_view.audioplayer {
        audioplayer.play_random_marker_sound();
    }
}

fn trigger_brush_sound(engine_view: &mut EngineViewMut) {
    if let Some(audioplayer) = engine_view.audioplayer.as_mut() {
        audioplayer.trigger_random_brush_sound();
    }
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
