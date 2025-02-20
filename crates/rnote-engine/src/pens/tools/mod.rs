// Modules
mod laser;
mod offsetcamera;
mod verticalspace;
mod zoom;

// Re-Exports
use laser::LaserTool;
use offsetcamera::OffsetCameraTool;
use verticalspace::VerticalSpaceTool;
use zoom::ZoomTool;

// Imports
use super::PenBehaviour;
use super::PenStyle;
use super::pensconfig::toolsconfig::ToolStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::eventresult::EventResult;
use rnote_compose::penevent::{PenEvent, PenProgress};
use std::time::Instant;

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

#[derive(Debug, Default)]
pub struct Tools {
    verticalspace_tool: VerticalSpaceTool,
    offsetcamera_tool: OffsetCameraTool,
    zoom_tool: ZoomTool,
    laser_tool: LaserTool,
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
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        match engine_view.pens_config.tools_config.style {
            ToolStyle::VerticalSpace => {
                self.verticalspace_tool
                    .handle_event(event, now, engine_view)
            }
            ToolStyle::OffsetCamera => self.offsetcamera_tool.handle_event(event, now, engine_view),
            ToolStyle::Zoom => self.zoom_tool.handle_event(event, now, engine_view),
            ToolStyle::Laser => self.laser_tool.handle_event(event, now, engine_view),
        }
    }

    fn handle_animation_frame(&mut self, engine_view: &mut EngineViewMut, optimize_epd: bool) {
        match engine_view.pens_config.tools_config.style {
            ToolStyle::Laser => self
                .laser_tool
                .handle_animation_frame(engine_view, optimize_epd),
            _ => {}
        }
    }
}

impl DrawableOnDoc for Tools {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        match engine_view.pens_config.tools_config.style {
            ToolStyle::VerticalSpace => self.verticalspace_tool.bounds_on_doc(engine_view),
            ToolStyle::OffsetCamera => self.offsetcamera_tool.bounds_on_doc(engine_view),
            ToolStyle::Zoom => self.zoom_tool.bounds_on_doc(engine_view),
            ToolStyle::Laser => self.laser_tool.bounds_on_doc(engine_view),
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
            ToolStyle::Laser => {
                self.laser_tool.draw_on_doc(cx, engine_view)?;
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
