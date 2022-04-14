use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use crate::helpers::{AABBHelpers, Vector2Helpers};
use crate::shapes::Rectangle;
use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;
use crate::Transform;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "line")]
/// A line
pub struct Line {
    #[serde(rename = "start")]
    /// The line start
    pub start: na::Vector2<f64>,
    #[serde(rename = "end")]
    /// The line end
    pub end: na::Vector2<f64>,
}

impl TransformBehaviour for Line {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.start += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = (isometry * na::Point2::from(self.start)).coords;
        self.end = (isometry * na::Point2::from(self.end)).coords;
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.end = self.end.component_mul(&scale);
    }
}

impl ShapeBehaviour for Line {
    fn bounds(&self) -> AABB {
        AABBHelpers::new_positive(na::Point2::from(self.start), na::Point2::from(self.end))
    }
}

impl Line {
    /// creates a rect in the direction of the line, with a constant given width
    pub fn line_w_width_to_rect(&self, width: f64) -> Rectangle {
        let vec = self.end - self.start;
        let magn = vec.magnitude();
        let angle = na::Rotation2::rotation_between(&na::Vector2::x(), &vec).angle();

        Rectangle {
            cuboid: p2d::shape::Cuboid::new(na::vector![magn / 2.0, width / 2.0]),
            transform: Transform::new_w_isometry(na::Isometry2::new(self.start + vec / 2.0, angle)),
        }
    }

    /// to kurbo
    pub fn to_kurbo(&self) -> kurbo::Line {
        kurbo::Line::new(self.start.to_kurbo_point(), self.end.to_kurbo_point())
    }
}
