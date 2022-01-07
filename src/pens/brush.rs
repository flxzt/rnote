use crate::utils;

use gtk4::gdk;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BrushStyle {
    Linear,
    CubicBezier,
    Experimental,
}

impl Default for BrushStyle {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Brush {
    width: f64,
    sensitivity: f64,
    pub color: utils::Color,
    pub current_style: BrushStyle,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            sensitivity: Self::SENSITIVITY_DEFAULT,
            color: utils::Color::from(Self::COLOR_DEFAULT),
            current_style: BrushStyle::default(),
        }
    }
}

impl Brush {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 6.0;
    pub const SENSITIVITY_MIN: f64 = 0.0;
    pub const SENSITIVITY_MAX: f64 = 1.0;
    pub const SENSITIVITY_DEFAULT: f64 = 0.5;

    pub const TEMPLATE_BOUNDS_PADDING: f64 = 50.0;

    pub const COLOR_DEFAULT: gdk::RGBA = gdk::RGBA {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }

    pub fn sensitivity(&self) -> f64 {
        self.sensitivity
    }

    pub fn set_sensitivity(&mut self, sensitivity: f64) {
        self.sensitivity = sensitivity.clamp(Self::SENSITIVITY_MIN, Self::SENSITIVITY_MAX);
    }
}
