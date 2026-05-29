// Imports
use crate::Transformable;
use crate::ext::{AabbExt, DPose2Ext, Vector2Ext};
use crate::shapes::Rectangle;
use crate::shapes::Shapeable;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "line")]
/// A line.
pub struct Line {
    #[serde(rename = "start", with = "crate::serialize::glam_vector2_dp3")]
    /// Start coordinate.
    pub start: Vector2,
    #[serde(rename = "end", with = "crate::serialize::glam_vector2_dp3")]
    /// End coordinate.
    pub end: Vector2,
}

impl Transformable for Line {
    fn translate(&mut self, offset: Vector2) {
        self.start += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        let pose = DPose2::IDENTITY.append_rotation_wrt_center(angle, center);
        self.start = pose.transform_point(self.start);
        self.end = pose.transform_point(self.end);
    }

    fn scale(&mut self, scale: Vector2) {
        self.start *= scale;
        self.end *= scale;
    }
}

impl Shapeable for Line {
    fn bounds(&self) -> Aabb {
        AabbExt::new_positive(self.start, self.end)
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let n_splits = super::hitbox_elems_for_shape_len((self.end - self.start).length());

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
    pub fn new(start: Vector2, end: Vector2) -> Self {
        Self { start, end }
    }

    /// Create a rectangle rotated in the direction of the line, with the given width.
    pub fn line_w_width_to_rect(&self, width: f64) -> Rectangle {
        let vec = self.end - self.start;
        let angle = Vector2::X.angle_to(vec);
        let magn = vec.length();

        Rectangle {
            cuboid: p2d::shape::Cuboid::new(Vector2::new(magn * 0.5, width * 0.5)),
            affine: DAffine2::from_angle_translation(angle, self.start + vec * 0.5),
        }
    }

    /// Split itself given the number of splits.
    pub fn split(&self, n_splits: i32) -> Vec<Self> {
        (0..n_splits)
            .map(|i| {
                let sub_start = self
                    .start
                    .lerp(self.end, f64::from(i) / f64::from(n_splits));
                let sub_end = self
                    .start
                    .lerp(self.end, f64::from(i + 1) / f64::from(n_splits));

                Line {
                    start: sub_start,
                    end: sub_end,
                }
            })
            .collect::<Vec<Self>>()
    }
}
