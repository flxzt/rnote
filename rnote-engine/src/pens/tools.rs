use crate::document::Document;
use crate::engine::EngineTaskSender;
use crate::store::StrokeKey;
use crate::{Camera, DrawOnDocBehaviour, StrokeStore, SurfaceFlags};
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};
use rnote_compose::penhelpers::PenEvent;

use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::AudioPlayer;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "verticalspace_tool")]
pub struct VerticalSpaceTool {
    #[serde(skip)]
    start_pos_y: f64,
    #[serde(skip)]
    current_pos_y: f64,
    #[serde(skip)]
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

impl VerticalSpaceTool {
    const Y_OFFSET_THRESHOLD: f64 = 0.1;

    const FILL_COLOR: piet::Color = color::GNOME_BRIGHTS[2].with_a8(0x17);
    const THRESHOLD_LINE_COLOR: piet::Color = color::GNOME_GREENS[4].with_a8(0xf0);
    const OFFSET_LINE_COLOR: piet::Color = color::GNOME_BLUES[3];

    const THRESHOLD_LINE_WIDTH: f64 = 4.0;
    const OFFSET_LINE_WIDTH: f64 = 2.0;
}

impl DrawOnDocBehaviour for VerticalSpaceTool {
    fn bounds_on_doc(
        &self,
        _doc: &Document,
        _store: &StrokeStore,
        camera: &Camera,
    ) -> Option<AABB> {
        let viewport = camera.viewport();

        let x = viewport.mins[0];
        let y = self.start_pos_y;
        let width = viewport.extents()[0];
        let height = self.current_pos_y - self.start_pos_y;
        let tool_bounds = AABB::new_positive(na::point![x, y], na::point![x + width, y + height]);

        Some(tool_bounds)
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        _doc: &Document,
        _store: &StrokeStore,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        let viewport = camera.viewport();
        let x = viewport.mins[0];
        let y = self.start_pos_y;
        let width = viewport.extents()[0];
        let height = self.current_pos_y - self.start_pos_y;
        let tool_bounds = AABB::new_positive(na::point![x, y], na::point![x + width, y + height]);

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

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "dragproximity_tool")]
pub struct DragProximityTool {
    #[serde(skip)]
    pub pos: na::Vector2<f64>,
    #[serde(skip)]
    pub offset: na::Vector2<f64>,
    #[serde(rename = "radius")]
    pub radius: f64,
}

impl Default for DragProximityTool {
    fn default() -> Self {
        Self {
            pos: na::Vector2::zeros(),
            offset: na::Vector2::zeros(),
            radius: Self::RADIUS_DEFAULT,
        }
    }
}

impl DragProximityTool {
    const OFFSET_MAGNITUDE_THRESHOLD: f64 = 4.0;
    const OUTLINE_COLOR: piet::Color = color::GNOME_GREENS[4];
    const FILL_COLOR: piet::Color = color::GNOME_BLUES[1].with_a8(0x60);

    pub const OUTLINE_WIDTH: f64 = 1.0;
    pub const RADIUS_DEFAULT: f64 = 60.0;
}

impl DrawOnDocBehaviour for DragProximityTool {
    fn bounds_on_doc(
        &self,
        _doc: &Document,
        _store: &StrokeStore,
        _camera: &Camera,
    ) -> Option<AABB> {
        Some(AABB::from_half_extents(
            na::Point2::from(self.pos),
            na::Vector2::repeat(self.radius),
        ))
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        _doc: &Document,
        _store: &StrokeStore,
        _camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
        let mut radius = self.radius;

        let n_circles = 7;
        for i in (0..n_circles).rev() {
            radius *= f64::from(i) / f64::from(n_circles);

            let circle = kurbo::Circle::new(self.pos.to_kurbo_point(), radius);

            cx.fill(circle, &Self::FILL_COLOR);
            cx.stroke(circle, &Self::OUTLINE_COLOR, Self::OUTLINE_WIDTH);
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "offsetcamera_tool")]
pub struct OffsetCameraTool {
    #[serde(skip)]
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
    const DRAW_SIZE: na::Vector2<f64> = na::vector![16.0, 16.0];
    const FILL_COLOR: piet::Color = color::GNOME_DARKS[3].with_a8(0xf0);
    const OUTLINE_COLOR: piet::Color = color::GNOME_BRIGHTS[1].with_a8(0xf0);
    const PATH_WIDTH: f64 = 2.0;
}

impl DrawOnDocBehaviour for OffsetCameraTool {
    fn bounds_on_doc(
        &self,
        _doc: &Document,
        _store: &StrokeStore,
        camera: &Camera,
    ) -> Option<AABB> {
        Some(AABB::from_half_extents(
            na::Point2::from(self.start),
            ((Self::DRAW_SIZE + na::Vector2::repeat(Self::PATH_WIDTH)) * 0.5) / camera.total_zoom(),
        ))
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        doc: &Document,
        store: &StrokeStore,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        if let Some(bounds) = self.bounds_on_doc(doc, store, camera) {
            cx.transform(kurbo::Affine::translate(bounds.mins.coords.to_kurbo_vec()));
            cx.transform(kurbo::Affine::scale(1.0 / camera.total_zoom()));

            let bez_path = kurbo::BezPath::from_svg(include_str!(
                "../../data/images/offsetcameratool-path.txt"
            ))
            .unwrap();

            cx.stroke(bez_path.clone(), &Self::OUTLINE_COLOR, Self::PATH_WIDTH);
            cx.fill(bez_path, &Self::FILL_COLOR);
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "tools_style")]
pub enum ToolsStyle {
    #[serde(rename = "verticalspace")]
    VerticalSpace,
    #[serde(rename = "dragproximity")]
    DragProximity,
    #[serde(rename = "offsetcamera")]
    OffsetCamera,
}

impl Default for ToolsStyle {
    fn default() -> Self {
        Self::VerticalSpace
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename = "tools")]
pub struct Tools {
    #[serde(rename = "style")]
    pub style: ToolsStyle,
    #[serde(rename = "verticalspace_tool")]
    pub verticalspace_tool: VerticalSpaceTool,
    #[serde(rename = "dragproximity_tool")]
    pub dragproximity_tool: DragProximityTool,
    #[serde(rename = "offsetcamera_tool")]
    pub offsetcamera_tool: OffsetCameraTool,

    #[serde(skip)]
    state: ToolsState,
}

impl PenBehaviour for Tools {
    fn handle_event(
        &mut self,
        event: PenEvent,
        tasks_tx: EngineTaskSender,
        doc: &mut Document,
        store: &mut StrokeStore,
        camera: &mut Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags) {
        let mut surface_flags = SurfaceFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (
                ToolsState::Idle,
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                surface_flags.merge_with_other(store.record());

                match self.style {
                    ToolsStyle::VerticalSpace => {
                        self.verticalspace_tool.start_pos_y = element.pos[1];
                        self.verticalspace_tool.current_pos_y = element.pos[1];

                        self.verticalspace_tool.strokes_below =
                            store.keys_below_y_pos(self.verticalspace_tool.current_pos_y);
                    }
                    ToolsStyle::DragProximity => {
                        self.dragproximity_tool.pos = element.pos;
                        self.dragproximity_tool.offset = na::Vector2::zeros();
                    }
                    ToolsStyle::OffsetCamera => {
                        self.offsetcamera_tool.start = element.pos;
                    }
                }

                self.state = ToolsState::Active;

                doc.resize_autoexpand(store, camera);

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.store_changed = true;
                surface_flags.hide_scrollbars = Some(true);

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
                let pen_progress = match self.style {
                    ToolsStyle::VerticalSpace => {
                        let y_offset = element.pos[1] - self.verticalspace_tool.current_pos_y;

                        if y_offset.abs() > VerticalSpaceTool::Y_OFFSET_THRESHOLD {
                            store.translate_strokes(
                                &self.verticalspace_tool.strokes_below,
                                na::vector![0.0, y_offset],
                            );
                            store.translate_strokes_images(
                                &self.verticalspace_tool.strokes_below,
                                na::vector![0.0, y_offset],
                            );

                            self.verticalspace_tool.current_pos_y = element.pos[1];
                        }

                        PenProgress::InProgress
                    }
                    ToolsStyle::DragProximity => {
                        let offset = element.pos - self.dragproximity_tool.pos;
                        self.dragproximity_tool.offset = offset;

                        if self.dragproximity_tool.offset.magnitude()
                            > DragProximityTool::OFFSET_MAGNITUDE_THRESHOLD
                        {
                            store.drag_strokes_proximity(&self.dragproximity_tool);
                            store.regenerate_rendering_in_viewport_threaded(
                                tasks_tx,
                                false,
                                camera.viewport(),
                                camera.image_scale(),
                            );

                            self.dragproximity_tool.pos = element.pos;
                            self.dragproximity_tool.offset = na::Vector2::zeros();
                        }

                        PenProgress::InProgress
                    }
                    ToolsStyle::OffsetCamera => {
                        let offset = camera
                            .transform()
                            .transform_point(&na::Point2::from(element.pos))
                            .coords
                            - camera
                                .transform()
                                .transform_point(&na::Point2::from(self.offsetcamera_tool.start))
                                .coords;

                        if offset.magnitude() > 1.0 {
                            camera.offset -= offset;

                            doc.resize_autoexpand(store, camera);

                            surface_flags.resize = true;
                            surface_flags.camera_changed = true;
                        }

                        PenProgress::InProgress
                    }
                };

                surface_flags.redraw = true;
                surface_flags.store_changed = true;

                pen_progress
            }
            (ToolsState::Active, PenEvent::Up { .. }) => {
                match self.style {
                    ToolsStyle::VerticalSpace => {
                        store.update_geometry_for_strokes(&self.verticalspace_tool.strokes_below);
                    }
                    ToolsStyle::DragProximity => {}
                    ToolsStyle::OffsetCamera => {}
                }
                store.regenerate_rendering_in_viewport_threaded(
                    tasks_tx,
                    false,
                    camera.viewport(),
                    camera.image_scale(),
                );

                self.reset();
                self.state = ToolsState::Idle;

                doc.resize_autoexpand(store, camera);

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.store_changed = true;
                surface_flags.hide_scrollbars = Some(false);

                PenProgress::Finished
            }
            (ToolsState::Active, PenEvent::Proximity { .. }) => PenProgress::InProgress,
            (ToolsState::Active, PenEvent::KeyPressed { .. }) => PenProgress::InProgress,
            (ToolsState::Active, PenEvent::Cancel) => {
                self.reset();
                self.state = ToolsState::Idle;

                doc.resize_autoexpand(store, camera);

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.store_changed = true;
                surface_flags.hide_scrollbars = Some(false);

                PenProgress::Finished
            }
        };

        (pen_progress, surface_flags)
    }
}

impl DrawOnDocBehaviour for Tools {
    fn bounds_on_doc(&self, doc: &Document, store: &StrokeStore, camera: &Camera) -> Option<AABB> {
        match self.state {
            ToolsState::Active => match self.style {
                ToolsStyle::VerticalSpace => {
                    self.verticalspace_tool.bounds_on_doc(doc, store, camera)
                }
                ToolsStyle::DragProximity => {
                    self.dragproximity_tool.bounds_on_doc(doc, store, camera)
                }
                ToolsStyle::OffsetCamera => {
                    self.offsetcamera_tool.bounds_on_doc(doc, store, camera)
                }
            },
            ToolsState::Idle => None,
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        doc: &Document,
        store: &StrokeStore,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        match &self.style {
            ToolsStyle::VerticalSpace => {
                self.verticalspace_tool
                    .draw_on_doc(cx, doc, store, camera)?;
            }
            ToolsStyle::DragProximity => {
                self.dragproximity_tool
                    .draw_on_doc(cx, doc, store, camera)?;
            }
            ToolsStyle::OffsetCamera => {
                self.offsetcamera_tool.draw_on_doc(cx, doc, store, camera)?;
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

impl Tools {
    fn reset(&mut self) {
        let current_style = self.style;

        match current_style {
            ToolsStyle::VerticalSpace => {
                self.verticalspace_tool.start_pos_y = 0.0;
                self.verticalspace_tool.current_pos_y = 0.0;
            }
            ToolsStyle::DragProximity => {
                self.dragproximity_tool.pos = na::Vector2::zeros();
                self.dragproximity_tool.offset = na::Vector2::zeros();
            }
            ToolsStyle::OffsetCamera => {
                self.offsetcamera_tool.start = na::Vector2::zeros();
            }
        }
    }
}
