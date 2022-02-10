use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::compose::color::Color;
use crate::compose::geometry;
use crate::render::Renderer;
use crate::strokes::strokestyle::InputData;
use crate::strokesstate::StrokeKey;
use crate::ui::appwindow::RnoteAppWindow;
use crate::{compose, render};

use anyhow::Context;
use gtk4::{glib, prelude::*, Snapshot};
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, glib::Enum)]
#[serde(rename = "tool_style")]
#[enum_type(name = "ToolStyle")]
pub enum ToolStyle {
    #[serde(rename = "expandsheet")]
    #[enum_value(name = "Expandsheet", nick = "expandsheet")]
    ExpandSheet,
    #[serde(rename = "dragproximity")]
    #[enum_value(name = "Dragproximity", nick = "dragproximity")]
    DragProximity,
}

impl Default for ToolStyle {
    fn default() -> Self {
        Self::ExpandSheet
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "expandsheet_tool")]
pub struct ExpandSheetTool {
    #[serde(skip)]
    y_start_pos: f64,
    #[serde(skip)]
    y_current_pos: f64,
    #[serde(skip)]
    strokes_below: Vec<StrokeKey>,
}

impl Default for ExpandSheetTool {
    fn default() -> Self {
        Self {
            y_start_pos: 0.0,
            y_current_pos: 0.0,
            strokes_below: vec![],
        }
    }
}

impl ExpandSheetTool {
    pub const Y_OFFSET_THRESHOLD: f64 = 2.0;
    pub const FILL_COLOR: Color = Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 0.2,
    };
    pub const THRESHOLD_LINE_COLOR: Color = Color {
        r: 0.5,
        g: 0.7,
        b: 0.7,
        a: 1.0,
    };
    pub const THRESHOLD_LINE_STROKE_WIDTH: f64 = 10.0;
    pub const OFFSET_LINE_COLOR: Color = Color {
        r: 0.0,
        g: 0.7,
        b: 1.0,
        a: 1.0,
    };
    pub const OFFSET_LINE_STROKE_WIDTH: f64 = 2.0;

    pub fn draw(
        &self,
        sheet_bounds: AABB,
        zoom: f64,
        snapshot: &Snapshot,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        let x = sheet_bounds.mins[0];
        let y = self.y_start_pos;
        let width = sheet_bounds.extents()[0];
        let height = self.y_current_pos - self.y_start_pos;
        let bounds = geometry::aabb_ceil(geometry::aabb_new_positive(
            na::point![x, y],
            na::point![x + width, y + height],
        ));

        let bounds_rect = svg::node::element::Rectangle::new()
            .set("x", bounds.mins[0])
            .set("y", bounds.mins[1])
            .set("width", bounds.extents()[0])
            .set("height", bounds.extents()[1])
            .set("stroke", "none")
            .set("fill", Self::FILL_COLOR.to_css_color())
            .set("stroke-linejoin", "miter")
            .set("stroke-linecap", "butt");

        let threshold_line = svg::node::element::Line::new()
            .set("x1", x)
            .set("y1", y)
            .set("x2", x + width)
            .set("y2", y)
            .set("stroke", Self::THRESHOLD_LINE_COLOR.to_css_color())
            .set("stroke-width", Self::THRESHOLD_LINE_STROKE_WIDTH)
            .set("stroke-dasharray", "16 12")
            .set("stroke-linecap", "butt");

        let offset_line = svg::node::element::Line::new()
            .set("x1", x)
            .set("y1", y + height)
            .set("x2", x + width)
            .set("y2", y + height)
            .set("stroke", Self::OFFSET_LINE_COLOR.to_css_color())
            .set("stroke-width", Self::OFFSET_LINE_STROKE_WIDTH)
            .set("stroke-linecap", "butt");

        let group = svg::node::element::Group::new()
            .add(bounds_rect)
            .add(threshold_line)
            .add(offset_line);

        let svg_data = compose::svg_node_to_string(&group)?;
        let svg = render::Svg { svg_data, bounds };

        if let Some(image) = renderer.read().unwrap().gen_image(zoom, &[svg], bounds)? {
            let rendernode = render::image_to_rendernode(&image, zoom)
                .context("ExpandSheetTool draw() failed")?;
            snapshot.append_node(&rendernode);
        }

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

    pub fn draw(
        &self,
        _sheet_bounds: AABB,
        zoom: f64,
        snapshot: &Snapshot,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        let cx = self.pos[0] + self.offset[0];
        let cy = self.pos[1] + self.offset[1];
        let r = self.radius;
        let mut draw_bounds = geometry::aabb_new_positive(
            na::point![cx - r - Self::OUTLINE_WIDTH, cy - r - Self::OUTLINE_WIDTH],
            na::point![cx + r + Self::OUTLINE_WIDTH, cy + r + Self::OUTLINE_WIDTH],
        );
        draw_bounds.take_point(na::Point2::from(self.pos.add_scalar(-Self::OUTLINE_WIDTH)));
        draw_bounds.take_point(na::Point2::from(self.pos.add_scalar(Self::OUTLINE_WIDTH)));

        let mut group = svg::node::element::Group::new();

        let n_circles = 7;
        for i in (0..n_circles).rev() {
            let r = r * (f64::from(i) / f64::from(n_circles));

            let outline_circle = svg::node::element::Circle::new()
                .set("cx", cx)
                .set("cy", cy)
                .set("r", r)
                .set("stroke", Self::OUTLINE_COLOR.to_css_color())
                .set("stroke-width", Self::OUTLINE_WIDTH)
                .set("fill", Self::FILL_COLOR.to_css_color());

            group = group.add(outline_circle);
        }

        let svg_data = compose::svg_node_to_string(&group)?;
        let svg = render::Svg {
            svg_data,
            bounds: draw_bounds,
        };

        if let Some(image) = renderer
            .read()
            .unwrap()
            .gen_image(zoom, &[svg], draw_bounds)?
        {
            let rendernode = render::image_to_rendernode(&image, zoom)
                .context("DrawProximityTool draw() failed")?;
            snapshot.append_node(&rendernode);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename = "tools")]
pub struct Tools {
    #[serde(rename = "style")]
    pub style: ToolStyle,
    #[serde(skip)]
    pub expand_sheet_tool: ExpandSheetTool,
    #[serde(skip)]
    pub drag_proximity_tool: DragProximityTool,
}

impl PenBehaviour for Tools {
    fn begin(mut data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().motion_cursor()));

        if let Some(inputdata) = data_entries.pop_back() {
            let current_style = appwindow.canvas().pens().borrow().tools.style;

            match current_style {
                ToolStyle::ExpandSheet => {
                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .tools
                        .expand_sheet_tool
                        .y_start_pos = inputdata.pos()[1];
                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .tools
                        .expand_sheet_tool
                        .y_current_pos = inputdata.pos()[1];

                    let y_current_pos = appwindow
                        .canvas()
                        .pens()
                        .borrow()
                        .tools
                        .expand_sheet_tool
                        .y_current_pos;

                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .tools
                        .expand_sheet_tool
                        .strokes_below = appwindow
                        .canvas()
                        .sheet()
                        .borrow()
                        .strokes_state
                        .strokes_below_y_pos(y_current_pos);
                }
                ToolStyle::DragProximity => {
                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .tools
                        .drag_proximity_tool
                        .pos = inputdata.pos();
                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .tools
                        .drag_proximity_tool
                        .offset = na::Vector2::zeros();
                }
            }
        }
    }

    fn motion(mut data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        if let Some(inputdata) = data_entries.pop_back() {
            let current_style = appwindow.canvas().pens().borrow().tools.style;

            match current_style {
                ToolStyle::ExpandSheet => {
                    let y_offset = inputdata.pos()[1]
                        - appwindow
                            .canvas()
                            .pens()
                            .borrow()
                            .tools
                            .expand_sheet_tool
                            .y_current_pos;

                    if y_offset.abs() > ExpandSheetTool::Y_OFFSET_THRESHOLD {
                        appwindow
                            .canvas()
                            .sheet()
                            .borrow_mut()
                            .strokes_state
                            .translate_strokes(
                                &appwindow
                                    .canvas()
                                    .pens()
                                    .borrow()
                                    .tools
                                    .expand_sheet_tool
                                    .strokes_below,
                                na::vector![0.0, y_offset],
                                appwindow.canvas().zoom(),
                            );

                        appwindow
                            .canvas()
                            .pens()
                            .borrow_mut()
                            .tools
                            .expand_sheet_tool
                            .y_current_pos = inputdata.pos()[1];
                    }
                }
                ToolStyle::DragProximity => {
                    let offset = inputdata.pos()
                        - appwindow
                            .canvas()
                            .pens()
                            .borrow()
                            .tools
                            .drag_proximity_tool
                            .pos;
                    appwindow
                        .canvas()
                        .pens()
                        .borrow_mut()
                        .tools
                        .drag_proximity_tool
                        .offset = offset;

                    if appwindow
                        .canvas()
                        .pens()
                        .borrow()
                        .tools
                        .drag_proximity_tool
                        .offset
                        .magnitude()
                        > DragProximityTool::OFFSET_MAGN_THRESHOLD
                    {
                        appwindow
                            .canvas()
                            .sheet()
                            .borrow_mut()
                            .strokes_state
                            .drag_strokes_proximity(
                                &appwindow.canvas().pens().borrow().tools.drag_proximity_tool,
                                appwindow.canvas().renderer(),
                                appwindow.canvas().zoom(),
                            );

                        appwindow
                            .canvas()
                            .pens()
                            .borrow_mut()
                            .tools
                            .drag_proximity_tool
                            .pos = inputdata.pos();
                        appwindow
                            .canvas()
                            .pens()
                            .borrow_mut()
                            .tools
                            .drag_proximity_tool
                            .offset = na::Vector2::zeros();
                    }
                }
            }
        }
    }

    fn end(_data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
        appwindow
            .canvas()
            .set_cursor(Some(&appwindow.canvas().cursor()));
        let current_style = appwindow.canvas().pens().borrow().tools.style;

        match current_style {
            ToolStyle::ExpandSheet => {
                appwindow
                    .canvas()
                    .pens()
                    .borrow_mut()
                    .tools
                    .expand_sheet_tool
                    .y_start_pos = 0.0;
                appwindow
                    .canvas()
                    .pens()
                    .borrow_mut()
                    .tools
                    .expand_sheet_tool
                    .y_current_pos = 0.0;
            }
            ToolStyle::DragProximity => {
                appwindow
                    .canvas()
                    .pens()
                    .borrow_mut()
                    .tools
                    .drag_proximity_tool
                    .pos = na::Vector2::zeros();
                appwindow
                    .canvas()
                    .pens()
                    .borrow_mut()
                    .tools
                    .drag_proximity_tool
                    .offset = na::Vector2::zeros();
            }
        }

        appwindow.canvas().update_size_autoexpand();

        appwindow.canvas().queue_resize();
        appwindow.canvas().queue_draw();
    }

    fn draw(
        &self,
        sheet_bounds: AABB,
        zoom: f64,
        snapshot: &Snapshot,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        match &self.style {
            ToolStyle::ExpandSheet => {
                self.expand_sheet_tool
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
            ToolStyle::DragProximity => {
                self.drag_proximity_tool
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
        }

        Ok(())
    }
}
