use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DrawStyle {
    Smooth,
    Rough,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineConfig {
    width: f64,
    pub color: Option<utils::Color>,
    pub fill: Option<utils::Color>,
}

impl Default for LineConfig {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            color: Self::COLOR_DEFAULT,
            fill: Self::FILL_DEFAULT,
        }
    }
}

impl LineConfig {
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
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RectangleConfig {
    width: f64,
    pub color: Option<utils::Color>,
    pub fill: Option<utils::Color>,
}

impl Default for RectangleConfig {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            color: Self::COLOR_DEFAULT,
            fill: Self::FILL_DEFAULT,
        }
    }
}

impl RectangleConfig {
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
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EllipseConfig {
    width: f64,
    pub color: Option<utils::Color>,
    pub fill: Option<utils::Color>,
}

impl Default for EllipseConfig {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            color: Self::COLOR_DEFAULT,
            fill: Self::FILL_DEFAULT,
        }
    }
}

impl EllipseConfig {
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
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shaper {
    pub current_shape: CurrentShape,
    pub drawstyle: DrawStyle,
    pub line_config: LineConfig,
    pub rectangle_config: RectangleConfig,
    pub ellipse_config: EllipseConfig,
    pub roughconfig: rough_rs::options::Options,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            current_shape: CurrentShape::default(),
            drawstyle: DrawStyle::default(),
            line_config: LineConfig::default(),
            rectangle_config: RectangleConfig::default(),
            ellipse_config: EllipseConfig::default(),
            roughconfig: rough_rs::options::Options::default(),
        }
    }
}

impl Shaper {
    pub fn apply_roughconfig_onto(&self, options: &mut rough_rs::options::Options) {
        options.roughness = self.roughconfig.roughness();
        options.bowing = self.roughconfig.bowing();
    }
}
