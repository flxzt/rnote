mod element;
mod segment;

// Re exports
pub use element::Element;
pub use segment::Segment;

use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename = "path")]
pub struct PenPath(VecDeque<Segment>);

impl Deref for PenPath {
    type Target = VecDeque<Segment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PenPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ShapeBehaviour for PenPath {
    fn bounds(&self) -> AABB {
        self.iter()
            .map(|segment| segment.bounds())
            .fold(AABB::new_invalid(), |prev, next| prev.merged(&next))
    }
}

impl TransformBehaviour for PenPath {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.iter_mut().for_each(|segment| {
            segment.translate(offset);
        });
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.iter_mut().for_each(|segment| {
            segment.rotate(angle, center);
        });
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.iter_mut().for_each(|segment| {
            segment.scale(scale);
        });
    }
}

impl PenPath {
    pub fn new_w_dot(element: Element) -> Self {
        Self::new_w_segment(Segment::Dot { element })
    }

    pub fn new_w_segment(segment: Segment) -> Self {
        let mut segment_vec = VecDeque::with_capacity(1);
        segment_vec.push_back(segment);

        Self(segment_vec)
    }

    pub fn into_elements(self) -> Vec<Element> {
        self.0
            .into_iter()
            .map(|segment| match segment {
                Segment::Dot { element: pos } => vec![pos],
                Segment::Line { start, end } => vec![start, end],
                Segment::QuadBez { start, cp: _, end } => vec![start, end],
                Segment::CubBez {
                    start,
                    cp1: _,
                    cp2: _,
                    end,
                } => vec![start, end],
            })
            .flatten()
            .collect()
    }
}

impl std::iter::FromIterator<Segment> for PenPath {
    fn from_iter<T: IntoIterator<Item = Segment>>(iter: T) -> Self {
        Self(VecDeque::from_iter(iter))
    }
}
