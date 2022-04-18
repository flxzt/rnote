use gtk4::{gdk, glib, graphene, gsk};
use p2d::bounding_volume::AABB;
use rnote_compose::Transform;

pub trait GdkRGBAHelpers
where
    Self: Sized,
{
    fn from_compose_color(color: rnote_compose::Color) -> Self;
    fn into_compose_color(self) -> rnote_compose::Color;
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
}

pub trait GrapheneRectHelpers
where
    Self: Sized,
{
    fn from_p2d_aabb(aabb: AABB) -> Self;
}

impl GrapheneRectHelpers for graphene::Rect {
    fn from_p2d_aabb(aabb: AABB) -> Self {
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
        base64::decode(base64.as_bytes()).map_err(|e| serde::de::Error::custom(e))
    }
}
