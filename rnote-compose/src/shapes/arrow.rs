use serde::{Serialize, Deserialize};

use crate::{transform::TransformBehaviour, helpers::AABBHelpers};

use super::ShapeBehaviour;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "Arrow")]
pub struct Arrow {
    /// The anker of the arrow
    #[serde(rename = "start")]
    pub start: na::Vector2<f64>,

    /// The tip of the arrow
    #[serde(rename = "end")]
    pub end: na::Vector2<f64>,
}

impl TransformBehaviour for Arrow {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.start += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = (isometry * na::Point2::from(self.start)).coords;
        self.end = (isometry * na::Point2::from(self.end)).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.end = self.end.component_mul(&scale);
    }
}

impl ShapeBehaviour for Arrow {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        AABBHelpers::new_positive(na::Point2::from(self.start), na::Point2::from(self.end))
    }

    fn hitboxes(&self) -> Vec<p2d::bounding_volume::AABB> {
        let n_splits = super::hitbox_elems_for_shape_len((self.end - self.start).norm());

        self.split(n_splits)
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }
}

impl Arrow {
    pub fn split(&self, n_splits: i32) -> Vec<Self> {
        todo!()

    }
}
