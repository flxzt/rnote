use serde::{Deserialize, Serialize};

use crate::{
    helpers::{AABBHelpers, Vector2Helpers},
    transform::TransformBehaviour,
    Transform,
};

use super::{Rectangle, ShapeBehaviour};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "Arrow")]
pub struct Arrow {
    /// The anker of the arrow
    #[serde(rename = "start")]
    pub start: na::Vector2<f64>,

    /// The tip of the arrow
    #[serde(rename = "tip")]
    pub tip: na::Vector2<f64>,
}

impl TransformBehaviour for Arrow {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.start += offset;
        self.tip += offset;
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = (isometry * na::Point2::from(self.start)).coords;
        self.tip = (isometry * na::Point2::from(self.tip)).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.tip = self.tip.component_mul(&scale);
    }
}

impl ShapeBehaviour for Arrow {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        AABBHelpers::new_positive(na::Point2::from(self.start), na::Point2::from(self.tip))
    }

    fn hitboxes(&self) -> Vec<p2d::bounding_volume::AABB> {
        let n_splits = super::hitbox_elems_for_shape_len((self.tip - self.start).norm());

        self.split(n_splits)
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }
}

impl Arrow {
    /// creates a rect in the direction of the line, with a constant given width
    pub fn line_w_width_to_rect(&self, width: f64) -> Rectangle {
        let vec = self.tip - self.start;
        let magn = vec.magnitude();
        let angle = na::Rotation2::rotation_between(&na::Vector2::x(), &vec).angle();

        Rectangle {
            cuboid: p2d::shape::Cuboid::new(na::vector![magn * 0.5, width * 0.5]),
            transform: Transform::new_w_isometry(na::Isometry2::new(self.start + vec * 0.5, angle)),
        }
    }

    pub fn split(&self, n_splits: i32) -> Vec<Self> {
        (0..n_splits)
            .map(|i| {
                let sub_start = self
                    .start
                    .lerp(&self.tip, f64::from(i) / f64::from(n_splits));
                let sub_end = self
                    .start
                    .lerp(&self.tip, f64::from(i + 1) / f64::from(n_splits));

                Arrow {
                    start: sub_start,
                    tip: sub_end,
                }
            })
            .collect::<Vec<Self>>()
    }

    /// to kurbo
    pub fn to_kurbo(&self) -> kurbo::Line {
        kurbo::Line::new(self.start.to_kurbo_point(), self.tip.to_kurbo_point())
    }
}
