// Imports
use super::Line;
use crate::Transform;
use crate::ext::{AabbExt, Vector2Ext};
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "rectangle")]
/// A rectangle.
pub struct Rectangle {
    #[serde(rename = "cuboid", with = "crate::serialize::p2d_cuboid_dp3")]
    /// The cuboid, specifies the extents.
    pub cuboid: p2d::shape::Cuboid,
    #[serde(rename = "transform")]
    /// The transform of the center of the cuboid.
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

impl Shapeable for Rectangle {
    fn bounds(&self) -> Aabb {
        let center = self.transform.affine * na::point![0.0, 0.0];
        // using a vector to ignore the translation
        let half_extents = na::Vector2::from_homogeneous(
            self.transform.affine.into_inner().abs()
                * self.cuboid.half_extents.abs().to_homogeneous(),
        )
        .unwrap()
        .abs();

        Aabb::from_half_extents(center, half_extents)
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

impl Transformable for Rectangle {
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

impl Rectangle {
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

    /// The outlines of the rect.
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

    /// Mirrors rectangle around line 'x = centerline_x'
    pub fn mirror_x(&mut self, centerline_x: f64) {
        self.transform.append_mirror_x_mut(centerline_x);
    }

    /// Mirrors rectangle around line 'y = centerline_y'
    pub fn mirror_y(&mut self, centerline_y: f64) {
        self.transform.append_mirror_y_mut(centerline_y);
    }
}
