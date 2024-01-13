// Imports
use crate::ext::{AabbExt, Vector2Ext};
use crate::shapes::Rectangle;
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use crate::Transform;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "line")]
/// A line.
pub struct Line {
    #[serde(rename = "start", with = "crate::serialize::na_vector2_f64_dp3")]
    /// Start coordinate.
    pub start: na::Vector2<f64>,
    #[serde(rename = "end", with = "crate::serialize::na_vector2_f64_dp3")]
    /// End coordinate.
    pub end: na::Vector2<f64>,
}

impl Transformable for Line {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.start += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = isometry.transform_point(&self.start.into()).coords;
        self.end = isometry.transform_point(&self.end.into()).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.end = self.end.component_mul(&scale);
    }
}

impl Shapeable for Line {
    fn bounds(&self) -> Aabb {
        AabbExt::new_positive(self.start.into(), self.end.into())
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let n_splits = super::hitbox_elems_for_shape_len((self.end - self.start).magnitude());

        self.split(n_splits)
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        kurbo::Line::new(self.start.to_kurbo_point(), self.end.to_kurbo_point()).to_path(0.25)
    }
}

impl Line {
    /// A new line.
    pub fn new(start: na::Vector2<f64>, end: na::Vector2<f64>) -> Self {
        Self { start, end }
    }

    /// Create a rectangle rotated in the direction of the line, with the given width.
    pub fn line_w_width_to_rect(&self, width: f64) -> Rectangle {
        let vec = self.end - self.start;
        let magn = vec.magnitude();
        let angle = na::Rotation2::rotation_between(&na::Vector2::x(), &vec).angle();

        Rectangle {
            cuboid: p2d::shape::Cuboid::new(na::vector![magn * 0.5, width * 0.5]),
            transform: Transform::new_w_isometry(na::Isometry2::new(self.start + vec * 0.5, angle)),
        }
    }

    /// Split itself given the number of splits.
    pub fn split(&self, n_splits: u32) -> Vec<Self> {
        (0..n_splits)
            .map(|i| {
                let sub_start = self
                    .start
                    .lerp(&self.end, f64::from(i) / f64::from(n_splits));
                let sub_end = self
                    .start
                    .lerp(&self.end, f64::from(i + 1) / f64::from(n_splits));

                Line {
                    start: sub_start,
                    end: sub_end,
                }
            })
            .collect::<Vec<Self>>()
    }
}
