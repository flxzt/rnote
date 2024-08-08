// Imports
use crate::fileformats::xoppformat;
use geo::line_string;
use p2d::bounding_volume::Aabb;
use rnote_compose::Color;
use std::ops::Range;

pub const fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn color_from_xopp(xopp_color: xoppformat::XoppColor) -> Color {
    Color {
        r: f64::from(xopp_color.red) / 255.0,
        g: f64::from(xopp_color.green) / 255.0,
        b: f64::from(xopp_color.blue) / 255.0,
        a: f64::from(xopp_color.alpha) / 255.0,
    }
}

pub fn xoppcolor_from_color(color: Color) -> xoppformat::XoppColor {
    xoppformat::XoppColor {
        red: (color.r * 255.0).floor() as u8,
        green: (color.g * 255.0).floor() as u8,
        blue: (color.b * 255.0).floor() as u8,
        alpha: (color.a * 255.0).floor() as u8,
    }
}

pub fn now_formatted_string() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}

pub fn doc_pages_files_names(file_stem_name: String, i: usize) -> String {
    file_stem_name + &format!(" - Page {i:02}")
}

pub fn convert_value_dpi(value: f64, current_dpi: f64, target_dpi: f64) -> f64 {
    (value / current_dpi) * target_dpi
}

pub fn convert_coord_dpi(
    coord: na::Vector2<f64>,
    current_dpi: f64,
    target_dpi: f64,
) -> na::Vector2<f64> {
    (coord / current_dpi) * target_dpi
}

#[cfg(feature = "ui")]
pub fn transform_to_gsk(transform: &rnote_compose::Transform) -> gtk4::gsk::Transform {
    gtk4::gsk::Transform::new().matrix(&gtk4::graphene::Matrix::from_2d(
        transform.affine[(0, 0)],
        transform.affine[(1, 0)],
        transform.affine[(0, 1)],
        transform.affine[(1, 1)],
        transform.affine[(0, 2)],
        transform.affine[(1, 2)],
    ))
}

/// Convert an [Aabb] to [`geo::Polygon<f64>`]
pub fn p2d_aabb_to_geo_polygon(aabb: Aabb) -> geo::Polygon<f64> {
    let line_string = line_string![
        (x: aabb.mins[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.mins[1]),
    ];
    geo::Polygon::new(line_string, vec![])
}

pub fn positive_range<I>(first: I, second: I) -> Range<I>
where
    I: PartialOrd,
{
    if first < second {
        first..second
    } else {
        second..first
    }
}

/// (De)Serialize a [glib::Bytes] with base64 encoding
pub mod glib_bytes_base64 {
    use serde::{Deserializer, Serializer};

    /// Serialize a [`Vec<u8>`] as base64 encoded
    pub fn serialize<S: Serializer>(v: &glib::Bytes, s: S) -> Result<S::Ok, S::Error> {
        rnote_compose::serialize::sliceu8_base64::serialize(v, s)
    }

    /// Deserialize base64 encoded [glib::Bytes]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<glib::Bytes, D::Error> {
        rnote_compose::serialize::sliceu8_base64::deserialize(d).map(glib::Bytes::from_owned)
    }
}
