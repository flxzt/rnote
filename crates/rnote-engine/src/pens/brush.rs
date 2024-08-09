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
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Copy, Clone)]
pub struct PosTimeDict {
    pub pos: na::Vector2<f64>,
    distance_to_previous: f64,
    time: Instant,
}

impl Default for PosTimeDict {
    fn default() -> Self {
        Self {
            pos: na::Vector2::new(0.0, 0.0),
            distance_to_previous: 0.0,
            time: Instant::now(),
        }
    }
}

#[derive(Debug)]
enum BrushState {
    Idle,
    Drawing {
        path_builder: Box<dyn Buildable<Emit = Segment>>,
        current_stroke_key: StrokeKey,
    },
}

#[derive(Debug, Default)]
pub struct LongPressDetector {
    distance: f64,
    total_distance: f64,
    pub last_strokes: VecDeque<PosTimeDict>,
}

impl LongPressDetector {
    fn clear(&mut self) {
        self.last_strokes.clear();
    }

    fn total_distance(&self) -> f64 {
        self.total_distance
    }

    fn distance(&self) -> f64 {
        self.distance
    }

    fn reset(&mut self, element: Element, now: Instant) {
        self.clear();
        self.last_strokes.push_front(PosTimeDict {
            pos: element.pos,
            distance_to_previous: 0.0,
            time: now,
        });
        self.distance = 0.0;
        self.total_distance = 0.0;
    }

    fn add_event(&mut self, element: Element, now: Instant) {
        // add event to the front of the vecdeque
        let latest_pos = self.last_strokes.front().unwrap().pos;
        let dist_delta = latest_pos.metric_distance(&element.pos);

        self.last_strokes.push_front(PosTimeDict {
            pos: element.pos,
            distance_to_previous: dist_delta,
            time: now,
        });
        self.distance += dist_delta;

        println!("adding {:?}", dist_delta);

        self.total_distance += dist_delta;

        while self.last_strokes.back().is_some()
            && self.last_strokes.back().unwrap().time
                < now - Duration::from_secs_f64(Brush::LONGPRESS_TIMEOUT)
        {
            // remove the last element
            let back_element = self.last_strokes.pop_back().unwrap();
            self.distance -= back_element.distance_to_previous;
            println!("removing {:?}", back_element.distance_to_previous);
        }
        // println!("last stroke vecdeque {:?}", self.last_strokes);
    }

    pub fn get_latest_pos(&self) -> na::Vector2<f64> {
        self.last_strokes.front().unwrap().pos
    }
}

#[derive(Debug)]
pub struct Brush {
    state: BrushState,
    /// save the current path for recognition one level upper
    pub pen_path_recognition: Option<PenPath>,
    /// handle for the separate task that makes it possible to
    /// trigger long press for input with no jitter (where a long press
    /// hold wouldn't trigger any new event)
    longpress_handle: Option<crate::tasks::OneOffTaskHandle>,
    /// stroke key in progress when a long press occurs
    pub current_stroke_key: Option<StrokeKey>,
    /// save the start position for the current stroke
    /// This prevents long press from happening on a point
    /// We create a deadzone around the start position
    pub start_position: Option<PosTimeDict>,
    pub stroke_width: Option<f64>,
    pub long_press_detector: LongPressDetector,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            state: BrushState::Idle,
            pen_path_recognition: None,
            current_stroke_key: None,
            longpress_handle: None,
            start_position: None,
            stroke_width: None,
            long_press_detector: LongPressDetector::default(),
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

                    self.stroke_width =
                        Some(engine_view.pens_config.brush_config.get_stroke_width());

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

                    self.start_position = Some(PosTimeDict {
                        pos: element.pos,
                        distance_to_previous: 0.0,
                        time: now,
                    });
                    let tasks_tx = engine_view.tasks_tx.clone();
                    self.longpress_handle = Some(crate::tasks::OneOffTaskHandle::new(
                        move || tasks_tx.send(EngineTask::LongPressStatic),
                        Duration::from_secs_f64(Self::LONGPRESS_TIMEOUT),
                    ));
                    self.long_press_detector.reset(element, now);

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
                self.current_stroke_key = None;
                self.pen_path_recognition = None;
                self.start_position = None;
                self.long_press_detector.clear();
                self.cancel_handle_long_press();

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
                    path_builder.handle_event(pen_event.clone(), now, Constraints::default());
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

                        // first send the event:
                        if let Some(handle) = self.longpress_handle.as_mut() {
                            let _ = handle.reset_timeout();
                            // we may have asusmed wrongly the type of pen event ?
                            // could be a key press that's ignored ? KeyPressed
                            // or Text ?
                            match pen_event {
                                PenEvent::Down { element, .. } => {
                                    self.long_press_detector.add_event(element, now)
                                }
                                _ => (),
                            }
                        } else {
                            // recreate the handle if it was dropped
                            // this happens when we sent a long_hold event and cancelled the long
                            // press.
                            // We have to restart he handle and the long press detector
                            let tasks_tx = engine_view.tasks_tx.clone();
                            self.longpress_handle = Some(crate::tasks::OneOffTaskHandle::new(
                                move || tasks_tx.send(EngineTask::LongPressStatic),
                                Duration::from_secs_f64(Self::LONGPRESS_TIMEOUT),
                            ));

                            match pen_event {
                                PenEvent::Down { element, .. } => {
                                    self.start_position = Some(PosTimeDict {
                                        pos: element.pos,
                                        distance_to_previous: 0.0,
                                        time: now,
                                    });
                                    self.current_stroke_key = None;
                                    self.pen_path_recognition = None;

                                    self.long_press_detector.reset(element, now);
                                }
                                _ => {
                                    // we are drawing only if the pen is down...
                                }
                            }
                        }
                        // then test : long press ?
                        let is_deadzone = self.long_press_detector.total_distance()
                            > 4.0 * self.stroke_width.unwrap_or(0.5);
                        let is_static =
                            self.long_press_detector.distance() < 4.0 * self.stroke_width.unwrap();
                        let time_delta =
                            now - self.start_position.unwrap_or(PosTimeDict::default()).time;

                        println!("static distance  {:?}", self.long_press_detector.distance());
                        println!(
                            "deadzone : {:?}, static {:?}, {:?}",
                            is_deadzone, is_static, time_delta
                        );

                        if time_delta > Duration::from_secs_f64(Self::LONGPRESS_TIMEOUT)
                            && is_static
                            && is_deadzone
                        {
                            // save the current stroke for recognition
                            println!("saving the current stroke data");

                            //save the key for potentially deleting it and replacing it with a shape
                            self.current_stroke_key = Some(current_stroke_key.clone());

                            // save the current stroke for recognition
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_ref(*current_stroke_key)
                            {
                                let path = brushstroke.path.clone();
                                self.pen_path_recognition = Some(path);
                            }

                            widget_flags.long_hold = true;

                            // quit the handle. Either recognition is successful and we are right
                            // or we aren't and a new handle will be create on the next event
                            self.cancel_handle_long_press();
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
                        self.cancel_handle_long_press();

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
    const LONGPRESS_TIMEOUT: f64 = 0.5;

    pub fn cancel_handle_long_press(&mut self) {
        // cancel the long press handle
        if let Some(handle) = self.longpress_handle.as_mut() {
            let _ = handle.quit();
        }
        self.longpress_handle = None;
    }

    pub fn reset_long_press(&mut self, element: Element, now: Instant) {
        self.start_position = None;
        self.current_stroke_key = None;
        self.cancel_handle_long_press();
        self.longpress_handle = None;
        self.stroke_width = None;
        self.long_press_detector.reset(element, now);
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
