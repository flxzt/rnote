// Imports
use super::pensconfig::toolsconfig::ToolStyle;
use super::PenBehaviour;
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::{Camera, DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::ext::{AabbExt, Vector2Ext};
use rnote_compose::penevent::{PenEvent, PenProgress};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct VerticalSpaceTool {
    start_pos_y: f64,
    pos_y: f64,
    strokes_below: Vec<StrokeKey>,
}

impl Default for VerticalSpaceTool {
    fn default() -> Self {
        Self {
            start_pos_y: 0.0,
            pos_y: 0.0,
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
}

impl DrawableOnDoc for VerticalSpaceTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
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
        let x = viewport.mins[0];
        let y = self.start_pos_y;
        let width = viewport.extents()[0];
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

#[derive(Clone, Debug)]
pub struct OffsetCameraTool {
    pub start: na::Vector2<f64>,
}

impl Default for OffsetCameraTool {
    fn default() -> Self {
        Self {
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

#[derive(Clone, Debug)]
pub struct ZoomTool {
    pub start_surface_coord: na::Vector2<f64>,
    pub current_surface_coord: na::Vector2<f64>,
}

impl Default for ZoomTool {
    fn default() -> Self {
        Self {
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
}

impl DrawableOnDoc for ZoomTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
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

#[derive(Debug, Clone, Copy)]
enum ToolsState {
    Idle,
    Active,
}

impl Default for ToolsState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug, Default)]
pub struct Tools {
    pub verticalspace_tool: VerticalSpaceTool,
    pub offsetcamera_tool: OffsetCameraTool,
    pub zoom_tool: ZoomTool,
    state: ToolsState,
}

impl PenBehaviour for Tools {
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn style(&self) -> PenStyle {
        PenStyle::Tools
    }

    fn update_state(&mut self, _engine_view: &mut EngineViewMut) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let event_result = match (&mut self.state, event) {
            (ToolsState::Idle, PenEvent::Down { element, .. }) => {
                match engine_view.pens_config.tools_config.style {
                    ToolStyle::VerticalSpace => {
                        self.verticalspace_tool.start_pos_y = element.pos[1];
                        self.verticalspace_tool.pos_y = element.pos[1];

                        self.verticalspace_tool.strokes_below = engine_view
                            .store
                            .keys_below_y(self.verticalspace_tool.pos_y);
                    }
                    ToolStyle::OffsetCamera => {
                        self.offsetcamera_tool.start = element.pos;
                    }
                    ToolStyle::Zoom => {
                        self.zoom_tool.start_surface_coord = engine_view
                            .camera
                            .transform()
                            .transform_point(&element.pos.into())
                            .coords;
                        self.zoom_tool.current_surface_coord = engine_view
                            .camera
                            .transform()
                            .transform_point(&element.pos.into())
                            .coords;
                    }
                }
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
                match engine_view.pens_config.tools_config.style {
                    ToolStyle::VerticalSpace => {
                        let y_offset = if (element.pos[1] - self.verticalspace_tool.start_pos_y)
                            .abs()
                            < VerticalSpaceTool::SNAP_START_POS_DIST
                        {
                            self.verticalspace_tool.start_pos_y - self.verticalspace_tool.pos_y
                        } else {
                            engine_view.document.snap_position(
                                element.pos - na::vector![0., self.verticalspace_tool.pos_y],
                            )[1]
                        };

                        if y_offset.abs() > VerticalSpaceTool::Y_OFFSET_THRESHOLD {
                            engine_view.store.translate_strokes(
                                &self.verticalspace_tool.strokes_below,
                                na::vector![0.0, y_offset],
                            );
                            engine_view.store.translate_strokes_images(
                                &self.verticalspace_tool.strokes_below,
                                na::vector![0.0, y_offset],
                            );
                            self.verticalspace_tool.pos_y += y_offset;

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
                    }
                    ToolStyle::OffsetCamera => {
                        let offset = engine_view
                            .camera
                            .transform()
                            .transform_point(&element.pos.into())
                            .coords
                            - engine_view
                                .camera
                                .transform()
                                .transform_point(&self.offsetcamera_tool.start.into())
                                .coords;

                        widget_flags |= engine_view
                            .camera
                            .set_offset(engine_view.camera.offset() - offset, engine_view.document);
                        widget_flags |= engine_view
                            .document
                            .resize_autoexpand(engine_view.store, engine_view.camera);
                    }
                    ToolStyle::Zoom => {
                        let total_zoom_old = engine_view.camera.total_zoom();
                        let camera_offset = engine_view.camera.offset();

                        let new_surface_coord = engine_view
                            .camera
                            .transform()
                            .transform_point(&element.pos.into())
                            .coords;

                        let offset = new_surface_coord - self.zoom_tool.current_surface_coord;

                        // Drag down zooms out, drag up zooms in
                        let new_zoom =
                            total_zoom_old * (1.0 - offset[1] * Camera::DRAG_ZOOM_MAGN_ZOOM_FACTOR);

                        if (Camera::ZOOM_MIN..=Camera::ZOOM_MAX).contains(&new_zoom) {
                            widget_flags |= engine_view
                                .camera
                                .zoom_w_timeout(new_zoom, engine_view.tasks_tx.clone());

                            // Translate the camera view so that the start_surface_coord has the same surface position
                            // as before the zoom occurred
                            let new_camera_offset = (((camera_offset
                                + self.zoom_tool.start_surface_coord)
                                / total_zoom_old)
                                * new_zoom)
                                - self.zoom_tool.start_surface_coord;
                            widget_flags |= engine_view
                                .camera
                                .set_offset(new_camera_offset, engine_view.document);

                            widget_flags |= engine_view
                                .document
                                .expand_autoexpand(engine_view.camera, engine_view.store);
                        }
                        self.zoom_tool.current_surface_coord = new_surface_coord;
                    }
                }

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (ToolsState::Active, PenEvent::Up { .. }) => {
                match engine_view.pens_config.tools_config.style {
                    ToolStyle::VerticalSpace => {
                        engine_view
                            .store
                            .update_geometry_for_strokes(&self.verticalspace_tool.strokes_below);

                        widget_flags |= engine_view.store.record(Instant::now());
                        widget_flags.store_modified = true;
                    }
                    ToolStyle::OffsetCamera | ToolStyle::Zoom => {}
                }

                widget_flags |= engine_view
                    .document
                    .resize_autoexpand(engine_view.store, engine_view.camera);
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.reset(engine_view);

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

                self.reset(engine_view);

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (ToolsState::Active, PenEvent::Text { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
        };

        (event_result, widget_flags)
    }
}

impl DrawableOnDoc for Tools {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        match self.state {
            ToolsState::Active => match engine_view.pens_config.tools_config.style {
                ToolStyle::VerticalSpace => self.verticalspace_tool.bounds_on_doc(engine_view),
                ToolStyle::OffsetCamera => self.offsetcamera_tool.bounds_on_doc(engine_view),
                ToolStyle::Zoom => self.zoom_tool.bounds_on_doc(engine_view),
            },
            ToolsState::Idle => None,
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        match &engine_view.pens_config.tools_config.style {
            ToolStyle::VerticalSpace => {
                self.verticalspace_tool.draw_on_doc(cx, engine_view)?;
            }
            ToolStyle::OffsetCamera => {
                self.offsetcamera_tool.draw_on_doc(cx, engine_view)?;
            }
            ToolStyle::Zoom => {
                self.zoom_tool.draw_on_doc(cx, engine_view)?;
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Tools {
    fn reset(&mut self, engine_view: &mut EngineViewMut) {
        match engine_view.pens_config.tools_config.style {
            ToolStyle::VerticalSpace => {
                self.verticalspace_tool.start_pos_y = 0.0;
                self.verticalspace_tool.pos_y = 0.0;
            }
            ToolStyle::OffsetCamera => {
                self.offsetcamera_tool.start = na::Vector2::zeros();
            }
            ToolStyle::Zoom => {
                self.zoom_tool.start_surface_coord = na::Vector2::zeros();
                self.zoom_tool.current_surface_coord = na::Vector2::zeros();
            }
        }
        self.state = ToolsState::Idle;
    }
}
