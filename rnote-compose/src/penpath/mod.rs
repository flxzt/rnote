mod element;
mod segment;

// Re exports
pub use element::Element;
use kurbo::Shape;
pub use segment::Segment;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use serde::{Deserialize, Serialize};

use crate::helpers::KurboHelpers;
use crate::shapes::{CubicBezier, Line, QuadraticBezier, ShapeBehaviour};
use crate::transform::TransformBehaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "pen_path")]
/// a pen path, consisting of segments of pen input elements
pub struct PenPath {
    /// The path start
    #[serde(rename = "start")]
    pub start: Element,
    /// The segments
    #[serde(rename = "segments")]
    pub segments: Vec<Segment>,
}

impl ShapeBehaviour for PenPath {
    fn bounds(&self) -> Aabb {
        let mut bounds = Aabb::from_points(&[na::Point2::from(self.start.pos)]);

        let mut prev = self.start;
        for seg in self.segments.iter() {
            match seg {
                Segment::LineTo { end } => {
                    bounds.take_point(na::Point2::from(end.pos));

                    prev = *end;
                }
                Segment::QuadBezTo { cp, end } => {
                    let quadbez = QuadraticBezier {
                        start: prev.pos,
                        cp: *cp,
                        end: end.pos,
                    };

                    bounds.merge(&quadbez.to_kurbo().bounding_box().bounds_as_p2d_aabb());
                    prev = *end;
                }
                Segment::CubBezTo { cp1, cp2, end } => {
                    let cubbez = CubicBezier {
                        start: prev.pos,
                        cp1: *cp1,
                        cp2: *cp2,
                        end: end.pos,
                    };

                    bounds.merge(&cubbez.to_kurbo().bounding_box().bounds_as_p2d_aabb());
                    prev = *end;
                }
            }
        }

        bounds
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.hitboxes_priv()
            .into_iter()
            .flat_map(|(_, hb)| hb)
            .collect()
    }
}

impl TransformBehaviour for PenPath {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.start.translate(offset);
        self.segments.iter_mut().for_each(|segment| {
            segment.translate(offset);
        });
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.start.rotate(angle, center);
        self.segments.iter_mut().for_each(|segment| {
            segment.rotate(angle, center);
        });
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.start.scale(scale);
        self.segments.iter_mut().for_each(|segment| {
            segment.scale(scale);
        });
    }
}

impl PenPath {
    /// A new pen path with a first dot segment
    pub fn new(start: Element) -> Self {
        Self {
            start,
            segments: Vec::default(),
        }
    }

    /// A new pen path with segments
    pub fn new_w_segments(start: Element, segments: impl IntoIterator<Item = Segment>) -> Self {
        Self {
            start,
            segments: segments.into_iter().collect(),
        }
    }

    /// extracts the elements from the path. the path shape will be lost, as only the actual input elements are returned.
    pub fn into_elements(self) -> Vec<Element> {
        let mut elements = vec![self.start];

        elements.extend(self.segments.into_iter().map(|seg| match seg {
            Segment::LineTo { end } => end,
            Segment::QuadBezTo { end, .. } => end,
            Segment::CubBezTo { end, .. } => end,
        }));

        elements
    }

    /// Try to create a pen path from the elements. the first element will be the start
    pub fn try_from_elements(elements_iter: impl IntoIterator<Item = Element>) -> Option<Self> {
        let mut elements_iter = elements_iter.into_iter();

        let start = elements_iter.next()?;
        let segments = elements_iter
            .map(|el| Segment::LineTo { end: el })
            .collect::<Vec<Segment>>();

        Some(Self { start, segments })
    }

    /// Extends the pen path with the segments of the other.
    pub fn extend_w_other(&mut self, other: Self) {
        self.segments.extend(other.segments);
    }

    /// Checks whether a bounds collides with the path. If it does, it returns the index of the colliding segment
    pub fn hittest(&self, hit: &Aabb, loosened: f64) -> Option<usize> {
        for (i, seg_hitboxes) in self.hitboxes_priv() {
            if seg_hitboxes
                .into_iter()
                .any(|hitbox| hitbox.loosened(loosened).intersects(hit))
            {
                return Some(i);
            }
        }

        None
    }

    fn hitboxes_priv(&self) -> Vec<(usize, Vec<Aabb>)> {
        let mut hitboxes = Vec::with_capacity(self.segments.len());

        let mut prev = self.start;
        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                Segment::LineTo { end } => {
                    let n_splits = hitbox_elems_for_segment_len((end.pos - prev.pos).magnitude());
                    let line = Line {
                        start: prev.pos,
                        end: end.pos,
                    };

                    hitboxes.push((
                        i,
                        line.split(n_splits)
                            .into_iter()
                            .map(|line| line.bounds())
                            .collect(),
                    ));
                    prev = *end;
                }
                Segment::QuadBezTo { cp, end } => {
                    let quadbez = QuadraticBezier {
                        start: prev.pos,
                        cp: *cp,
                        end: end.pos,
                    };

                    // TODO: basing this off of the actual curve len
                    let n_splits = hitbox_elems_for_segment_len(quadbez.to_kurbo().perimeter(0.1));

                    hitboxes.push((
                        i,
                        quadbez
                            .approx_with_lines(n_splits)
                            .into_iter()
                            .map(|line| line.bounds())
                            .collect(),
                    ));
                    prev = *end;
                }
                Segment::CubBezTo { cp1, cp2, end } => {
                    let cubbez = CubicBezier {
                        start: prev.pos,
                        cp1: *cp1,
                        cp2: *cp2,
                        end: end.pos,
                    };

                    // TODO: basing this off of the actual curve len
                    let n_splits = hitbox_elems_for_segment_len(cubbez.to_kurbo().perimeter(0.1));

                    hitboxes.push((
                        i,
                        cubbez
                            .approx_with_lines(n_splits)
                            .into_iter()
                            .map(|line| line.bounds())
                            .collect(),
                    ));
                    prev = *end;
                }
            }
        }

        hitboxes
    }
}

impl Extend<Segment> for PenPath {
    fn extend<T: IntoIterator<Item = Segment>>(&mut self, iter: T) {
        self.segments.extend(iter);
    }
}

/// Calculates the number hitbox elems for the given length capped with a maximum no of hitbox elements
fn hitbox_elems_for_segment_len(len: f64) -> i32 {
    // Maximum hitbox diagonal ( below the threshold )
    const MAX_HITBOX_DIAGONAL: f64 = 15.0;
    const MAX_ELEMS: i32 = 6;

    if len < MAX_HITBOX_DIAGONAL * f64::from(MAX_ELEMS) {
        ((len / MAX_HITBOX_DIAGONAL).ceil() as i32).max(1)
    } else {
        // capping the no of elements for bigger len's, avoiding huge amounts of hitboxes for large strokes that are drawn when zoomed out
        MAX_ELEMS
    }
}
