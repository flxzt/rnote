use gtk4::{glib, graphene, gsk, Snapshot};
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use crate::compose::color::Color;
use crate::compose::geometry::AABBHelpers;

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "PredefinedFormat")]
#[serde(rename = "predefined_format")]
pub enum PredefinedFormat {
    #[enum_value(name = "A6", nick = "a6")]
    #[serde(rename = "a6")]
    A6 = 0,
    #[enum_value(name = "A5", nick = "a5")]
    #[serde(rename = "a5")]
    A5,
    #[enum_value(name = "A4", nick = "a4")]
    #[serde(rename = "a4")]
    A4,
    #[enum_value(name = "A3", nick = "a3")]
    #[serde(rename = "a3")]
    A3,
    #[enum_value(name = "A2", nick = "a2")]
    #[serde(rename = "a2")]
    A2,
    #[enum_value(name = "US Letter", nick = "us-letter")]
    #[serde(rename = "us_letter")]
    UsLetter,
    #[enum_value(name = "US Legal", nick = "us-legal")]
    #[serde(rename = "us_legal")]
    UsLegal,
    #[enum_value(name = "Custom", nick = "custom")]
    #[serde(rename = "custom")]
    Custom,
}

impl Default for PredefinedFormat {
    fn default() -> Self {
        Self::A3
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(default, rename = "width")]
pub struct Format {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "height")]
    pub height: f64,
    #[serde(rename = "dpi")]
    pub dpi: f64,
    #[serde(rename = "orientation")]
    pub orientation: Orientation,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            height: Self::HEIGHT_DEFAULT,
            dpi: Self::DPI_DEFAULT,
            orientation: Orientation::default(),
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

    pub const FORMAT_BORDER_COLOR: Color = Color {
        r: 0.6,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub fn draw(&self, sheet_bounds: AABB, snapshot: &Snapshot, zoom: f64) {
        let border_radius = graphene::Size::new(0.0, 0.0);
        let border_width = 1_f32;

        snapshot.push_clip(
            &sheet_bounds
                .scale(na::Vector2::from_element(zoom))
                .to_graphene_rect(),
        );

        let pages_bounds =
            sheet_bounds.split_extended_origin_aligned(na::vector![self.width, self.height]);

        for page_bounds in pages_bounds {
            let rounded_rect = gsk::RoundedRect::new(
                page_bounds
                    .scale(na::Vector2::from_element(zoom))
                    .to_graphene_rect(),
                border_radius.clone(),
                border_radius.clone(),
                border_radius.clone(),
                border_radius.clone(),
            );
            snapshot.append_border(
                &rounded_rect,
                &[border_width, border_width, border_width, border_width],
                &[
                    Self::FORMAT_BORDER_COLOR.to_gdk(),
                    Self::FORMAT_BORDER_COLOR.to_gdk(),
                    Self::FORMAT_BORDER_COLOR.to_gdk(),
                    Self::FORMAT_BORDER_COLOR.to_gdk(),
                ],
            );
        }

        snapshot.pop();
    }
}
