use crate::sheet::Sheet;
use crate::strokesstate::StrokeKey;
use crate::{Camera, DrawOnSheetBehaviour, StrokesState, SurfaceFlags};
use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};
use rnote_compose::{Color, PenEvent};

use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;
use super::AudioPlayer;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "expandsheet_tool")]
pub struct ExpandSheetTool {
    #[serde(skip)]
    start_pos_y: f64,
    #[serde(skip)]
    current_pos_y: f64,
    #[serde(skip)]
    strokes_below: Vec<StrokeKey>,
}

impl Default for ExpandSheetTool {
    fn default() -> Self {
        Self {
            start_pos_y: 0.0,
            current_pos_y: 0.0,
            strokes_below: vec![],
        }
    }
}

impl ExpandSheetTool {
    pub const Y_OFFSET_THRESHOLD: f64 = 2.0;
    pub const FILL_COLOR: Color = Color {
        r: 0.7,
        g: 0.8,
        b: 0.9,
        a: 0.15,
    };
    pub const THRESHOLD_LINE_COLOR: Color = Color {
        r: 0.5,
        g: 0.7,
        b: 0.7,
        a: 1.0,
    };
    pub const THRESHOLD_LINE_WIDTH: f64 = 4.0;
    pub const OFFSET_LINE_COLOR: Color = Color {
        r: 0.0,
        g: 0.7,
        b: 1.0,
        a: 1.0,
    };
    pub const OFFSET_LINE_WIDTH: f64 = 2.0;
}

impl DrawOnSheetBehaviour for ExpandSheetTool {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        let viewport = camera.viewport();

        let x = viewport.mins[0];
        let y = self.start_pos_y;
        let width = viewport.extents()[0];
        let height = self.current_pos_y - self.start_pos_y;
        let tool_bounds = AABB::new_positive(na::point![x, y], na::point![x + width, y + height]);

        Some(tool_bounds)
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        camera: &Camera,
    ) -> Result<(), anyhow::Error> {
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
        cx.fill(
            tool_bounds_rect,
            &piet::PaintBrush::Color(Self::FILL_COLOR.into()),
        );

        let threshold_line =
            kurbo::Line::new(kurbo::Point::new(x, y), kurbo::Point::new(x + width, y));

        cx.stroke_styled(
            threshold_line,
            &piet::PaintBrush::Color(Self::THRESHOLD_LINE_COLOR.into()),
            Self::THRESHOLD_LINE_WIDTH,
            &piet::StrokeStyle::new().dash_pattern(&[12.0, 6.0]),
        );

        let offset_line = kurbo::Line::new(
            kurbo::Point::new(x, y + height),
            kurbo::Point::new(x + width, y + height),
        );
        cx.stroke(
            offset_line,
            &piet::PaintBrush::Color(Self::OFFSET_LINE_COLOR.into()),
            Self::OFFSET_LINE_WIDTH,
        );

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
    pub const OFFSET_MAGN_THRESHOLD: f64 = 4.0;
    pub const OUTLINE_COLOR: Color = Color {
        r: 0.5,
        g: 0.7,
        b: 0.7,
        a: 1.0,
    };
    pub const OUTLINE_WIDTH: f64 = 1.0;
    pub const FILL_COLOR: Color = Color {
        r: 0.8,
        g: 0.8,
        b: 0.8,
        a: 0.2,
    };
    pub const RADIUS_DEFAULT: f64 = 60.0;
}

impl DrawOnSheetBehaviour for DragProximityTool {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        Some(AABB::from_half_extents(
            na::Point2::from(self.pos),
            na::Vector2::repeat(self.radius),
        ))
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        _camera: &Camera,
    ) -> Result<(), anyhow::Error> {
        let mut radius = self.radius;

        let n_circles = 7;
        for i in (0..n_circles).rev() {
            radius *= f64::from(i) / f64::from(n_circles);

            let circle = kurbo::Circle::new(self.pos.to_kurbo_point(), radius);

            cx.fill(circle, &piet::PaintBrush::Color(Self::FILL_COLOR.into()));
            cx.stroke(
                circle,
                &piet::PaintBrush::Color(Self::OUTLINE_COLOR.into()),
                Self::OUTLINE_WIDTH,
            );
        }

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

impl DrawOnSheetBehaviour for OffsetCameraTool {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        Some(AABB::from_half_extents(
            na::Point2::from(self.start),
            (Self::DRAW_SIZE * 0.5 + na::Vector2::repeat(Self::PATH_WIDTH)) / camera.total_zoom(),
        ))
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        sheet_bounds: AABB,
        camera: &Camera,
    ) -> Result<(), anyhow::Error> {
        if let Some(bounds) = self.bounds_on_sheet(sheet_bounds, camera) {
            cx.transform(kurbo::Affine::translate(bounds.mins.coords.to_kurbo_vec()));
            cx.transform(kurbo::Affine::scale(1.0 / camera.total_zoom()));

            let bez_path =
                kurbo::BezPath::from_svg(include_str!("../../data/images/hand-grab-path.txt"))
                    .unwrap();

            cx.stroke(
                bez_path.clone(),
                &piet::PaintBrush::Color(Self::OUTLINE_COLOR.into()),
                Self::PATH_WIDTH,
            );
            cx.stroke(
                bez_path,
                &piet::PaintBrush::Color(Self::PATH_COLOR.into()),
                Self::PATH_WIDTH * 0.5,
            );
        }
        Ok(())
    }
}

impl OffsetCameraTool {
    const DRAW_SIZE: na::Vector2<f64> = na::vector![28.0, 28.0];
    const PATH_COLOR: Color = Color {
        r: 0.5,
        g: 0.7,
        b: 0.7,
        a: 1.0,
    };
    const OUTLINE_COLOR: Color = Color {
        r: 0.9,
        g: 0.9,
        b: 0.9,
        a: 0.8,
    };
    const PATH_WIDTH: f64 = 2.0;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "tools_style")]
pub enum ToolsStyle {
    #[serde(rename = "expandsheet")]
    ExpandSheet,
    #[serde(rename = "dragproximity")]
    DragProximity,
    #[serde(rename = "offsetcamera")]
    OffsetCamera,
}

impl Default for ToolsStyle {
    fn default() -> Self {
        Self::ExpandSheet
    }
}

#[derive(Debug, Clone, Copy)]
enum ToolsState {
    UpState,
    DownState,
}

impl Default for ToolsState {
    fn default() -> Self {
        Self::UpState
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename = "tools")]
pub struct Tools {
    #[serde(rename = "style")]
    pub style: ToolsStyle,
    #[serde(rename = "expandsheet_tool")]
    pub expandsheet_tool: ExpandSheetTool,
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
        _sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        camera: &mut Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        match (self.state, event) {
            (
                ToolsState::UpState,
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                match self.style {
                    ToolsStyle::ExpandSheet => {
                        self.expandsheet_tool.start_pos_y = element.pos[1];
                        self.expandsheet_tool.current_pos_y = element.pos[1];

                        self.expandsheet_tool.strokes_below =
                            strokes_state.keys_below_y_pos(self.expandsheet_tool.current_pos_y);
                    }
                    ToolsStyle::DragProximity => {
                        self.dragproximity_tool.pos = element.pos;
                        self.dragproximity_tool.offset = na::Vector2::zeros();
                    }
                    ToolsStyle::OffsetCamera => {
                        self.offsetcamera_tool.start = element.pos;
                    }
                }

                self.state = ToolsState::DownState;
            }
            (
                ToolsState::DownState,
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => match self.style {
                ToolsStyle::ExpandSheet => {
                    let y_offset = element.pos[1] - self.expandsheet_tool.current_pos_y;

                    if y_offset.abs() > ExpandSheetTool::Y_OFFSET_THRESHOLD {
                        strokes_state.translate_strokes(
                            &self.expandsheet_tool.strokes_below,
                            na::vector![0.0, y_offset],
                        );

                        self.expandsheet_tool.current_pos_y = element.pos[1];
                    }
                }
                ToolsStyle::DragProximity => {
                    let offset = element.pos - self.dragproximity_tool.pos;
                    self.dragproximity_tool.offset = offset;

                    if self.dragproximity_tool.offset.magnitude()
                        > DragProximityTool::OFFSET_MAGN_THRESHOLD
                    {
                        strokes_state.drag_strokes_proximity(&self.dragproximity_tool);
                        strokes_state.regenerate_rendering_in_viewport_threaded(
                            false,
                            camera.viewport(),
                            camera.image_scale(),
                        );

                        self.dragproximity_tool.pos = element.pos;
                        self.dragproximity_tool.offset = na::Vector2::zeros();
                    }
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
                        surface_flags.new_camera_offset = true;
                    }
                }
            },
            (ToolsState::UpState, PenEvent::Up { .. }) => {}
            (ToolsState::DownState, PenEvent::Up { .. }) => {
                self.reset();
                self.state = ToolsState::UpState;
            }
            (ToolsState::UpState, PenEvent::Proximity { .. }) => {}
            (ToolsState::DownState, PenEvent::Proximity { .. }) => {
                self.reset();
                self.state = ToolsState::UpState;
            }
            (ToolsState::UpState, PenEvent::Cancel) => {}
            (ToolsState::DownState, PenEvent::Cancel) => {
                self.reset();
                self.state = ToolsState::UpState;
            }
        }

        surface_flags
    }
}

impl DrawOnSheetBehaviour for Tools {
    fn bounds_on_sheet(&self, sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        match self.style {
            ToolsStyle::ExpandSheet => self.expandsheet_tool.bounds_on_sheet(sheet_bounds, camera),
            ToolsStyle::DragProximity => self
                .dragproximity_tool
                .bounds_on_sheet(sheet_bounds, camera),
            ToolsStyle::OffsetCamera => {
                self.offsetcamera_tool.bounds_on_sheet(sheet_bounds, camera)
            }
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        sheet_bounds: AABB,
        camera: &Camera,
    ) -> Result<(), anyhow::Error> {
        match &self.style {
            ToolsStyle::ExpandSheet => {
                self.expandsheet_tool
                    .draw_on_sheet(cx, sheet_bounds, camera)
            }
            ToolsStyle::DragProximity => {
                self.dragproximity_tool
                    .draw_on_sheet(cx, sheet_bounds, camera)
            }
            ToolsStyle::OffsetCamera => {
                self.offsetcamera_tool
                    .draw_on_sheet(cx, sheet_bounds, camera)
            }
        }
    }
}

impl Tools {
    fn reset(&mut self) {
        let current_style = self.style;

        match current_style {
            ToolsStyle::ExpandSheet => {
                self.expandsheet_tool.start_pos_y = 0.0;
                self.expandsheet_tool.current_pos_y = 0.0;
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
