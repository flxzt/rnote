use crate::render::Renderer;
use crate::strokes::strokestyle::InputData;
use crate::strokes::StrokeKey;
use crate::ui::appwindow::RnoteAppWindow;
use crate::{compose, geometry, render, utils};

use gtk4::{prelude::*, Snapshot};

use super::penbehaviour::PenBehaviour;

#[derive(Clone, Debug)]
pub enum ToolStyle {
    ExpandSheet,
    DragProximity,
}

impl Default for ToolStyle {
    fn default() -> Self {
        Self::ExpandSheet
    }
}

#[derive(Clone, Debug)]
pub struct ExpandSheetTool {
    y_start_pos: f64,
    y_current_pos: f64,
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
    pub const FILL_COLOR: utils::Color = utils::Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 0.2,
    };
    pub const THRESHOLD_LINE_COLOR: utils::Color = utils::Color {
        r: 0.5,
        g: 0.7,
        b: 0.7,
        a: 1.0,
    };
    pub const THRESHOLD_LINE_STROKE_WIDTH: f64 = 10.0;
    pub const OFFSET_LINE_COLOR: utils::Color = utils::Color {
        r: 0.0,
        g: 0.7,
        b: 1.0,
        a: 1.0,
    };
    pub const OFFSET_LINE_STROKE_WIDTH: f64 = 2.0;

    pub fn draw(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        let x = sheet_bounds.mins[0];
        let y = self.y_start_pos;
        let width = sheet_bounds.extents()[0];
        let height = self.y_current_pos - self.y_start_pos;
        let bounds =
            geometry::aabb_new_positive(na::vector![x, y], na::vector![x + width, y + height]);

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

        let mut svg_data = rough_rs::node_to_string(&group)?;

        svg_data = compose::wrap_svg(&svg_data, Some(bounds), Some(bounds), true, false);
        let svg = render::Svg { svg_data, bounds };

        let image = renderer.gen_image(zoom, &[svg], bounds)?;
        let rendernode = render::image_to_rendernode(&image, zoom);
        snapshot.append_node(&rendernode);

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct DragProximityTool {
    pub pos: na::Vector2<f64>,
    pub offset: na::Vector2<f64>,
    pub radius: f64,
}

impl Default for DragProximityTool {
    fn default() -> Self {
        Self {
            pos: na::Vector2::<f64>::zeros(),
            offset: na::Vector2::<f64>::zeros(),
            radius: Self::RADIUS_DEFAULT,
        }
    }
}

impl DragProximityTool {
    pub const OFFSET_MAGN_THRESHOLD: f64 = 4.0;
    pub const OUTLINE_COLOR: utils::Color = utils::Color {
        r: 0.5,
        g: 0.7,
        b: 0.7,
        a: 1.0,
    };
    pub const OUTLINE_WIDTH: f64 = 1.0;
    pub const FILL_COLOR: utils::Color = utils::Color {
        r: 0.8,
        g: 0.8,
        b: 0.8,
        a: 0.2,
    };
    pub const RADIUS_DEFAULT: f64 = 60.0;

    pub fn draw(
        &self,
        _sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        let cx = self.pos[0] + self.offset[0];
        let cy = self.pos[1] + self.offset[1];
        let r = self.radius;
        let mut draw_bounds = geometry::aabb_new_positive(
            na::vector![cx - r - Self::OUTLINE_WIDTH, cy - r - Self::OUTLINE_WIDTH],
            na::vector![cx + r + Self::OUTLINE_WIDTH, cy + r + Self::OUTLINE_WIDTH],
        );
        draw_bounds.take_point(na::Point2::<f64>::from(
            self.pos.add_scalar(-Self::OUTLINE_WIDTH),
        ));
        draw_bounds.take_point(na::Point2::<f64>::from(
            self.pos.add_scalar(Self::OUTLINE_WIDTH),
        ));

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

        let mut svg_data = rough_rs::node_to_string(&group)?;

        svg_data = compose::wrap_svg(&svg_data, Some(draw_bounds), Some(draw_bounds), true, false);
        let svg = render::Svg {
            svg_data,
            bounds: draw_bounds,
        };

        let image = renderer.gen_image(zoom, &[svg], draw_bounds)?;
        let rendernode = render::image_to_rendernode(&image, zoom);
        snapshot.append_node(&rendernode);

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Tools {
    current_style: ToolStyle,
    pub expand_sheet_tool: ExpandSheetTool,
    pub drag_proximity_tool: DragProximityTool,
}

impl PenBehaviour for Tools {
    fn begin(&mut self, inputdata: InputData, appwindow: &RnoteAppWindow) {
        match &mut self.current_style {
            ToolStyle::ExpandSheet => {
                self.expand_sheet_tool.y_start_pos = inputdata.pos()[1];
                self.expand_sheet_tool.y_current_pos = inputdata.pos()[1];

                self.expand_sheet_tool.strokes_below = appwindow
                    .canvas()
                    .sheet()
                    .strokes_state()
                    .borrow_mut()
                    .strokes_below_y_pos(self.expand_sheet_tool.y_current_pos);
            }
            ToolStyle::DragProximity => {
                self.drag_proximity_tool.pos = inputdata.pos();
                self.drag_proximity_tool.offset = na::Vector2::<f64>::zeros();
            }
        }
    }

    fn motion(&mut self, inputdata: InputData, appwindow: &RnoteAppWindow) {
        match &mut self.current_style {
            ToolStyle::ExpandSheet => {
                let y_offset = inputdata.pos()[1] - self.expand_sheet_tool.y_current_pos;

                if y_offset.abs() > ExpandSheetTool::Y_OFFSET_THRESHOLD {
                    appwindow
                        .canvas()
                        .sheet()
                        .strokes_state()
                        .borrow_mut()
                        .translate_strokes(&self.expand_sheet_tool.strokes_below, na::vector![0.0, y_offset]);

                    self.expand_sheet_tool.y_current_pos = inputdata.pos()[1];
                }
            }
            ToolStyle::DragProximity => {
                self.drag_proximity_tool.offset = inputdata.pos() - self.drag_proximity_tool.pos;

                if self.drag_proximity_tool.offset.magnitude()
                    > DragProximityTool::OFFSET_MAGN_THRESHOLD
                {
                    appwindow
                        .canvas()
                        .sheet()
                        .strokes_state()
                        .borrow_mut()
                        .drag_strokes_proximity(&self.drag_proximity_tool);

                    self.drag_proximity_tool.pos = inputdata.pos();
                    self.drag_proximity_tool.offset = na::Vector2::<f64>::zeros();
                }
            }
        }
    }

    fn end(&mut self, inputdata: InputData, appwindow: &RnoteAppWindow) {
        match &mut self.current_style {
            ToolStyle::ExpandSheet => {
                self.expand_sheet_tool.y_start_pos = inputdata.pos()[1];
                self.expand_sheet_tool.y_current_pos = 0.0;
            }
            ToolStyle::DragProximity => {
                self.drag_proximity_tool.pos = inputdata.pos();
                self.drag_proximity_tool.offset = na::Vector2::<f64>::zeros();
            }
        }

        if appwindow.canvas().sheet().resize_endless() {
            appwindow.canvas().update_background_rendernode();
        }

        appwindow.canvas().queue_resize();
        appwindow.canvas().queue_draw();
    }

    fn draw(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        match &self.current_style {
            ToolStyle::ExpandSheet => {
                self.expand_sheet_tool
                    .draw(sheet_bounds, renderer, zoom, snapshot)?;
            }
            ToolStyle::DragProximity => {
                self.drag_proximity_tool
                    .draw(sheet_bounds, renderer, zoom, snapshot)?;
            }
        }

        Ok(())
    }
}

impl Tools {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(&self) -> ToolStyle {
        self.current_style.clone()
    }

    pub fn set_style(&mut self, style: ToolStyle) {
        self.current_style = style;
    }
}
