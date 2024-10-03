// Imports
use super::pensconfig::eraserconfig::EraserStyle;
use super::PenBehaviour;
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::ext::AabbExt;
use rnote_compose::penevent::{PenEvent, PenProgress};
use rnote_compose::penpath::Element;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum EraserState {
    Up,
    Proximity(Element),
    Down(Element),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EraserMotion {
    last_element: Option<(Element, Instant)>,
    pub speed: f64,
}

impl EraserMotion {
    pub const SMOOTHING_FACTOR: f64 = 3.0;
    pub const SPEED_LIMIT: f64 = 10000.0;

    fn update(&mut self, element: Element, time: Instant) {
        if let Some((last_element, last_element_time)) = self.last_element {
            let delta = element.pos - last_element.pos;
            let delta_time = time - last_element_time;
            let new_speed = delta.norm() / delta_time.as_secs_f64();
            self.speed = Self::SPEED_LIMIT.min(
                (self.speed * Self::SMOOTHING_FACTOR + new_speed) / (Self::SMOOTHING_FACTOR + 1.0),
            );
        }
        self.last_element = Some((element, time));
    }

    fn reset(&mut self) {
        self.last_element = None;
        self.speed = 0.0;
    }
}

#[derive(Clone, Debug)]
pub struct Eraser {
    pub(crate) state: EraserState,
    pub(crate) motion: EraserMotion,
}

impl Default for Eraser {
    fn default() -> Self {
        Self {
            state: EraserState::Up,
            motion: EraserMotion::default(),
        }
    }
}

impl PenBehaviour for Eraser {
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn style(&self) -> PenStyle {
        PenStyle::Eraser
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
            (EraserState::Up | EraserState::Proximity { .. }, PenEvent::Down { element, .. }) => {
                self.motion.update(element, now);
                widget_flags |= erase(element, self.motion.speed, engine_view);
                self.state = EraserState::Down(element);
                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (EraserState::Up | EraserState::Down { .. }, PenEvent::Proximity { element, .. }) => {
                self.motion.update(element, now);
                self.state = EraserState::Proximity(element);
                EventResult {
                    handled: false,
                    propagate: EventPropagation::Proceed,
                    progress: PenProgress::Idle,
                }
            }
            (
                EraserState::Up,
                PenEvent::KeyPressed { .. } | PenEvent::Up { .. } | PenEvent::Cancel,
            ) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (EraserState::Down(current_element), PenEvent::Down { element, .. }) => {
                self.motion.update(element, now);
                widget_flags |= erase(element, self.motion.speed, engine_view);
                *current_element = element;
                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (EraserState::Down { .. }, PenEvent::Up { element, .. }) => {
                self.motion.reset();
                widget_flags |= erase(element, self.motion.speed, engine_view)
                    | engine_view.store.record(Instant::now());
                self.state = EraserState::Up;
                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (EraserState::Down(_), PenEvent::KeyPressed { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            (EraserState::Proximity(_), PenEvent::Up { .. }) => {
                self.motion.reset();
                self.state = EraserState::Up;
                EventResult {
                    handled: false,
                    propagate: EventPropagation::Proceed,
                    progress: PenProgress::Idle,
                }
            }
            (EraserState::Proximity(current_element), PenEvent::Proximity { element, .. }) => {
                self.motion.update(element, now);
                *current_element = element;
                EventResult {
                    handled: false,
                    propagate: EventPropagation::Proceed,
                    progress: PenProgress::Idle,
                }
            }
            (EraserState::Proximity { .. } | EraserState::Down { .. }, PenEvent::Cancel) => {
                self.state = EraserState::Up;
                widget_flags |= engine_view.store.record(Instant::now());
                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (EraserState::Proximity(_), PenEvent::KeyPressed { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (EraserState::Up, PenEvent::Text { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (EraserState::Proximity(_), PenEvent::Text { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (EraserState::Down(_), PenEvent::Text { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
        };

        (event_result, widget_flags)
    }
}

impl DrawableOnDoc for Eraser {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        match &self.state {
            EraserState::Up => None,
            EraserState::Proximity(current_element) | EraserState::Down(current_element) => Some(
                engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(*current_element, self.motion.speed),
            ),
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[2].with_a8(240);
        const FILL_COLOR: piet::Color = color::GNOME_REDS[0].with_a8(160);
        const PROXIMITY_FILL_COLOR: piet::Color = color::GNOME_REDS[0].with_a8(51);
        let outline_width = 2.0 / engine_view.camera.total_zoom();

        match &self.state {
            EraserState::Up => {}
            EraserState::Proximity(current_element) => {
                let bounds = engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(*current_element, self.motion.speed);

                let fill_rect = bounds.to_kurbo_rect();
                let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

                cx.fill(fill_rect, &PROXIMITY_FILL_COLOR);
                cx.stroke(outline_rect, &OUTLINE_COLOR, outline_width);
            }
            EraserState::Down(current_element) => {
                let bounds = engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(*current_element, self.motion.speed);

                let fill_rect = bounds.to_kurbo_rect();
                let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

                cx.fill(fill_rect, &FILL_COLOR);
                cx.stroke(outline_rect, &OUTLINE_COLOR, outline_width);
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

fn erase(element: Element, speed: f64, engine_view: &mut EngineViewMut) -> WidgetFlags {
    // the widget_flags.store_modified flag is set in the `.trash_..()` methods
    let mut widget_flags = WidgetFlags::default();

    match &engine_view.pens_config.eraser_config.style {
        EraserStyle::TrashCollidingStrokes => {
            widget_flags |= engine_view.store.trash_colliding_strokes(
                engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(element, speed),
                engine_view.camera.viewport(),
            );
        }
        EraserStyle::SplitCollidingStrokes => {
            let (modified_strokes, wf) = engine_view.store.split_colliding_strokes(
                engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(element, speed),
                engine_view.camera.viewport(),
            );
            widget_flags |= wf;

            engine_view.store.regenerate_rendering_for_strokes(
                &modified_strokes,
                engine_view.camera.viewport(),
                engine_view.camera.image_scale(),
            );
        }
    }

    widget_flags
}
