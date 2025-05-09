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
use super::shortcuts::ShortcutAction;
use super::{PenStyle, Shortcuts};
use rnote_compose::Color;
use rnote_compose::penevent::ShortcutKey;
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

    #[serde(rename = "shortcuts")]
    pub shortcuts: Shortcuts,
    #[serde(rename = "pen_mode_pen_style")]
    pub pen_mode_pen_style: PenStyle,
    #[serde(rename = "pen_mode_eraser_style")]
    pub pen_mode_eraser_style: PenStyle,
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

    /// Get the current registered shortcuts.
    pub fn shortcuts(&self) -> Shortcuts {
        self.shortcuts.clone()
    }

    /// Clear all shortcuts
    pub fn clear_shortcuts(&mut self) {
        self.shortcuts.clear();
    }

    /// Replace all shortcuts.
    pub fn set_shortcuts(&mut self, shortcuts: Shortcuts) {
        self.shortcuts = shortcuts;
    }

    /// Register a shortcut key and action.
    pub fn register_shortcut(&mut self, key: ShortcutKey, action: ShortcutAction) {
        self.shortcuts.insert(key, action);
    }

    /// Remove the shortcut action for the given shortcut key, if it is registered.
    pub fn remove_shortcut(&mut self, key: ShortcutKey) -> Option<ShortcutAction> {
        self.shortcuts.remove(&key)
    }

    // Get the current registered action the the given shortcut key.
    pub fn get_shortcut_action(&self, key: ShortcutKey) -> Option<ShortcutAction> {
        self.shortcuts.get(&key).cloned()
    }

    /// List all current registered shortcut keys and their action.
    pub fn list_current_shortcuts(&self) -> Vec<(ShortcutKey, ShortcutAction)> {
        self.shortcuts
            .iter()
            .map(|(key, action)| (*key, *action))
            .collect()
    }
}
