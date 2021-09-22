use serde::{Deserialize, Serialize};

use crate::strokes;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Marker {
    width: f64,
    color: strokes::Color,
}

impl Default for Marker {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            color: Self::COLOR_DEFAULT,
        }
    }
}

impl Marker {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 5.0;

    pub const COLOR_DEFAULT: strokes::Color = strokes::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }

    pub fn color(&self) -> strokes::Color {
        self.color
    }

    pub fn set_color(&mut self, color: strokes::Color) {
        self.color = color;
    }
}
