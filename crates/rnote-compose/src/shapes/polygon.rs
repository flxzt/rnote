// Imports
use super::{Line, Shapeable};
use crate::Transformable;
use crate::ext::{AabbExt, DPose2Ext, Vector2Ext};
use p2d::bounding_volume::Aabb;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "polygon")]
/// A Polygon.
pub struct Polygon {
    /// The polygon start
    #[serde(rename = "start")]
    pub start: Vector2,
    /// The polygon path
    #[serde(rename = "path")]
    pub path: Vec<Vector2>,
}

impl Transformable for Polygon {
    fn translate(&mut self, offset: Vector2) {
        self.start += offset;
        for p in &mut self.path {
            *p += offset;
        }
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        let pose = DPose2::IDENTITY.append_rotation_wrt_center(angle, center);
        self.start = pose.transform_point(self.start);
        for p in &mut self.path {
            *p = pose.transform_point(*p);
        }
    }

    fn scale(&mut self, scale: Vector2) {
        self.start *= scale;
        for p in &mut self.path {
            *p *= scale;
        }
    }
}

impl Shapeable for Polygon {
    fn bounds(&self) -> Aabb {
        let mut bounds = Aabb::new(self.start, self.start);
        for p in &self.path {
            bounds.take_point(*p);
        }
        bounds
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let mut hitboxes = Vec::with_capacity(self.path.len() + 1);
        hitboxes.push(Aabb::new(self.start, self.start));

        let mut prev = self.start;
        for p in &self.path {
            let n_splits = super::hitbox_elems_for_shape_len((p - prev).length());
            let line = Line::new(prev, *p);
            hitboxes.extend(line.split(n_splits).into_iter().map(|line| line.bounds()));
            prev = *p;
        }
        hitboxes.push(Aabb::new_positive(prev, self.start));

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
    pub fn new(start: Vector2) -> Self {
        Self {
            start,
            path: Vec::new(),
        }
    }
}

impl Extend<Vector2> for Polygon {
    fn extend<T: IntoIterator<Item = Vector2>>(&mut self, iter: T) {
        self.path.extend(iter);
    }
}
