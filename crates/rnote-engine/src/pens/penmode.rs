use crate::CloneConfig;

// Imports
use super::PenStyle;
use serde::{Deserialize, Serialize};

/// The pen mode.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "pen_mode")]
pub enum PenMode {
    /// "Normal" pen mode.
    /// Usually the default "side" of a stylus, when no buttons are pressed.
    #[serde(rename = "pen")]
    Pen,
    /// Eraser mode.
    #[serde(rename = "eraser")]
    Eraser,
}

/// The pen mode state, holding the current mode and pen styles for all pen modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "pen_mode_state")]
pub struct PenModeState {
    #[serde(rename = "pen_mode")]
    pen_mode: PenMode,
    #[serde(rename = "penmode_pen_style")]
    penmode_pen_style: PenStyle,
    #[serde(rename = "penmode_eraser_style")]
    penmode_eraser_style: PenStyle,

    #[serde(skip)]
    penmode_pen_style_override: Option<PenStyle>,
    #[serde(skip)]
    penmode_eraser_style_override: Option<PenStyle>,
}

impl Default for PenModeState {
    fn default() -> Self {
        Self {
            pen_mode: PenMode::Pen,
            penmode_pen_style: PenStyle::Brush,
            penmode_eraser_style: PenStyle::Eraser,

            penmode_pen_style_override: None,
            penmode_eraser_style_override: None,
        }
    }
}

impl CloneConfig for PenModeState {
    fn clone_config(&self) -> Self {
        Self {
            pen_mode: self.pen_mode,
            penmode_pen_style: self.penmode_pen_style,
            penmode_eraser_style: self.penmode_eraser_style,
            ..Default::default()
        }
    }
}

impl PenModeState {
    pub fn current_style_w_override(&self) -> PenStyle {
        match self.pen_mode {
            PenMode::Pen => self
                .penmode_pen_style_override
                .unwrap_or(self.penmode_pen_style),
            PenMode::Eraser => self
                .penmode_eraser_style_override
                .unwrap_or(self.penmode_eraser_style),
        }
    }

    pub fn remove_all_overrides(&mut self) {
        self.penmode_pen_style_override = None;
        self.penmode_eraser_style_override = None;
    }

    pub fn style(&self) -> PenStyle {
        match self.pen_mode {
            PenMode::Pen => self.penmode_pen_style,
            PenMode::Eraser => self.penmode_eraser_style,
        }
    }

    pub fn set_style(&mut self, style: PenStyle) {
        match self.pen_mode {
            PenMode::Pen => self.penmode_pen_style = style,
            PenMode::Eraser => self.penmode_eraser_style = style,
        }
    }

    pub fn set_style_all_modes(&mut self, style: PenStyle) {
        self.penmode_pen_style = style;
        self.penmode_eraser_style = style;
    }

    pub fn style_override(&self) -> Option<PenStyle> {
        match self.pen_mode {
            PenMode::Pen => self.penmode_pen_style_override,
            PenMode::Eraser => self.penmode_eraser_style_override,
        }
    }

    pub fn set_style_override(&mut self, style_override: Option<PenStyle>) {
        self.remove_all_overrides();

        match self.pen_mode {
            PenMode::Pen => {
                self.penmode_pen_style_override = style_override;
            }
            PenMode::Eraser => {
                self.penmode_eraser_style_override = style_override;
            }
        }
    }

    pub fn take_style_override(&mut self) -> Option<PenStyle> {
        match self.pen_mode {
            PenMode::Pen => self.penmode_pen_style_override.take(),
            PenMode::Eraser => self.penmode_eraser_style_override.take(),
        }
    }

    pub fn pen_mode(&self) -> PenMode {
        self.pen_mode
    }

    pub fn set_pen_mode(&mut self, pen_mode: PenMode) {
        if self.pen_mode != pen_mode {
            self.remove_all_overrides();

            self.pen_mode = pen_mode;
        }
    }
}
