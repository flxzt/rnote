// Imports
use geo::line_string;
use gtk4::{gdk, graphene, gsk};
use p2d::bounding_volume::Aabb;
use rnote_compose::Color;
use rnote_compose::Transform;
use rnote_fileformats::xoppformat;
use std::ops::Range;

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
