// Imports
use rnote_compose::{color, Color};
use serde::{Deserialize, Serialize};

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
    pub fn size_mm(&self, orientation: Orientation) -> Option<na::Vector2<f64>> {
        let mut size_portrait = match self {
            PredefinedFormat::A6 => Some((105.0, 148.0)),
            PredefinedFormat::A5 => Some((148.0, 210.0)),
            PredefinedFormat::A4 => Some((210.0, 297.0)),
            PredefinedFormat::A3 => Some((297.0, 420.0)),
            PredefinedFormat::A2 => Some((420.0, 594.0)),
            PredefinedFormat::UsLetter => Some((215.9, 279.4)),
            PredefinedFormat::UsLegal => Some((215.9, 355.6)),
            PredefinedFormat::Custom => None,
        };
        if let Some((mut width, mut height)) = &mut size_portrait {
            if orientation == Orientation::Landscape {
                std::mem::swap(&mut width, &mut height);
            }
        }
        size_portrait.map(|(width, height)| na::vector![width, height])
    }
}

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
#[serde(rename = "measure_unit")]
pub enum MeasureUnit {
    #[serde(rename = "px")]
    Px = 0,
    #[serde(rename = "mm")]
    Mm,
    #[serde(rename = "cm")]
    Cm,
}

impl Default for MeasureUnit {
    fn default() -> Self {
        Self::Px
    }
}

impl TryFrom<u32> for MeasureUnit {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("MeasureUnit try_from::<u32>() for value {} failed", value)
        })
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
            MeasureUnit::Cm => (value_in_px / desired_dpi) * Self::AMOUNT_MM_IN_INCH / 10.0,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "orientation")]
pub enum Orientation {
    #[serde(rename = "portrait")]
    Portrait = 0,
    #[serde(rename = "landscape")]
    Landscape,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "format")]
pub struct Format {
    #[serde(rename = "width", with = "rnote_compose::serialize::f64_dp3")]
    width: f64,
    #[serde(rename = "height", with = "rnote_compose::serialize::f64_dp3")]
    height: f64,
    #[serde(rename = "dpi", with = "rnote_compose::serialize::f64_dp3")]
    dpi: f64,
    #[serde(rename = "orientation")]
    orientation: Orientation,
    #[serde(rename = "border_color")]
    pub border_color: Color,
    #[serde(rename = "show_borders")]
    pub show_borders: bool,
    #[serde(rename = "show_origin_indicator")]
    pub show_origin_indicator: bool,
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
            show_origin_indicator: true,
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

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(Self::WIDTH_MIN, Self::WIDTH_MAX);
        self.orientation = self.determine_orientation();
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = height.clamp(Self::HEIGHT_MIN, Self::HEIGHT_MAX);
        self.orientation = self.determine_orientation();
    }

    pub fn dpi(&self) -> f64 {
        self.dpi
    }

    pub fn set_dpi(&mut self, dpi: f64) {
        self.dpi = dpi.clamp(Self::DPI_MIN, Self::DPI_MAX);
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn size(&self) -> na::Vector2<f64> {
        na::vector![self.width, self.height]
    }

    fn determine_orientation(&self) -> Orientation {
        if self.width <= self.height {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }
}
