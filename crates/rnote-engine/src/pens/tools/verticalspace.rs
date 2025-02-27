// Imports
use super::ToolsState;
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::eventresult::EventPropagation;
use rnote_compose::ext::{AabbExt, Vector2Ext};
use rnote_compose::penevent::PenProgress;
use rnote_compose::{EventResult, PenEvent, color};
use std::time::Instant;

#[derive(Clone, Debug)]
pub(super) struct VerticalSpaceTool {
    state: ToolsState,
    start_pos_y: f64,
    pos_y: f64,
    limit_x: Option<(f64, f64)>,
    strokes_below: Vec<StrokeKey>,
}

impl Default for VerticalSpaceTool {
    fn default() -> Self {
        Self {
            state: ToolsState::default(),
            start_pos_y: 0.0,
            pos_y: 0.0,
            limit_x: None,
            strokes_below: vec![],
        }
    }
}

impl VerticalSpaceTool {
    const Y_OFFSET_THRESHOLD: f64 = 0.1;
    const SNAP_START_POS_DIST: f64 = 10.;
    const OFFSET_LINE_COLOR: piet::Color = color::GNOME_BLUES[3];
    const THRESHOLD_LINE_WIDTH: f64 = 3.0;
    const THRESHOLD_LINE_DASH_PATTERN: [f64; 2] = [9.0, 6.0];
    const OFFSET_LINE_WIDTH: f64 = 1.5;
    const FILL_COLOR: piet::Color = color::GNOME_BRIGHTS[2].with_a8(23);
    const THRESHOLD_LINE_COLOR: piet::Color = color::GNOME_GREENS[4].with_a8(240);

    pub(super) fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let result = match (&mut self.state, event) {
            (ToolsState::Idle, PenEvent::Down { element, .. }) => {
                self.start_pos_y = element.pos[1];
                self.pos_y = element.pos[1];
                let pos_x = element.pos[0];
                let limit_movement_horizontal_borders = engine_view
                    .pens_config
                    .tools_config
                    .verticalspace_tool_config
                    .limit_movement_horizontal_borders;
                let limit_movement_vertical_borders = engine_view
                    .pens_config
                    .tools_config
                    .verticalspace_tool_config
                    .limit_movement_vertical_borders;
                let y_max = ((self.pos_y / engine_view.document.format.height()).floor() + 1.0f64)
                    * engine_view.document.format.height();
                let limit_x = {
                    let page_number_hor = (pos_x / engine_view.document.format.width()).floor();
                    (
                        page_number_hor * engine_view.document.format.width(),
                        (page_number_hor + 1.0f64) * engine_view.document.format.width(),
                    )
                };
                self.limit_x = if limit_movement_vertical_borders {
                    Some(limit_x)
                } else {
                    None
                };
                self.strokes_below = engine_view.store.keys_between(
                    self.pos_y,
                    y_max,
                    limit_x,
                    limit_movement_vertical_borders,
                    limit_movement_horizontal_borders,
                );
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
                let y_offset = if (element.pos[1] - self.start_pos_y).abs()
                    < VerticalSpaceTool::SNAP_START_POS_DIST
                {
                    self.start_pos_y - self.pos_y
                } else {
                    engine_view
                        .document
                        .snap_position(element.pos - na::vector![0., self.pos_y])[1]
                };

                if y_offset.abs() > VerticalSpaceTool::Y_OFFSET_THRESHOLD {
                    engine_view
                        .store
                        .translate_strokes(&self.strokes_below, na::vector![0.0, y_offset]);
                    engine_view
                        .store
                        .translate_strokes_images(&self.strokes_below, na::vector![0.0, y_offset]);
                    self.pos_y += y_offset;

                    widget_flags.store_modified = true;
                }

                // possibly nudge camera
                widget_flags |= engine_view
                    .camera
                    .nudge_w_pos(element.pos, engine_view.document);
                widget_flags |= engine_view
                    .document
                    .expand_autoexpand(engine_view.camera, engine_view.store);
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (ToolsState::Active, PenEvent::Up { .. }) => {
                engine_view
                    .store
                    .update_geometry_for_strokes(&self.strokes_below);

                widget_flags |= engine_view.store.record(Instant::now());
                widget_flags.store_modified = true;

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
        self.start_pos_y = 0.0;
        self.pos_y = 0.0;
        self.state = ToolsState::Idle;
    }
}

impl DrawableOnDoc for VerticalSpaceTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        if matches!(self.state, ToolsState::Idle) {
            return None;
        }

        let viewport = engine_view.camera.viewport();

        let x = viewport.mins[0];
        let y = self.start_pos_y;
        let width = viewport.extents()[0];
        let height = self.pos_y - self.start_pos_y;
        let tool_bounds = Aabb::new_positive(na::point![x, y], na::point![x + width, y + height]);

        Some(tool_bounds)
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let total_zoom = engine_view.camera.total_zoom();
        let viewport = engine_view.camera.viewport();
        let x = if self.limit_x.is_some() {
            viewport.mins[0].max(self.limit_x.unwrap().0)
        } else {
            viewport.mins[0]
        };
        let y = self.start_pos_y;
        let width = if self.limit_x.is_some() {
            self.limit_x.unwrap().1 - viewport.mins[0].max(self.limit_x.unwrap().0)
        } else {
            viewport.extents()[0]
        };
        let height = self.pos_y - self.start_pos_y;
        let tool_bounds = Aabb::new_positive(na::point![x, y], na::point![x + width, y + height]);

        let tool_bounds_rect = kurbo::Rect::from_points(
            tool_bounds.mins.coords.to_kurbo_point(),
            tool_bounds.maxs.coords.to_kurbo_point(),
        );
        cx.fill(tool_bounds_rect, &Self::FILL_COLOR);

        let threshold_line =
            kurbo::Line::new(kurbo::Point::new(x, y), kurbo::Point::new(x + width, y));
        cx.stroke_styled(
            threshold_line,
            &Self::THRESHOLD_LINE_COLOR,
            Self::THRESHOLD_LINE_WIDTH / total_zoom,
            &piet::StrokeStyle::new().dash_pattern(&Self::THRESHOLD_LINE_DASH_PATTERN),
        );

        let offset_line = kurbo::Line::new(
            kurbo::Point::new(x, y + height),
            kurbo::Point::new(x + width, y + height),
        );
        cx.stroke(
            offset_line,
            &Self::OFFSET_LINE_COLOR,
            Self::OFFSET_LINE_WIDTH / total_zoom,
        );
        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
