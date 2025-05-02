// Imports
use super::{PenStyle, PensConfig};
use serde::{Deserialize, Serialize};

/// The pen mode.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "pen_mode")]
pub enum PenMode {
    /// "Normal" pen mode.
    /// Usually the default "side" of a stylus, when no buttons are pressed.
    #[serde(rename = "pen")]
    #[default]
    Pen,
    /// Eraser mode.
    #[serde(rename = "eraser")]
    Eraser,
}

/// The pen mode state, holding the current mode and pen styles for all pen modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "pen_mode_state")]
pub struct PenModeState {
    #[serde(skip)]
    penmode_pen_style_override: Option<PenStyle>,
    #[serde(skip)]
    penmode_eraser_style_override: Option<PenStyle>,
}

impl Default for PenModeState {
    fn default() -> Self {
        Self {
            penmode_pen_style_override: None,
            penmode_eraser_style_override: None,
        }
    }
}

impl PenModeState {
    pub fn current_style_w_override(&self, config: &PensConfig) -> PenStyle {
        match config.penmode {
            PenMode::Pen => self
                .penmode_pen_style_override
                .unwrap_or(config.penmode_pen_style),
            PenMode::Eraser => self
                .penmode_eraser_style_override
                .unwrap_or(config.penmode_eraser_style),
        }
    }

    pub fn remove_all_overrides(&mut self) {
        self.penmode_pen_style_override = None;
        self.penmode_eraser_style_override = None;
    }

    pub fn style(&self, config: &PensConfig) -> PenStyle {
        match config.penmode {
            PenMode::Pen => config.penmode_pen_style,
            PenMode::Eraser => config.penmode_eraser_style,
        }
    }

    pub fn set_style(&mut self, config: &mut PensConfig, style: PenStyle) {
        match config.penmode {
            PenMode::Pen => config.penmode_pen_style = style,
            PenMode::Eraser => config.penmode_eraser_style = style,
        }
    }

    pub fn set_style_all_modes(&mut self, config: &mut PensConfig, style: PenStyle) {
        config.penmode_pen_style = style;
        config.penmode_eraser_style = style;
    }

    pub fn style_override(&self, config: &PensConfig) -> Option<PenStyle> {
        match config.penmode {
            PenMode::Pen => self.penmode_pen_style_override,
            PenMode::Eraser => self.penmode_eraser_style_override,
        }
    }

    pub fn set_style_override(&mut self, config: &PensConfig, style_override: Option<PenStyle>) {
        self.remove_all_overrides();

        match config.penmode {
            PenMode::Pen => {
                self.penmode_pen_style_override = style_override;
            }
            PenMode::Eraser => {
                self.penmode_eraser_style_override = style_override;
            }
        }
    }

    pub fn take_style_override(&mut self, config: &PensConfig) -> Option<PenStyle> {
        match config.penmode {
            PenMode::Pen => self.penmode_pen_style_override.take(),
            PenMode::Eraser => self.penmode_eraser_style_override.take(),
        }
    }

    pub fn pen_mode(&self, config: &PensConfig) -> PenMode {
        config.penmode
    }

    pub fn set_pen_mode(&mut self, config: &mut PensConfig, pen_mode: PenMode) {
        if config.penmode != pen_mode {
            self.remove_all_overrides();

            config.penmode = pen_mode;
        }
    }
}
