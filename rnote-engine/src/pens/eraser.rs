use std::time::Instant;

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::pensconfig::eraserconfig::EraserStyle;
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawOnDocBehaviour, WidgetFlags};
use once_cell::sync::Lazy;
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::helpers::AabbHelpers;
use rnote_compose::penevents::PenEvent;
use rnote_compose::penpath::Element;

use p2d::bounding_volume::{Aabb, BoundingVolume};

#[derive(Debug, Clone, Copy)]
pub enum EraserState {
    Up,
    Proximity(Element),
    Down(Element),
}

#[derive(Clone, Debug)]
pub struct Eraser {
    pub(crate) state: EraserState,
}

impl Default for Eraser {
    fn default() -> Self {
        Self {
            state: EraserState::Up,
        }
    }
}

impl PenBehaviour for Eraser {
    fn style(&self) -> PenStyle {
        PenStyle::Eraser
    }

    fn update_state(&mut self, _engine_view: &mut EngineViewMut) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (
                EraserState::Up | EraserState::Proximity { .. },
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                widget_flags.merge(engine_view.store.record(Instant::now()));

                match &engine_view.pens_config.eraser_config.style {
                    EraserStyle::TrashCollidingStrokes => {
                        widget_flags.merge(engine_view.store.trash_colliding_strokes(
                            engine_view.pens_config.eraser_config.eraser_bounds(element),
                            engine_view.camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = engine_view.store.split_colliding_strokes(
                            engine_view.pens_config.eraser_config.eraser_bounds(element),
                            engine_view.camera.viewport(),
                        );

                        engine_view.store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                    }
                }

                self.state = EraserState::Down(element);

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
            (EraserState::Up | EraserState::Down { .. }, PenEvent::Proximity { element, .. }) => {
                self.state = EraserState::Proximity(element);
                widget_flags.redraw = true;

                PenProgress::Idle
            }
            (
                EraserState::Up,
                PenEvent::KeyPressed { .. } | PenEvent::Up { .. } | PenEvent::Cancel,
            ) => PenProgress::Idle,
            (EraserState::Down(current_element), PenEvent::Down { element, .. }) => {
                match &engine_view.pens_config.eraser_config.style {
                    EraserStyle::TrashCollidingStrokes => {
                        widget_flags.merge(engine_view.store.trash_colliding_strokes(
                            engine_view.pens_config.eraser_config.eraser_bounds(element),
                            engine_view.camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = engine_view.store.split_colliding_strokes(
                            engine_view.pens_config.eraser_config.eraser_bounds(element),
                            engine_view.camera.viewport(),
                        );

                        engine_view.store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                    }
                }

                *current_element = element;

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
            (EraserState::Down { .. }, PenEvent::Up { element, .. }) => {
                match &engine_view.pens_config.eraser_config.style {
                    EraserStyle::TrashCollidingStrokes => {
                        widget_flags.merge(engine_view.store.trash_colliding_strokes(
                            engine_view.pens_config.eraser_config.eraser_bounds(element),
                            engine_view.camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = engine_view.store.split_colliding_strokes(
                            engine_view.pens_config.eraser_config.eraser_bounds(element),
                            engine_view.camera.viewport(),
                        );

                        engine_view.store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );
                    }
                }

                self.state = EraserState::Up;

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::Finished
            }
            (EraserState::Down(_), PenEvent::KeyPressed { .. }) => PenProgress::InProgress,
            (EraserState::Proximity(_), PenEvent::Up { .. }) => {
                self.state = EraserState::Up;
                widget_flags.redraw = true;

                PenProgress::Idle
            }
            (EraserState::Proximity(current_element), PenEvent::Proximity { element, .. }) => {
                *current_element = element;
                widget_flags.redraw = true;

                PenProgress::Idle
            }
            (EraserState::Proximity { .. } | EraserState::Down { .. }, PenEvent::Cancel) => {
                self.state = EraserState::Up;

                widget_flags.redraw = true;

                PenProgress::Finished
            }
            (EraserState::Proximity(_), PenEvent::KeyPressed { .. }) => PenProgress::Idle,
            (EraserState::Up, PenEvent::Text { .. }) => PenProgress::Idle,
            (EraserState::Proximity(_), PenEvent::Text { .. }) => PenProgress::Idle,
            (EraserState::Down(_), PenEvent::Text { .. }) => PenProgress::InProgress,
        };

        (pen_progress, widget_flags)
    }
}

impl DrawOnDocBehaviour for Eraser {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        match &self.state {
            EraserState::Up => None,
            EraserState::Proximity(current_element) | EraserState::Down(current_element) => Some(
                engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(*current_element),
            ),
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        static OUTLINE_COLOR: Lazy<piet::Color> =
            Lazy::new(|| color::GNOME_REDS[2].with_alpha(0.941));
        static FILL_COLOR: Lazy<piet::Color> = Lazy::new(|| color::GNOME_REDS[0].with_alpha(0.627));
        static PROXIMITY_FILL_COLOR: Lazy<piet::Color> =
            Lazy::new(|| color::GNOME_REDS[0].with_alpha(0.5));
        let outline_width = 2.0 / engine_view.camera.total_zoom();

        match &self.state {
            EraserState::Up => {}
            EraserState::Proximity(current_element) => {
                let bounds = engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(*current_element);

                let fill_rect = bounds.to_kurbo_rect();
                let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

                cx.fill(fill_rect, &*PROXIMITY_FILL_COLOR);
                cx.stroke(outline_rect, &*OUTLINE_COLOR, outline_width);
            }
            EraserState::Down(current_element) => {
                let bounds = engine_view
                    .pens_config
                    .eraser_config
                    .eraser_bounds(*current_element);

                let fill_rect = bounds.to_kurbo_rect();
                let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

                cx.fill(fill_rect, &*FILL_COLOR);
                cx.stroke(outline_rect, &*OUTLINE_COLOR, outline_width);
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
