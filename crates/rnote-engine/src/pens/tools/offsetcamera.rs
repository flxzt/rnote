// Imports
use super::ToolsState;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::eventresult::EventPropagation;
use rnote_compose::ext::Vector2Ext;
use rnote_compose::penevent::PenProgress;
use rnote_compose::{color, EventResult, PenEvent};
use std::time::Instant;

#[derive(Clone, Debug)]
pub(super) struct OffsetCameraTool {
    state: ToolsState,
    start: na::Vector2<f64>,
}

impl Default for OffsetCameraTool {
    fn default() -> Self {
        Self {
            state: ToolsState::default(),
            start: na::Vector2::zeros(),
        }
    }
}

impl OffsetCameraTool {
    const CURSOR_SIZE: na::Vector2<f64> = na::vector![16.0, 16.0];
    const CURSOR_STROKE_WIDTH: f64 = 2.0;
    const CURSOR_PATH: &'static str = "m 8 1.078125 l -3 3 h 2 v 2.929687 h -2.960938 v -2 l -3 3 l 3 3 v -2 h 2.960938 v 2.960938 h -2 l 3 3 l 3 -3 h -2 v -2.960938 h 3.054688 v 2 l 3 -3 l -3 -3 v 2 h -3.054688 v -2.929687 h 2 z m 0 0";
    const DARK_COLOR: piet::Color = color::GNOME_DARKS[3].with_a8(240);
    const LIGHT_COLOR: piet::Color = color::GNOME_BRIGHTS[1].with_a8(240);

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
                let offset = engine_view
                    .camera
                    .transform()
                    .transform_point(&element.pos.into())
                    .coords
                    - engine_view
                        .camera
                        .transform()
                        .transform_point(&self.start.into())
                        .coords;

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
        self.start = na::Vector2::zeros();
        self.state = ToolsState::Idle;
    }
}

impl DrawableOnDoc for OffsetCameraTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        Some(Aabb::from_half_extents(
            self.start.into(),
            ((Self::CURSOR_SIZE + na::Vector2::repeat(Self::CURSOR_STROKE_WIDTH)) * 0.5)
                / engine_view.camera.total_zoom(),
        ))
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        if let Some(bounds) = self.bounds_on_doc(engine_view) {
            cx.transform(kurbo::Affine::translate(bounds.mins.coords.to_kurbo_vec()));
            cx.transform(kurbo::Affine::scale(1.0 / engine_view.camera.total_zoom()));

            let bez_path = kurbo::BezPath::from_svg(Self::CURSOR_PATH).unwrap();

            cx.stroke(
                bez_path.clone(),
                &Self::LIGHT_COLOR,
                Self::CURSOR_STROKE_WIDTH,
            );
            cx.fill(bez_path, &Self::DARK_COLOR);
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
