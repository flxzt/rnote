use crate::render;
use crate::strokes::strokestyle::InputData;

use gtk4::{gsk};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ToolStyle {
    ExpandSheet(ExpandSheetTool),
    ModifyStroke(ModifyStrokeTool),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpandSheetTool {
    input: Vec<InputData>,
}

impl Default for ExpandSheetTool {
    fn default() -> Self {
        Self { input: vec![] }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tools {
    style: ToolStyle,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
    shown: bool,
}

impl Default for Tools {
    fn default() -> Self {
        Self {
            style: ToolStyle::ExpandSheet(ExpandSheetTool::default()),
            rendernode: render::default_rendernode(),
            shown: false,
        }
    }
}

impl Tools {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(&self) -> ToolStyle {
        self.style.clone()
    }

    pub fn set_style(&mut self, style: ToolStyle) {
        self.style = style;
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn tool_begin(&mut self, _inputdata: InputData) {

    }

    pub fn add_input_to_tool(&mut self, _inputdata: InputData) {

    }

    pub fn evaluate_tool(&mut self) {

    }
}
