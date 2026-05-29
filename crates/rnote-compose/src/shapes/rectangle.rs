// Imports
use super::Line;
use crate::Transform;
use crate::ext::{AabbExt, Vector2Ext};
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use p2d::bounding_volume::Aabb;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
use p2d::shape::Cuboid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "rectangle")]
/// A rectangle.
pub struct Rectangle {
    #[serde(rename = "cuboid", with = "crate::serialize::p2d_cuboid_dp3")]
    /// The cuboid, specifies the extents.
    pub cuboid: Cuboid,
    #[serde(rename = "transform")]
    /// The transform of the center of the cuboid.
    pub transform: Transform,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            cuboid: Cuboid::new(Vector2::ZERO),
            transform: Transform::IDENTITY,
        }
    }
}

impl Shapeable for Rectangle {
    fn bounds(&self) -> Aabb {
        self.transform.transform_aabb(self.cuboid.local_aabb())
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.outline_lines()
            .into_iter()
            .flat_map(|line| line.hitboxes())
            .collect()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        let tl = self.transform.affine.transform_point2(Vector2::new(
            -self.cuboid.half_extents[0],
            -self.cuboid.half_extents[1],
        ));
        let tr = self.transform.affine.transform_point2(Vector2::new(
            self.cuboid.half_extents[0],
            -self.cuboid.half_extents[1],
        ));
        let bl = self.transform.affine.transform_point2(Vector2::new(
            -self.cuboid.half_extents[0],
            self.cuboid.half_extents[1],
        ));
        let br = self.transform.affine.transform_point2(Vector2::new(
            self.cuboid.half_extents[0],
            self.cuboid.half_extents[1],
        ));

        kurbo::BezPath::from_vec(vec![
            kurbo::PathEl::MoveTo(tl.to_kurbo_point()),
            kurbo::PathEl::LineTo(tr.to_kurbo_point()),
            kurbo::PathEl::LineTo(br.to_kurbo_point()),
            kurbo::PathEl::LineTo(bl.to_kurbo_point()),
            kurbo::PathEl::ClosePath,
        ])
    }
}

impl Transformable for Rectangle {
    fn translate(&mut self, offset: Vector2) {
        self.transform.append_translation_mut(offset);
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        self.transform.append_rotation_wrt_center_mut(angle, center)
    }

    fn scale(&mut self, scale: Vector2) {
        self.transform.append_scale_mut(scale);
    }
}

impl Rectangle {
    /// Construct from center and half extents
    pub fn from_half_extents(center: Vector2, half_extents: Vector2) -> Self {
        let cuboid = Cuboid::new(half_extents);
        let transform = Transform::new_w_pose(DPose2::from_translation(center));

        Self { cuboid, transform }
    }

    /// Construct from corners across from each other.
    pub fn from_corners(first: Vector2, second: Vector2) -> Self {
        let half_extents = (second - first).abs() * 0.5;
        let center = first + (second - first) * 0.5;
        let cuboid = Cuboid::new(half_extents);
        let transform = Transform::new_w_pose(DPose2::from_translation(center));
        Self { cuboid, transform }
    }

    /// Construct from bounds.
    pub fn from_p2d_aabb(mut bounds: Aabb) -> Self {
        bounds.ensure_positive();
        let cuboid = Cuboid::new(bounds.half_extents());
        let transform = Transform::new_w_pose(DPose2::from_translation(bounds.center()));

        Self { cuboid, transform }
    }

    /// The outlines of the rect.
    pub fn outline_lines(&self) -> [Line; 4] {
        let upper_left = self.transform.transform_point(Vector2::new(
            -self.cuboid.half_extents[0],
            -self.cuboid.half_extents[1],
        ));
        let upper_right = self.transform.transform_point(Vector2::new(
            self.cuboid.half_extents[0],
            -self.cuboid.half_extents[1],
        ));
        let lower_left = self.transform.transform_point(Vector2::new(
            -self.cuboid.half_extents[0],
            self.cuboid.half_extents[1],
        ));
        let lower_right = self.transform.transform_point(Vector2::new(
            self.cuboid.half_extents[0],
            self.cuboid.half_extents[1],
        ));

        [
            Line {
                start: upper_left,
                end: lower_left,
            },
            Line {
                start: lower_left,
                end: lower_right,
            },
            Line {
                start: lower_right,
                end: upper_right,
            },
            Line {
                start: upper_right,
                end: upper_left,
            },
        ]
    }
}
