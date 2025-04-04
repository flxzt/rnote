// Imports
use super::Line;
use crate::ext::{AabbExt, KurboShapeExt, Vector2Ext};
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use crate::Transform;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "parabola")]
/// A parabola.
pub struct Parabola {
    #[serde(rename = "cuboid", with = "crate::serialize::p2d_cuboid_dp3")]
    /// The cuboid, specifies the extents.
    pub cuboid: p2d::shape::Cuboid,
    #[serde(rename = "transform")]
    /// The transform of the center of the cuboid.
    pub transform: Transform,
}

impl Default for Parabola {
    fn default() -> Self {
        Self {
            cuboid: p2d::shape::Cuboid::new(na::Vector2::zeros()),
            transform: Transform::default(),
        }
    }
}

impl Shapeable for Parabola {
    fn bounds(&self) -> Aabb {
        let tl = self.transform.affine
            * na::point![-self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let tr = self.transform.affine
            * na::point![self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let bm = self.transform.affine * na::point![0.0, 3.0 * self.cuboid.half_extents[1]];

        let bez = kurbo::QuadBez::new(
            na::Vector3::from(tl).xy().to_kurbo_point(),
            na::Vector3::from(bm).xy().to_kurbo_point(),
            na::Vector3::from(tr).xy().to_kurbo_point(),
        );

        bez.bounds_to_p2d_aabb()
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.outline_lines()
            .into_iter()
            .flat_map(|line| line.hitboxes())
            .collect()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        let tl = self.transform.affine
            * na::point![-self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let tr = self.transform.affine
            * na::point![self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let bm = self.transform.affine * na::point![0.0, 3.0 * self.cuboid.half_extents[1]];

        kurbo::QuadBez::new(
            na::Vector3::from(tl).xy().to_kurbo_point(),
            na::Vector3::from(bm).xy().to_kurbo_point(),
            na::Vector3::from(tr).xy().to_kurbo_point(),
        )
        .to_path(0.25)
    }
}

impl Transformable for Parabola {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.transform.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.transform.append_rotation_wrt_point_mut(angle, center)
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.transform.append_scale_mut(scale);
    }
}

impl Parabola {
    /// Construct from center and half extents
    pub fn from_half_extents(center: na::Vector2<f64>, half_extents: na::Vector2<f64>) -> Self {
        let cuboid = p2d::shape::Cuboid::new(half_extents);
        let transform = Transform::new_w_isometry(na::Isometry2::new(center, 0.0));

        Self { cuboid, transform }
    }

    /// Construct from corners across from each other.
    pub fn from_corners(first: na::Vector2<f64>, second: na::Vector2<f64>) -> Self {
        let half_extents = (second - first).abs() * 0.5;
        let center = first + (second - first) * 0.5;

        let cuboid = p2d::shape::Cuboid::new(half_extents);
        let transform = Transform::new_w_isometry(na::Isometry2::new(center, 0.0));

        Self { cuboid, transform }
    }

    /// Construct from bounds.
    pub fn from_p2d_aabb(mut bounds: Aabb) -> Self {
        bounds.ensure_positive();
        let cuboid = p2d::shape::Cuboid::new(bounds.half_extents());
        let transform = Transform::new_w_isometry(na::Isometry2::new(bounds.center().coords, 0.0));

        Self { cuboid, transform }
    }

    /// The outlines of the parabola.
    pub fn outline_lines(&self) -> [Line; 4] {
        let upper_left = self.transform.transform_point(na::point![
            -self.cuboid.half_extents[0],
            -self.cuboid.half_extents[1]
        ]);
        let upper_right = self.transform.transform_point(na::point![
            self.cuboid.half_extents[0],
            -self.cuboid.half_extents[1]
        ]);
        let lower_left = self.transform.transform_point(na::point![
            -self.cuboid.half_extents[0],
            self.cuboid.half_extents[1]
        ]);
        let lower_right = self.transform.transform_point(na::point![
            self.cuboid.half_extents[0],
            self.cuboid.half_extents[1]
        ]);

        [
            Line {
                start: upper_left.coords,
                end: lower_left.coords,
            },
            Line {
                start: lower_left.coords,
                end: lower_right.coords,
            },
            Line {
                start: lower_right.coords,
                end: upper_right.coords,
            },
            Line {
                start: upper_right.coords,
                end: upper_left.coords,
            },
        ]
    }
}
