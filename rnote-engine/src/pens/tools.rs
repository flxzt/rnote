use crate::sheet::Sheet;
use crate::strokesstate::StrokeKey;
use crate::{Camera, DrawOnSheetBehaviour, StrokesState};
use rnote_compose::helpers::{AABBHelpers, Vector2Helpers};
use rnote_compose::{Color, PenEvent};

use gtk4::glib;
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::AudioPlayer;
use super::penbehaviour::PenBehaviour;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, glib::Enum)]
#[serde(rename = "tools_style")]
#[enum_type(name = "ToolsStyle")]
pub enum ToolsStyle {
    #[serde(rename = "expandsheet")]
    #[enum_value(name = "Expandsheet", nick = "expandsheet")]
    ExpandSheet,
    #[serde(rename = "dragproximity")]
    #[enum_value(name = "Dragproximity", nick = "dragproximity")]
    DragProximity,
}

impl Default for ToolsStyle {
    fn default() -> Self {
        Self::ExpandSheet
    }
}

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
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, viewport: AABB) -> Option<AABB> {
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
        viewport: AABB,
    ) -> Result<(), anyhow::Error> {
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
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _viewport: AABB) -> Option<AABB> {
        Some(AABB::from_half_extents(
            na::Point2::from(self.pos),
            na::Vector2::repeat(self.radius),
        ))
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        _viewport: AABB,
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
    #[serde(skip)]
    pub expand_sheet_tool: ExpandSheetTool,
    #[serde(skip)]
    pub drag_proximity_tool: DragProximityTool,

    #[serde(skip)]
    state: ToolsState,
}

impl PenBehaviour for Tools {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        camera: &Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) {
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
                        self.expand_sheet_tool.start_pos_y = element.pos[1];
                        self.expand_sheet_tool.current_pos_y = element.pos[1];

                        self.expand_sheet_tool.strokes_below =
                            strokes_state.keys_below_y_pos(self.expand_sheet_tool.current_pos_y);
                    }
                    ToolsStyle::DragProximity => {
                        self.drag_proximity_tool.pos = element.pos;
                        self.drag_proximity_tool.offset = na::Vector2::zeros();
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
                    let y_offset = element.pos[1] - self.expand_sheet_tool.current_pos_y;

                    if y_offset.abs() > ExpandSheetTool::Y_OFFSET_THRESHOLD {
                        strokes_state.translate_strokes(
                            &self.expand_sheet_tool.strokes_below,
                            na::vector![0.0, y_offset],
                        );

                        self.expand_sheet_tool.current_pos_y = element.pos[1];
                    }
                }
                ToolsStyle::DragProximity => {
                    let offset = element.pos - self.drag_proximity_tool.pos;
                    self.drag_proximity_tool.offset = offset;

                    if self.drag_proximity_tool.offset.magnitude()
                        > DragProximityTool::OFFSET_MAGN_THRESHOLD
                    {
                        strokes_state.drag_strokes_proximity(&self.drag_proximity_tool);
                        strokes_state.regenerate_rendering_in_viewport_threaded(
                            false,
                            Some(camera.viewport()),
                            camera.image_scale(),
                        );

                        self.drag_proximity_tool.pos = element.pos;
                        self.drag_proximity_tool.offset = na::Vector2::zeros();
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
    }
}

impl DrawOnSheetBehaviour for Tools {
    fn bounds_on_sheet(&self, sheet_bounds: AABB, viewport: AABB) -> Option<AABB> {
        match self.style {
            ToolsStyle::ExpandSheet => self
                .expand_sheet_tool
                .bounds_on_sheet(sheet_bounds, viewport),
            ToolsStyle::DragProximity => self
                .drag_proximity_tool
                .bounds_on_sheet(sheet_bounds, viewport),
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        sheet_bounds: AABB,
        viewport: AABB,
    ) -> Result<(), anyhow::Error> {
        match &self.style {
            ToolsStyle::ExpandSheet => {
                self.expand_sheet_tool
                    .draw_on_sheet(cx, sheet_bounds, viewport)
            }
            ToolsStyle::DragProximity => {
                self.drag_proximity_tool
                    .draw_on_sheet(cx, sheet_bounds, viewport)
            }
        }
    }
}

impl Tools {
    fn reset(&mut self) {
        let current_style = self.style;

        match current_style {
            ToolsStyle::ExpandSheet => {
                self.expand_sheet_tool.start_pos_y = 0.0;
                self.expand_sheet_tool.current_pos_y = 0.0;
            }
            ToolsStyle::DragProximity => {
                self.drag_proximity_tool.pos = na::Vector2::zeros();
                self.drag_proximity_tool.offset = na::Vector2::zeros();
            }
        }
    }
}
