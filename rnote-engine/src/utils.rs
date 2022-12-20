use std::ops::Range;

use geo::line_string;
use gtk4::{gdk, gio, glib, graphene, gsk, pango, prelude::*};
use p2d::bounding_volume::Aabb;
use rnote_compose::Color;
use rnote_compose::{penevents::KeyboardKey, Transform};
use rnote_fileformats::xoppformat;

pub trait GdkRGBAHelpers
where
    Self: Sized,
{
    fn from_compose_color(color: rnote_compose::Color) -> Self;
    fn into_compose_color(self) -> rnote_compose::Color;
    fn from_piet_color(color: piet::Color) -> Self;
    fn into_piet_color(self) -> piet::Color;
}

impl GdkRGBAHelpers for gdk::RGBA {
    fn from_compose_color(color: rnote_compose::Color) -> Self {
        gdk::RGBA::new(
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        )
    }
    fn into_compose_color(self) -> rnote_compose::Color {
        rnote_compose::Color {
            r: f64::from(self.red()),
            g: f64::from(self.green()),
            b: f64::from(self.blue()),
            a: f64::from(self.alpha()),
        }
    }

    fn from_piet_color(color: piet::Color) -> Self {
        let (r, g, b, a) = color.as_rgba();
        gdk::RGBA::new(r as f32, g as f32, b as f32, a as f32)
    }

    fn into_piet_color(self) -> piet::Color {
        piet::Color::rgba(
            f64::from(self.red()),
            f64::from(self.green()),
            f64::from(self.blue()),
            f64::from(self.alpha()),
        )
    }
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

pub trait GrapheneRectHelpers
where
    Self: Sized,
{
    fn from_p2d_aabb(aabb: Aabb) -> Self;
}

impl GrapheneRectHelpers for graphene::Rect {
    fn from_p2d_aabb(aabb: Aabb) -> Self {
        graphene::Rect::new(
            aabb.mins[0] as f32,
            aabb.mins[1] as f32,
            (aabb.extents()[0]) as f32,
            (aabb.extents()[1]) as f32,
        )
    }
}

pub fn now_formatted_string() -> String {
    match glib::DateTime::now_local() {
        Ok(datetime) => match datetime.format("%F_%H-%M-%S") {
            Ok(s) => s.to_string(),
            Err(_) => String::from("1970-01-01_12-00-00"),
        },
        Err(_) => String::from("1970-01-01_12-00-00"),
    }
}

pub fn default_file_title_for_export(
    output_file: Option<gio::File>,
    default_fallback: Option<&str>,
    suffix: Option<&str>,
) -> String {
    let mut title = output_file
        .and_then(|f| Some(f.basename()?.file_stem()?.to_string_lossy().to_string()))
        .unwrap_or_else(|| {
            default_fallback
                .map(|f| f.to_owned())
                .unwrap_or_else(now_formatted_string)
        });

    if let Some(suffix) = suffix {
        title += suffix;
    }

    title
}

pub fn doc_pages_files_names(file_stem_name: String, i: usize) -> String {
    file_stem_name + &format!(" - Page {:02}", i)
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

pub fn transform_to_gsk(transform: &Transform) -> gsk::Transform {
    gsk::Transform::new().matrix(&graphene::Matrix::from_2d(
        transform.affine[(0, 0)],
        transform.affine[(1, 0)],
        transform.affine[(0, 1)],
        transform.affine[(1, 1)],
        transform.affine[(0, 2)],
        transform.affine[(1, 2)],
    ))
}

pub fn pango_font_weight_to_raw(pango_font_weight: pango::Weight) -> u16 {
    match pango_font_weight {
        pango::Weight::Thin => 100,
        pango::Weight::Ultralight => 200,
        pango::Weight::Light => 300,
        pango::Weight::Semilight => 350,
        pango::Weight::Book => 380,
        pango::Weight::Normal => 400,
        pango::Weight::Medium => 500,
        pango::Weight::Semibold => 600,
        pango::Weight::Bold => 700,
        pango::Weight::Ultrabold => 800,
        pango::Weight::Heavy => 900,
        pango::Weight::Ultraheavy => 100,
        _ => 500,
    }
}

pub fn raw_font_weight_to_pango(raw_font_weight: u16) -> pango::Weight {
    match raw_font_weight {
        0..=149 => pango::Weight::Thin,
        150..=249 => pango::Weight::Ultralight,
        250..=324 => pango::Weight::Light,
        325..=364 => pango::Weight::Semilight,
        365..=389 => pango::Weight::Book,
        390..=449 => pango::Weight::Normal,
        450..=549 => pango::Weight::Medium,
        550..=649 => pango::Weight::Semibold,
        650..=749 => pango::Weight::Bold,
        750..=849 => pango::Weight::Ultrabold,
        850..=949 => pango::Weight::Heavy,
        950.. => pango::Weight::Ultraheavy,
    }
}

/// Converts a Aabb to a geo::Polygon
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

pub fn keyboard_key_from_gdk(gdk_key: gdk::Key) -> KeyboardKey {
    //log::debug!("gdk: pressed key: {:?}", gdk_key);

    if let Some(keychar) = gdk_key.to_unicode() {
        KeyboardKey::Unicode(keychar).filter_convert_unicode_control_chars()
    } else {
        match gdk_key {
            gdk::Key::BackSpace => KeyboardKey::BackSpace,
            gdk::Key::Tab => KeyboardKey::HorizontalTab,
            gdk::Key::Linefeed => KeyboardKey::Linefeed,
            gdk::Key::Return => KeyboardKey::CarriageReturn,
            gdk::Key::Escape => KeyboardKey::Escape,
            gdk::Key::Delete => KeyboardKey::Delete,
            gdk::Key::Down => KeyboardKey::NavDown,
            gdk::Key::Up => KeyboardKey::NavUp,
            gdk::Key::Left => KeyboardKey::NavLeft,
            gdk::Key::Right => KeyboardKey::NavRight,
            gdk::Key::Shift_L => KeyboardKey::ShiftLeft,
            gdk::Key::Shift_R => KeyboardKey::ShiftRight,
            gdk::Key::Control_L => KeyboardKey::CtrlLeft,
            gdk::Key::Control_R => KeyboardKey::CtrlRight,
            _ => KeyboardKey::Unsupported,
        }
    }
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

pub mod base64 {
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    /// Serialize a Vec<u8> as base64 encoded
    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::encode(v);
        String::serialize(&base64, s)
    }

    /// Deserialize base64 encoded Vec<u8>
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        base64::decode(base64.as_bytes()).map_err(serde::de::Error::custom)
    }
}
