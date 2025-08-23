// Imports
use super::{Line, Shapeable};
use crate::ext::{AabbExt, Vector2Ext};
use crate::point_utils;
use crate::transform::{MirrorOrientation, Transformable};
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "polygon")]
/// A Polygon.
pub struct Polygon {
    /// The polygon start
    #[serde(rename = "start")]
    pub start: na::Vector2<f64>,
    /// The polygon path
    #[serde(rename = "path")]
    pub path: Vec<na::Vector2<f64>>,
}

impl Transformable for Polygon {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.start += offset;
        for p in &mut self.path {
            *p += offset;
        }
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = isometry.transform_point(&self.start.into()).coords;
        for p in &mut self.path {
            *p = isometry.transform_point(&(*p).into()).coords;
        }
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        for p in &mut self.path {
            *p = p.component_mul(&scale);
        }
    }

    fn mirror(&mut self, centerline: f64, orientation: MirrorOrientation) {
        match orientation {
            MirrorOrientation::Horizontal => {
                point_utils::mirror_point_x(&mut self.start, centerline);

                for point in self.path.iter_mut() {
                    point_utils::mirror_point_x(point, centerline);
                }
            }
            MirrorOrientation::Vertical => {
                point_utils::mirror_point_y(&mut self.start, centerline);

                for point in self.path.iter_mut() {
                    point_utils::mirror_point_y(point, centerline);
                }
            }
        }
    }
}

impl Shapeable for Polygon {
    fn bounds(&self) -> Aabb {
        let mut bounds = Aabb::new(self.start.into(), self.start.into());
        for p in &self.path {
            bounds.take_point((*p).into());
        }
        bounds
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let mut hitboxes = Vec::with_capacity(self.path.len() + 1);
        hitboxes.push(Aabb::new(self.start.into(), self.start.into()));

        let mut prev = self.start;
        for p in &self.path {
            let n_splits = super::hitbox_elems_for_shape_len((p - prev).magnitude());
            let line = Line::new(prev, *p);

            hitboxes.extend(line.split(n_splits).into_iter().map(|line| line.bounds()));

            prev = *p;
        }
        hitboxes.push(Aabb::new_positive(prev.into(), self.start.into()));

        hitboxes
    }

    fn outline_path(&self) -> kurbo::BezPath {
        let iter = std::iter::once(kurbo::PathEl::MoveTo(self.start.to_kurbo_point())).chain(
            self.path
                .iter()
                .map(|p| kurbo::PathEl::LineTo(p.to_kurbo_point())),
        );
        let mut path = kurbo::BezPath::from_iter(iter);
        path.close_path();
        path
    }
}

impl Polygon {
    /// A new polygon
    pub fn new(start: na::Vector2<f64>) -> Self {
        Self {
            start,
            path: Vec::new(),
        }
    }
}

impl Extend<na::Vector2<f64>> for Polygon {
    fn extend<T: IntoIterator<Item = na::Vector2<f64>>>(&mut self, iter: T) {
        self.path.extend(iter);
    }
}
