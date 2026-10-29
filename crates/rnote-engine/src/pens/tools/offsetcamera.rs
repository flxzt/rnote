// Imports
use super::ToolsState;
use crate::engine::{EngineView, EngineViewMut};
use crate::{Previewable, WidgetFlags};
use kurbo::Stroke;
use p2d::bounding_volume::Aabb;
use p2d::math::Vector2;
use rnote_compose::eventresult::EventPropagation;
use rnote_compose::ext::Vector2Ext;
use rnote_compose::penevent::PenProgress;
use rnote_compose::{Color, EventResult, PenEvent, color};
use std::time::Instant;
use vello_cpu::RenderContext;

#[derive(Clone, Debug)]
pub(super) struct OffsetCameraTool {
    state: ToolsState,
    start: Vector2,
}

impl Default for OffsetCameraTool {
    fn default() -> Self {
        Self {
            state: ToolsState::default(),
            start: Vector2::ZERO,
        }
    }
}

impl OffsetCameraTool {
    const CURSOR_SIZE: Vector2 = Vector2::splat(16.);
    const CURSOR_STROKE_WIDTH: f64 = 2.0;
    const CURSOR_PATH: &'static str = "m 8 1.078125 l -3 3 h 2 v 2.929687 h -2.960938 v -2 l -3 3 l 3 3 v -2 h 2.960938 v 2.960938 h -2 l 3 3 l 3 -3 h -2 v -2.960938 h 3.054688 v 2 l 3 -3 l -3 -3 v 2 h -3.054688 v -2.929687 h 2 z m 0 0";
    const DARK_COLOR: Color = color::GNOME_DARKS[3].with_a8(240);
    const LIGHT_COLOR: Color = color::GNOME_BRIGHTS[1].with_a8(240);

    pub(super) fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let result = match (&mut self.state, event) {
            (ToolsState::Idle, PenEvent::Down { element, .. }) => {
                self.start = element.pos;
                widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);
                self.state = ToolsState::Active;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (ToolsState::Idle, _) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (ToolsState::Active, PenEvent::Down { element, .. }) => {
                let offset = engine_view.camera.transform().transform_point2(element.pos)
                    - engine_view.camera.transform().transform_point2(self.start);

                widget_flags |= engine_view
                    .camera
                    .set_offset(engine_view.camera.offset() - offset, engine_view.document);
                widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (ToolsState::Active, PenEvent::Up { .. }) => {
                widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.reset();

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (ToolsState::Active, PenEvent::Proximity { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            (ToolsState::Active, PenEvent::KeyPressed { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            (ToolsState::Active, PenEvent::Text { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            (ToolsState::Active, PenEvent::Cancel) => {
                widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.reset();

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
        };
        (result, widget_flags)
    }

    fn reset(&mut self) {
        self.start = Vector2::ZERO;
        self.state = ToolsState::Idle;
    }
}

impl Previewable for OffsetCameraTool {
    fn preview_bounds(&self, engine_view: &EngineView) -> Option<Aabb> {
        if matches!(self.state, ToolsState::Idle) {
            return None;
        }

        Some(Aabb::from_half_extents(
            self.start,
            ((Self::CURSOR_SIZE + Vector2::splat(Self::CURSOR_STROKE_WIDTH)) * 0.5)
                / engine_view.camera.total_zoom(),
        ))
    }

    fn preview_draw(&self, cx: &mut RenderContext, engine_view: &EngineView) -> anyhow::Result<()> {
        if let Some(bounds) = self.preview_bounds(engine_view) {
            let state = cx.save_current_state();

            let bez_path = kurbo::BezPath::from_svg(Self::CURSOR_PATH).unwrap();

            cx.set_transform(
                kurbo::Affine::scale(1.0 / engine_view.camera.total_zoom())
                    * kurbo::Affine::translate(bounds.mins.to_kurbo_vec()),
            );
            cx.set_paint(Self::LIGHT_COLOR);
            cx.set_stroke(Stroke::new(Self::CURSOR_STROKE_WIDTH));
            cx.stroke_path(&bez_path);
            cx.set_paint(Self::DARK_COLOR);
            cx.fill_path(&bez_path);

            cx.restore_state(state);
        }

        Ok(())
    }
}
