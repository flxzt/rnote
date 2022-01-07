use crate::render::Renderer;
use crate::strokes::strokestyle::InputData;
use crate::ui::appwindow::RnoteAppWindow;
use crate::{compose, geometry, render, utils};

use gtk4::Snapshot;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ToolStyle {
    ExpandSheet,
    ModifyStroke,
}

impl Default for ToolStyle {
    fn default() -> Self {
        Self::ExpandSheet
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpandSheetTool {
    y_threshold: f64,
    offset: f64,
}

impl Default for ExpandSheetTool {
    fn default() -> Self {
        Self {
            y_threshold: 0.0,
            offset: 0.0,
        }
    }
}

impl ExpandSheetTool {
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

    pub fn new_y_threshold(&mut self, y_threshold: f64) {
        self.y_threshold = y_threshold;
    }

    pub fn new_offset(&mut self, offset: f64) {
        self.offset = offset;
    }

    pub fn draw(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        let x = sheet_bounds.mins[0];
        let y = self.y_threshold;
        let width = sheet_bounds.extents()[0];
        let height = self.offset;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModifyStrokeTool {
    input: Vec<InputData>,
}

impl Default for ModifyStrokeTool {
    fn default() -> Self {
        Self { input: vec![] }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Tools {
    current_style: ToolStyle,
    pub expand_sheet_tool: ExpandSheetTool,
    pub modify_stroke_tool: ModifyStrokeTool,
}

impl PenBehaviour for Tools {
    fn begin(&mut self, inputdata: InputData) {
        match &mut self.current_style {
            ToolStyle::ExpandSheet => {
                self.expand_sheet_tool.y_threshold = inputdata.pos()[1];
                self.expand_sheet_tool.offset = 0.0;
            }
            ToolStyle::ModifyStroke => {}
        }
    }

    fn update(&mut self, inputdata: InputData) {
        match &mut self.current_style {
            ToolStyle::ExpandSheet => {
                self.expand_sheet_tool.offset =
                    inputdata.pos()[1] - self.expand_sheet_tool.y_threshold;
            }
            ToolStyle::ModifyStroke => {}
        }
    }

    fn apply(&mut self, appwindow: &RnoteAppWindow) {
        match &mut self.current_style {
            ToolStyle::ExpandSheet => {
                appwindow
                    .canvas()
                    .sheet()
                    .strokes_state()
                    .borrow_mut()
                    .translate_strokes_threshold_vertical(
                        self.expand_sheet_tool.y_threshold,
                        self.expand_sheet_tool.offset,
                    );
            }
            ToolStyle::ModifyStroke => {}
        }
    }

    fn reset(&mut self) {
        match &mut self.current_style {
            ToolStyle::ExpandSheet => {
                self.expand_sheet_tool.y_threshold = 0.0;
                self.expand_sheet_tool.offset = 0.0;
            }
            ToolStyle::ModifyStroke => {}
        }
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
            ToolStyle::ModifyStroke => {}
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
