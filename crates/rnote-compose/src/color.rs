// Imports
use palette::{
    convert::{FromColorUnclamped, IntoColorUnclamped},
    IntoColor,
};
use serde::{Deserialize, Serialize};

/// The threshold of the luminance of a color, deciding if a light or dark fg color is used. Between 0.0 and 1.0.
pub const FG_LUMINANCE_THRESHOLD: f64 = 0.7;

/// A rgba color
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    palette::convert::FromColorUnclamped,
    palette::WithAlpha,
)]
#[palette(
    skip_derives(Rgb),
    component = "f64",
    rgb_standard = "palette::encoding::Srgb"
)]
#[serde(default, rename = "color")]
pub struct Color {
    /// Red, ranging [0.0, 1.0].
    #[serde(rename = "r", with = "crate::serialize::f64_dp3")]
    pub r: f64,
    /// Green, ranging [0.0, 1.0].
    #[serde(rename = "g", with = "crate::serialize::f64_dp3")]
    pub g: f64,
    /// Blue, ranging [0.0, 1.0].
    #[serde(rename = "b", with = "crate::serialize::f64_dp3")]
    pub b: f64,
    /// Alpha, ranging [0.0, 1.0].
    #[palette(alpha)]
    #[serde(rename = "a", with = "crate::serialize::f64_dp3")]
    pub a: f64,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    /// Transparent color with r,g,b set to 0.0.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Black color.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// White color.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Red color.
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Green color.
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// Blue color.
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// A new color from rgba values.
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Approximate equality.
    pub fn approx_eq(self, other: Self) -> bool {
        approx::relative_eq!(self.r, other.r)
            && approx::relative_eq!(self.g, other.g)
            && approx::relative_eq!(self.b, other.b)
            && approx::relative_eq!(self.a, other.a)
    }

    /// Approximate equality.
    pub fn approx_eq_f32(self, other: Self) -> bool {
        approx::relative_eq!(self.r as f32, other.r as f32)
            && approx::relative_eq!(self.g as f32, other.g as f32)
            && approx::relative_eq!(self.b as f32, other.b as f32)
            && approx::relative_eq!(self.a as f32, other.a as f32)
    }

    /// The luma value, ranging [0.0 - 1.0].
    ///
    /// see: <https://en.wikipedia.org/wiki/Luma_(video)>
    pub fn luma(&self) -> f64 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Invert the perceived brightness of the color.
    pub fn to_inverted_brightness_color(self) -> Self {
        let mut hwba_color: palette::Okhwba<f64> = self.into_color();

        std::mem::swap(
            &mut hwba_color.color.whiteness,
            &mut hwba_color.color.blackness,
        );

        hwba_color.into_color()
    }

    /// Get the original or the inverted color, depending on which one is darker.
    pub fn to_darkest_color(self) -> Self {
        let inverted_color = self.to_inverted_brightness_color();

        if inverted_color.luma() > self.luma() {
            self
        } else {
            inverted_color
        }
    }

    /// Convert to a css color attribute in the style: `rgba(xxx,xxx,xxx,xxx)`.
    /// The values are 8 bit integers, ranging [0, 255].
    pub fn to_css_color_attr(self) -> String {
        format!(
            "rgba({:03},{:03},{:03},{:.3})",
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

impl From<roughr::Srgba> for Color {
    fn from(c: roughr::Srgba) -> Self {
        Self {
            r: c.blue as f64,
            g: c.green as f64,
            b: c.blue as f64,
            a: c.alpha as f64,
        }
    }
}

impl From<Color> for roughr::Srgba {
    fn from(c: Color) -> Self {
        roughr::Srgba::new(c.r as f32, c.g as f32, c.b as f32, c.a as f32)
    }
}

// Conversion function for (opaque) RGB to Color. `impl_default_conversions` take care of preserving the transparency.
impl<S> palette::convert::FromColorUnclamped<palette::rgb::Rgb<S, f64>> for Color
where
    palette::Srgb<f64>: FromColorUnclamped<palette::rgb::Rgb<S, f64>>,
{
    fn from_color_unclamped(color: palette::rgb::Rgb<S, f64>) -> Color {
        let srgb = palette::Srgb::from_color_unclamped(color).into_format();

        Color {
            r: srgb.red,
            g: srgb.green,
            b: srgb.blue,
            a: 1.0,
        }
    }
}

// Conversion function for Color to (opaque) RGB. `impl_default_conversions` take care of preserving the transparency.
impl<S> palette::convert::FromColorUnclamped<Color> for palette::rgb::Rgb<S, f64>
where
    palette::Srgb<f64>: IntoColorUnclamped<palette::rgb::Rgb<S, f64>>,
{
    fn from_color_unclamped(color: Color) -> palette::rgb::Rgb<S, f64> {
        palette::Srgb::new(color.r, color.g, color.b)
            .into_format()
            .into_color_unclamped()
    }
}

impl palette::Clamp for Color {
    fn clamp(self) -> Self {
        // The constructor clamps components to [0.0, 1.0].
        Color::new(self.r, self.g, self.b, self.a)
    }
}

/// Gnome palette blues.
pub const GNOME_BLUES: [piet::Color; 5] = [
    piet::Color::rgb8(0x99, 0xc1, 0xf1),
    piet::Color::rgb8(0x62, 0xa0, 0xea),
    piet::Color::rgb8(0x35, 0x84, 0xe4),
    piet::Color::rgb8(0x1c, 0x71, 0xd8),
    piet::Color::rgb8(0x1a, 0x5f, 0xb4),
];

/// Gnome palette greens.
pub const GNOME_GREENS: [piet::Color; 5] = [
    piet::Color::rgb8(0x8f, 0xf0, 0xa4),
    piet::Color::rgb8(0x57, 0xe3, 0x89),
    piet::Color::rgb8(0x33, 0xd1, 0x7a),
    piet::Color::rgb8(0x2e, 0xc2, 0x7e),
    piet::Color::rgb8(0x26, 0xa2, 0x69),
];

/// Gnome palette yellows.
pub const GNOME_YELLOWS: [piet::Color; 5] = [
    piet::Color::rgb8(0xf9, 0xf0, 0x6b),
    piet::Color::rgb8(0xf8, 0xe4, 0x5c),
    piet::Color::rgb8(0xf6, 0xd3, 0x2d),
    piet::Color::rgb8(0xf5, 0xc2, 0x11),
    piet::Color::rgb8(0xe5, 0xa5, 0x0a),
];

/// Gnome palette oranges.
pub const GNOME_ORANGES: [piet::Color; 5] = [
    piet::Color::rgb8(0xff, 0xbe, 0x6f),
    piet::Color::rgb8(0xff, 0xa3, 0x48),
    piet::Color::rgb8(0xff, 0x78, 0x00),
    piet::Color::rgb8(0xe6, 0x61, 0x00),
    piet::Color::rgb8(0xc6, 0x46, 0x00),
];

/// Gnome palette reds.
pub const GNOME_REDS: [piet::Color; 5] = [
    piet::Color::rgb8(0xf6, 0x61, 0x51),
    piet::Color::rgb8(0xed, 0x33, 0x3b),
    piet::Color::rgb8(0xe0, 0x1b, 0x24),
    piet::Color::rgb8(0xc0, 0x1c, 0x28),
    piet::Color::rgb8(0xa5, 0x1d, 0x2d),
];

/// Gnome palette purples.
pub const GNOME_PURPLES: [piet::Color; 5] = [
    piet::Color::rgb8(0xdc, 0x8a, 0xdd),
    piet::Color::rgb8(0xc0, 0x61, 0xcb),
    piet::Color::rgb8(0x91, 0x41, 0xac),
    piet::Color::rgb8(0x81, 0x3d, 0x9c),
    piet::Color::rgb8(0x61, 0x35, 0x83),
];

/// Gnome palette browns.
pub const GNOME_BROWNS: [piet::Color; 5] = [
    piet::Color::rgb8(0xcd, 0xab, 0x8f),
    piet::Color::rgb8(0xb5, 0x83, 0x5a),
    piet::Color::rgb8(0x98, 0x6a, 0x44),
    piet::Color::rgb8(0x86, 0x5e, 0x3c),
    piet::Color::rgb8(0x63, 0x45, 0x2c),
];

/// Gnome palette brights.
pub const GNOME_BRIGHTS: [piet::Color; 5] = [
    piet::Color::rgb8(0xff, 0xff, 0xff),
    piet::Color::rgb8(0xf6, 0xf5, 0xf4),
    piet::Color::rgb8(0xde, 0xdd, 0xda),
    piet::Color::rgb8(0xc0, 0xbf, 0xbc),
    piet::Color::rgb8(0x9a, 0x99, 0x96),
];

/// Gnome palette darks.
pub const GNOME_DARKS: [piet::Color; 5] = [
    piet::Color::rgb8(0x77, 0x76, 0x7b),
    piet::Color::rgb8(0x5e, 0x5c, 0x64),
    piet::Color::rgb8(0x3d, 0x38, 0x46),
    piet::Color::rgb8(0x24, 0x1f, 0x31),
    piet::Color::rgb8(0x00, 0x00, 0x00),
];
