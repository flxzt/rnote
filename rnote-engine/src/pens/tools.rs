use std::time::Instant;

use crate::engine::{EngineView, EngineViewMut};
use crate::store::StrokeKey;
use crate::{DrawOnDocBehaviour, WidgetFlags};
use once_cell::sync::Lazy;
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::helpers::{AabbHelpers, Vector2Helpers};
use rnote_compose::penevents::PenEvent;

use p2d::bounding_volume::Aabb;

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::pensconfig::toolsconfig::ToolsStyle;
use super::PenStyle;

#[derive(Clone, Debug)]
pub struct VerticalSpaceTool {
    start_pos_y: f64,
    current_pos_y: f64,
    strokes_below: Vec<StrokeKey>,
}

impl Default for VerticalSpaceTool {
    fn default() -> Self {
        Self {
            start_pos_y: 0.0,
            current_pos_y: 0.0,
            strokes_below: vec![],
        }
    }
}

static VERTICALSPACETOOL_FILL_COLOR: Lazy<piet::Color> =
    Lazy::new(|| color::GNOME_BRIGHTS[2].with_alpha(0.090));
static VERTICALSPACETOOL_THRESHOLD_LINE_COLOR: Lazy<piet::Color> =
    Lazy::new(|| color::GNOME_GREENS[4].with_alpha(0.941));

impl VerticalSpaceTool {
    const Y_OFFSET_THRESHOLD: f64 = 0.1;
    const OFFSET_LINE_COLOR: piet::Color = color::GNOME_BLUES[3];
    const THRESHOLD_LINE_WIDTH: f64 = 4.0;
    const OFFSET_LINE_WIDTH: f64 = 2.0;
}

impl DrawOnDocBehaviour for VerticalSpaceTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let viewport = engine_view.camera.viewport();

        let x = viewport.mins[0];
        let y = self.start_pos_y;
        let width = viewport.extents()[0];
        let height = self.current_pos_y - self.start_pos_y;
        let tool_bounds = Aabb::new_positive(na::point![x, y], na::point![x + width, y + height]);

        Some(tool_bounds)
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let viewport = engine_view.camera.viewport();
        let x = viewport.mins[0];
        let y = self.start_pos_y;
        let width = viewport.extents()[0];
        let height = self.current_pos_y - self.start_pos_y;
        let tool_bounds = Aabb::new_positive(na::point![x, y], na::point![x + width, y + height]);

        let tool_bounds_rect = kurbo::Rect::from_points(
            tool_bounds.mins.coords.to_kurbo_point(),
            tool_bounds.maxs.coords.to_kurbo_point(),
        );
        cx.fill(tool_bounds_rect, &*VERTICALSPACETOOL_FILL_COLOR);

        let threshold_line =
            kurbo::Line::new(kurbo::Point::new(x, y), kurbo::Point::new(x + width, y));

        cx.stroke_styled(
            threshold_line,
            &*VERTICALSPACETOOL_THRESHOLD_LINE_COLOR,
            Self::THRESHOLD_LINE_WIDTH,
            &piet::StrokeStyle::new().dash_pattern(&[12.0, 6.0]),
        );

        let offset_line = kurbo::Line::new(
            kurbo::Point::new(x, y + height),
            kurbo::Point::new(x + width, y + height),
        );
        cx.stroke(
            offset_line,
            &Self::OFFSET_LINE_COLOR,
            Self::OFFSET_LINE_WIDTH,
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

static OFFSETCAMERATOOL_FILL_COLOR: Lazy<piet::Color> =
    Lazy::new(|| color::GNOME_DARKS[3].with_alpha(0.941));
static OFFSETCAMERATOOL_OUTLINE_COLOR: Lazy<piet::Color> =
    Lazy::new(|| color::GNOME_BRIGHTS[1].with_alpha(0.941));

impl OffsetCameraTool {
    const DRAW_SIZE: na::Vector2<f64> = na::vector![16.0, 16.0];
    const PATH_WIDTH: f64 = 2.0;

    const CURSOR_PATH: &str = "m 8 1.078125 l -3 3 h 2 v 2.929687 h -2.960938 v -2 l -3 3 l 3 3 v -2 h 2.960938 v 2.960938 h -2 l 3 3 l 3 -3 h -2 v -2.960938 h 3.054688 v 2 l 3 -3 l -3 -3 v 2 h -3.054688 v -2.929687 h 2 z m 0 0";
}

impl DrawOnDocBehaviour for OffsetCameraTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        Some(Aabb::from_half_extents(
            na::Point2::from(self.start),
            ((Self::DRAW_SIZE + na::Vector2::repeat(Self::PATH_WIDTH)) * 0.5)
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
                &*OFFSETCAMERATOOL_OUTLINE_COLOR,
                Self::PATH_WIDTH,
            );
            cx.fill(bez_path, &*OFFSETCAMERATOOL_FILL_COLOR);
        }

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
    state: ToolsState,
}

impl PenBehaviour for Tools {
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
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (
                ToolsState::Idle,
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                widget_flags.merge(engine_view.store.record(Instant::now()));

                match engine_view.pens_config.tools_config.style {
                    ToolsStyle::VerticalSpace => {
                        self.verticalspace_tool.start_pos_y = element.pos[1];
                        self.verticalspace_tool.current_pos_y = element.pos[1];

                        self.verticalspace_tool.strokes_below = engine_view
                            .store
                            .keys_below_y_pos(self.verticalspace_tool.current_pos_y);
                    }
                    ToolsStyle::OffsetCamera => {
                        self.offsetcamera_tool.start = element.pos;
                    }
                }

                self.state = ToolsState::Active;

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
            (ToolsState::Idle, _) => PenProgress::Idle,
            (
                ToolsState::Active,
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                let pen_progress = match engine_view.pens_config.tools_config.style {
                    ToolsStyle::VerticalSpace => {
                        let y_offset = element.pos[1] - self.verticalspace_tool.current_pos_y;

                        if y_offset.abs() > VerticalSpaceTool::Y_OFFSET_THRESHOLD {
                            engine_view.store.translate_strokes(
                                &self.verticalspace_tool.strokes_below,
                                na::vector![0.0, y_offset],
                            );
                            engine_view.store.translate_strokes_images(
                                &self.verticalspace_tool.strokes_below,
                                na::vector![0.0, y_offset],
                            );

                            self.verticalspace_tool.current_pos_y = element.pos[1];
                        }

                        PenProgress::InProgress
                    }
                    ToolsStyle::OffsetCamera => {
                        let offset = engine_view
                            .camera
                            .transform()
                            .transform_point(&na::Point2::from(element.pos))
                            .coords
                            - engine_view
                                .camera
                                .transform()
                                .transform_point(&na::Point2::from(self.offsetcamera_tool.start))
                                .coords;

                        if offset.magnitude() > 1.0 {
                            engine_view.camera.offset -= offset;

                            engine_view
                                .doc
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            widget_flags.resize = true;
                            widget_flags.update_view = true;
                        }

                        PenProgress::InProgress
                    }
                };

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                pen_progress
            }
            (ToolsState::Active, PenEvent::Up { .. }) => {
                match engine_view.pens_config.tools_config.style {
                    ToolsStyle::VerticalSpace => {
                        engine_view
                            .store
                            .update_geometry_for_strokes(&self.verticalspace_tool.strokes_below);
                    }
                    ToolsStyle::OffsetCamera => {}
                }
                engine_view.store.regenerate_rendering_in_viewport_threaded(
                    engine_view.tasks_tx.clone(),
                    false,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.reset(engine_view);
                self.state = ToolsState::Idle;

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::Finished
            }
            (ToolsState::Active, PenEvent::Proximity { .. }) => PenProgress::InProgress,
            (ToolsState::Active, PenEvent::KeyPressed { .. }) => PenProgress::InProgress,
            (ToolsState::Active, PenEvent::Cancel) => {
                self.reset(engine_view);
                self.state = ToolsState::Idle;

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::Finished
            }
            (ToolsState::Active, PenEvent::Text { .. }) => PenProgress::InProgress,
        };

        (pen_progress, widget_flags)
    }
}

impl DrawOnDocBehaviour for Tools {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        match self.state {
            ToolsState::Active => match engine_view.pens_config.tools_config.style {
                ToolsStyle::VerticalSpace => self.verticalspace_tool.bounds_on_doc(engine_view),
                ToolsStyle::OffsetCamera => self.offsetcamera_tool.bounds_on_doc(engine_view),
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
            ToolsStyle::VerticalSpace => {
                self.verticalspace_tool.draw_on_doc(cx, engine_view)?;
            }
            ToolsStyle::OffsetCamera => {
                self.offsetcamera_tool.draw_on_doc(cx, engine_view)?;
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Tools {
    fn reset(&mut self, engine_view: &mut EngineViewMut) {
        match engine_view.pens_config.tools_config.style {
            ToolsStyle::VerticalSpace => {
                self.verticalspace_tool.start_pos_y = 0.0;
                self.verticalspace_tool.current_pos_y = 0.0;
            }
            ToolsStyle::OffsetCamera => {
                self.offsetcamera_tool.start = na::Vector2::zeros();
            }
        }
    }
}
