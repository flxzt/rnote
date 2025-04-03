// Imports
use super::ToolsState;
use crate::engine::{EngineView, EngineViewMut};
use crate::{Camera, DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::eventresult::EventPropagation;
use rnote_compose::ext::{AabbExt, Vector2Ext};
use rnote_compose::penevent::PenProgress;
use rnote_compose::{EventResult, PenEvent, color};
use std::time::Instant;

#[derive(Clone, Debug)]
pub(super) struct ZoomTool {
    state: ToolsState,
    start_surface_coord: na::Vector2<f64>,
    current_surface_coord: na::Vector2<f64>,
}

impl Default for ZoomTool {
    fn default() -> Self {
        Self {
            state: ToolsState::default(),
            start_surface_coord: na::Vector2::zeros(),
            current_surface_coord: na::Vector2::zeros(),
        }
    }
}

impl ZoomTool {
    const CURSOR_RADIUS: f64 = 4.0;
    const CURSOR_STROKE_WIDTH: f64 = 2.0;
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
                self.start_surface_coord = engine_view
                    .camera
                    .transform()
                    .transform_point(&element.pos.into())
                    .coords;
                self.current_surface_coord = engine_view
                    .camera
                    .transform()
                    .transform_point(&element.pos.into())
                    .coords;
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
                let total_zoom_old = engine_view.camera.total_zoom();
                let camera_offset = engine_view.camera.offset();

                let new_surface_coord = engine_view
                    .camera
                    .transform()
                    .transform_point(&element.pos.into())
                    .coords;

                let offset = new_surface_coord - self.current_surface_coord;

                // Drag down zooms out, drag up zooms in
                let new_zoom =
                    total_zoom_old * (1.0 - offset[1] * Camera::DRAG_ZOOM_MAGN_ZOOM_FACTOR);

                if (Camera::ZOOM_MIN..=Camera::ZOOM_MAX).contains(&new_zoom) {
                    widget_flags |= engine_view
                        .camera
                        .zoom_w_timeout(new_zoom, engine_view.tasks_tx.clone());

                    // Translate the camera view so that the start_surface_coord has the same surface position
                    // as before the zoom occurred
                    let new_camera_offset =
                        (((camera_offset + self.start_surface_coord) / total_zoom_old) * new_zoom)
                            - self.start_surface_coord;
                    widget_flags |= engine_view
                        .camera
                        .set_offset(new_camera_offset, engine_view.document);

                    widget_flags |= engine_view
                        .document
                        .expand_autoexpand(engine_view.camera, engine_view.store);
                }
                self.current_surface_coord = new_surface_coord;

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
        self.start_surface_coord = na::Vector2::zeros();
        self.current_surface_coord = na::Vector2::zeros();
        self.state = ToolsState::Idle;
    }
}

impl DrawableOnDoc for ZoomTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        if matches!(self.state, ToolsState::Idle) {
            return None;
        }

        let start_circle_center = engine_view
            .camera
            .transform()
            .inverse()
            .transform_point(&self.start_surface_coord.into());
        let current_circle_center = engine_view
            .camera
            .transform()
            .inverse()
            .transform_point(&self.current_surface_coord.into());

        Some(
            Aabb::new_positive(start_circle_center, current_circle_center).extend_by(
                na::Vector2::repeat(Self::CURSOR_RADIUS + Self::CURSOR_STROKE_WIDTH * 0.5)
                    / engine_view.camera.total_zoom(),
            ),
        )
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let total_zoom = engine_view.camera.total_zoom();

        let start_circle_center = engine_view
            .camera
            .transform()
            .inverse()
            .transform_point(&self.start_surface_coord.into())
            .coords
            .to_kurbo_point();
        let current_circle_center = engine_view
            .camera
            .transform()
            .inverse()
            .transform_point(&self.current_surface_coord.into())
            .coords
            .to_kurbo_point();

        // start circle
        cx.fill(
            kurbo::Circle::new(start_circle_center, Self::CURSOR_RADIUS * 0.8 / total_zoom),
            &Self::LIGHT_COLOR,
        );
        cx.fill(
            kurbo::Circle::new(start_circle_center, Self::CURSOR_RADIUS * 0.6 / total_zoom),
            &Self::DARK_COLOR,
        );

        // current circle
        cx.stroke(
            kurbo::Circle::new(current_circle_center, Self::CURSOR_RADIUS / total_zoom),
            &Self::LIGHT_COLOR,
            Self::CURSOR_STROKE_WIDTH / total_zoom,
        );
        cx.stroke(
            kurbo::Circle::new(current_circle_center, Self::CURSOR_RADIUS / total_zoom),
            &Self::DARK_COLOR,
            Self::CURSOR_STROKE_WIDTH * 0.7 / total_zoom,
        );

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
