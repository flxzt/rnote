use rnote_fileformats::xoppformat;

use serde::{Deserialize, Serialize};

/// A rgba color
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "color")]
pub struct Color {
    /// red, ranging [0.0, 1.0]
    #[serde(rename = "r")]
    pub r: f64,
    /// green, ranging [0.0, 1.0]
    #[serde(rename = "g")]
    pub g: f64,
    /// blue, ranging [0.0, 1.0]
    #[serde(rename = "b")]
    pub b: f64,
    /// alpha, ranging [0.0, 1.0]
    #[serde(rename = "a")]
    pub a: f64,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    /// Transparent color with rgb set to 0.0
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Black color
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// White color
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Red color
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Green color
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// Blue color
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// A new color from rgba values
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Returns the red part
    pub fn r(&self) -> f64 {
        self.r
    }

    /// Returns the green part
    pub fn g(&self) -> f64 {
        self.g
    }

    /// Returns the blue part
    pub fn b(&self) -> f64 {
        self.b
    }

    /// Returns the alpha part
    pub fn a(&self) -> f64 {
        self.a
    }

    /// converts to a css color attribute in the style: `rgb(xxx,xxx,xxx,xxx)`. The values are 8 bit integers, ranging [0, 255]
    pub fn to_css_color_attr(self) -> String {
        format!(
            "rgb({:03},{:03},{:03},{:.3})",
            (self.r * 255.0) as i32,
            (self.g * 255.0) as i32,
            (self.b * 255.0) as i32,
            ((1000.0 * self.a).round() / 1000.0),
        )
    }
}

impl From<piet::Color> for Color {
    fn from(piet_color: piet::Color) -> Self {
        let piet_rgba = piet_color.as_rgba();
        Self {
            r: piet_rgba.0,
            g: piet_rgba.1,
            b: piet_rgba.2,
            a: piet_rgba.3,
        }
    }
}

impl From<Color> for piet::Color {
    fn from(color: Color) -> Self {
        piet::Color::rgba(color.r, color.g, color.b, color.a)
    }
}

impl From<(f64, f64, f64, f64)> for Color {
    fn from(tuple: (f64, f64, f64, f64)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
            a: tuple.3,
        }
    }
}

impl From<Color> for (f64, f64, f64, f64) {
    fn from(color: Color) -> Self {
        (color.r, color.g, color.b, color.a)
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self {
            r: f64::from((value >> 24) & 0xff) / 255.0,
            g: f64::from((value >> 16) & 0xff) / 255.0,
            b: f64::from((value >> 8) & 0xff) / 255.0,
            a: f64::from((value) & 0xff) / 255.0,
        }
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        ((((color.r * 255.0).round() as u32) & 0xff) << 24)
            | ((((color.g * 255.0).round() as u32) & 0xff) << 16)
            | ((((color.b * 255.0).round() as u32) & 0xff) << 8)
            | (((color.a * 255.0).round() as u32) & 0xff)
    }
}

impl From<xoppformat::XoppColor> for Color {
    fn from(xopp_color: xoppformat::XoppColor) -> Self {
        Self {
            r: f64::from(xopp_color.red) / 255.0,
            g: f64::from(xopp_color.green) / 255.0,
            b: f64::from(xopp_color.blue) / 255.0,
            a: f64::from(xopp_color.alpha) / 255.0,
        }
    }
}

impl From<Color> for xoppformat::XoppColor {
    fn from(color: Color) -> Self {
        xoppformat::XoppColor {
            red: (color.r * 255.0).floor() as u8,
            green: (color.g * 255.0).floor() as u8,
            blue: (color.b * 255.0).floor() as u8,
            alpha: (color.a * 255.0).floor() as u8,
        }
    }
}
