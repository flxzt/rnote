// Modules
pub mod brushconfig;
pub mod eraserconfig;
pub mod selectorconfig;
pub mod shaperconfig;
pub mod toolsconfig;
pub mod typewriterconfig;

// Re-exports
pub use brushconfig::BrushConfig;
pub use eraserconfig::EraserConfig;
pub use selectorconfig::SelectorConfig;
pub use shaperconfig::ShaperConfig;
pub use toolsconfig::ToolsConfig;
pub use typewriterconfig::TypewriterConfig;

// Imports
use crate::CloneConfig;
use rnote_compose::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename = "pens_config")]
pub struct PensConfig {
    #[serde(default, rename = "brush_config")]
    pub brush_config: BrushConfig,
    #[serde(default, rename = "shaper_config")]
    pub shaper_config: ShaperConfig,
    #[serde(default, rename = "typewriter_config")]
    pub typewriter_config: TypewriterConfig,
    #[serde(default, rename = "eraser_config")]
    pub eraser_config: EraserConfig,
    #[serde(default, rename = "selector_config")]
    pub selector_config: SelectorConfig,
    #[serde(default, rename = "tools_config")]
    pub tools_config: ToolsConfig,
}

impl CloneConfig for PensConfig {
    fn clone_config(&self) -> Self {
        self.clone()
    }
}

impl PensConfig {
    pub fn set_all_stroke_colors(&mut self, stroke_color: Color) {
        self.brush_config.marker_options.stroke_color = Some(stroke_color);
        self.brush_config.solid_options.stroke_color = Some(stroke_color);
        self.brush_config.textured_options.stroke_color = Some(stroke_color);
        self.shaper_config.smooth_options.stroke_color = Some(stroke_color);
        self.shaper_config.rough_options.stroke_color = Some(stroke_color);
        self.typewriter_config.text_style.color = stroke_color;
    }

    pub fn set_all_fill_colors(&mut self, fill_color: Color) {
        self.brush_config.marker_options.fill_color = Some(fill_color);
        self.brush_config.solid_options.fill_color = Some(fill_color);
        self.shaper_config.smooth_options.fill_color = Some(fill_color);
        self.shaper_config.rough_options.fill_color = Some(fill_color);
    }
}
