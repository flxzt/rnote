use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CurrentShape {
    Line,
    Rectangle,
    Ellipse,
}

impl Default for CurrentShape {
    fn default() -> Self {
        Self::Rectangle
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DrawStyle {
    Smooth,
    Rough,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

impl DrawStyle {
    pub const ROUGH_MARGIN: f64 = 8.0;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Shaper {
    pub current_shape: CurrentShape,
    pub drawstyle: DrawStyle,
    width: f64,
    color: Option<utils::Color>,
    fill: Option<utils::Color>,
    pub roughconfig: rough_rs::options::Options,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            current_shape: CurrentShape::default(),
            drawstyle: DrawStyle::default(),
            width: Shaper::WIDTH_DEFAULT,
            color: Shaper::COLOR_DEFAULT,
            fill: Shaper::FILL_DEFAULT,
            roughconfig: rough_rs::options::Options::default(),
        }
    }
}

impl Shaper {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 2.0;

    pub const COLOR_DEFAULT: Option<utils::Color> = Some(utils::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    pub const FILL_DEFAULT: Option<utils::Color> = None;

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Shaper::WIDTH_MIN, Shaper::WIDTH_MAX);
    }

    pub fn color(&self) -> Option<utils::Color> {
        self.color
    }

    pub fn set_color(&mut self, color: Option<utils::Color>) {
        self.color = color;
    }

    pub fn fill(&self) -> Option<utils::Color> {
        self.fill
    }

    pub fn set_fill(&mut self, fill: Option<utils::Color>) {
        self.fill = fill;
    }

    pub fn apply_roughconfig_onto(&self, options: &mut rough_rs::options::Options) {
        options.roughness = self.roughconfig.roughness();
        options.bowing = self.roughconfig.bowing();
    }
}
