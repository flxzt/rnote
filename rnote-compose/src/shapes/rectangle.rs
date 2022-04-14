use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use crate::helpers::Vector2Helpers;
use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;
use crate::Transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "rectangle")]
/// A rectangle
pub struct Rectangle {
    #[serde(rename = "cuboid")]
    /// The cuboid, consisting of half extents.
    pub cuboid: p2d::shape::Cuboid,
    #[serde(rename = "transform")]
    /// The transform to place the rect in a coordinate space
    pub transform: Transform,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            cuboid: p2d::shape::Cuboid::new(na::Vector2::zeros()),
            transform: Transform::default(),
        }
    }
}

impl ShapeBehaviour for Rectangle {
    fn bounds(&self) -> AABB {
        let center = self.transform.affine * na::point![0.0, 0.0];
        // using a vector to ignore the translation
        let half_extents = na::Vector2::from_homogeneous(
            self.transform.affine.into_inner().abs() * self.cuboid.half_extents.to_homogeneous(),
        )
        .unwrap()
        .abs();

        AABB::from_half_extents(center, half_extents)
    }
}

impl TransformBehaviour for Rectangle {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.transform.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.transform.append_rotation_wrt_point_mut(angle, center)
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.transform.append_scale_mut(scale);
    }
}

impl Rectangle {
    /// to kurbo
    pub fn to_kurbo(&self) -> kurbo::BezPath {
        let tl = self.transform.affine
            * na::point![-self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let tr = self.transform.affine
            * na::point![self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let bl = self.transform.affine
            * na::point![-self.cuboid.half_extents[0], self.cuboid.half_extents[1]];
        let br = self.transform.affine
            * na::point![self.cuboid.half_extents[0], self.cuboid.half_extents[1]];

        kurbo::BezPath::from_vec(vec![
            kurbo::PathEl::MoveTo(tl.coords.to_kurbo_point()),
            kurbo::PathEl::LineTo(tr.coords.to_kurbo_point()),
            kurbo::PathEl::LineTo(br.coords.to_kurbo_point()),
            kurbo::PathEl::LineTo(bl.coords.to_kurbo_point()),
            kurbo::PathEl::ClosePath,
        ])
    }
}
