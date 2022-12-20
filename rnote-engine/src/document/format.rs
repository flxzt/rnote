use gtk4::glib;
use serde::{Deserialize, Serialize};

use rnote_compose::{color, Color};

#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "predefined_format")]
pub enum PredefinedFormat {
    #[serde(rename = "a6")]
    A6 = 0,
    #[serde(rename = "a5")]
    A5,
    #[serde(rename = "a4")]
    A4,
    #[serde(rename = "a3")]
    A3,
    #[serde(rename = "a2")]
    A2,
    #[serde(rename = "us_letter")]
    UsLetter,
    #[serde(rename = "us_legal")]
    UsLegal,
    #[serde(rename = "custom")]
    Custom,
}

impl Default for PredefinedFormat {
    fn default() -> Self {
        Self::A3
    }
}

impl TryFrom<u32> for PredefinedFormat {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "PredefinedFormat try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

impl PredefinedFormat {
    pub fn size_portrait_mm(&self) -> Option<(f64, f64)> {
        match self {
            PredefinedFormat::A6 => Some((105.0, 148.0)),
            PredefinedFormat::A5 => Some((148.0, 210.0)),
            PredefinedFormat::A4 => Some((210.0, 297.0)),
            PredefinedFormat::A3 => Some((297.0, 420.0)),
            PredefinedFormat::A2 => Some((420.0, 594.0)),
            PredefinedFormat::UsLetter => Some((215.9, 279.4)),
            PredefinedFormat::UsLegal => Some((215.9, 355.6)),
            PredefinedFormat::Custom => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "MeasureUnit")]
#[serde(rename = "measure_unit")]
pub enum MeasureUnit {
    #[enum_value(name = "Pixel", nick = "px")]
    #[serde(rename = "px")]
    Px = 0,
    #[enum_value(name = "Millimeter", nick = "mm")]
    #[serde(rename = "mm")]
    Mm,
    #[enum_value(name = "Centimeter", nick = "cm")]
    #[serde(rename = "cm")]
    Cm,
}

impl Default for MeasureUnit {
    fn default() -> Self {
        Self::Px
    }
}

impl MeasureUnit {
    pub const AMOUNT_MM_IN_INCH: f64 = 25.4;

    pub fn convert_measurement(
        value: f64,
        value_unit: MeasureUnit,
        value_dpi: f64,
        desired_unit: MeasureUnit,
        desired_dpi: f64,
    ) -> f64 {
        let value_in_px = match value_unit {
            MeasureUnit::Px => value,
            MeasureUnit::Mm => (value / Self::AMOUNT_MM_IN_INCH) * value_dpi,
            MeasureUnit::Cm => ((value * 10.0) / Self::AMOUNT_MM_IN_INCH) * value_dpi,
        };

        match desired_unit {
            MeasureUnit::Px => value_in_px,
            MeasureUnit::Mm => (value_in_px / desired_dpi) * Self::AMOUNT_MM_IN_INCH,
            MeasureUnit::Cm => (value_in_px / desired_dpi) * Self::AMOUNT_MM_IN_INCH * 10.0,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "FormatOrientation")]
#[serde(rename = "orientation")]
pub enum Orientation {
    #[enum_value(name = "Portrait", nick = "portrait")]
    #[serde(rename = "portrait")]
    Portrait = 0,
    #[enum_value(name = "Landscape", nick = "landscape")]
    #[serde(rename = "landscape")]
    Landscape,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "format")]
pub struct Format {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "height")]
    pub height: f64,
    #[serde(rename = "dpi")]
    pub dpi: f64,
    #[serde(rename = "orientation")]
    pub orientation: Orientation,
    #[serde(rename = "border_color")]
    pub border_color: Color,
    #[serde(rename = "show_borders")]
    pub show_borders: bool,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            height: Self::HEIGHT_DEFAULT,
            dpi: Self::DPI_DEFAULT,
            orientation: Orientation::default(),
            border_color: Color::from(Self::BORDER_COLOR_DEFAULT),
            show_borders: true,
        }
    }
}

impl Format {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 30000.0;
    pub const WIDTH_DEFAULT: f64 = 1123.0;

    pub const HEIGHT_MIN: f64 = 1.0;
    pub const HEIGHT_MAX: f64 = 30000.0;
    pub const HEIGHT_DEFAULT: f64 = 1587.0;

    pub const DPI_MIN: f64 = 1.0;
    pub const DPI_MAX: f64 = 5000.0;
    pub const DPI_DEFAULT: f64 = 96.0;

    pub const BORDER_COLOR_DEFAULT: piet::Color = color::GNOME_BRIGHTS[2];
}
