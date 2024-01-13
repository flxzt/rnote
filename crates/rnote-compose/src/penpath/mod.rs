// Modules
mod element;
mod segment;

// Re-exports
pub use element::Element;
pub use segment::Segment;

// Imports
use crate::ext::{KurboShapeExt, Vector2Ext};
use crate::shapes::{CubicBezier, Line, QuadraticBezier, Shapeable};
use crate::transform::Transformable;
use kurbo::Shape;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use serde::{Deserialize, Serialize};

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

impl Shapeable for PenPath {
    fn bounds(&self) -> Aabb {
        let mut bounds = Aabb::from_points(&[self.start.pos.into()]);

        let mut prev = self.start;
        for seg in self.segments.iter() {
            match seg {
                Segment::LineTo { end } => {
                    bounds.take_point(end.pos.into());

                    prev = *end;
                }
                Segment::QuadBezTo { cp, end } => {
                    let quadbez = QuadraticBezier {
                        start: prev.pos,
                        cp: *cp,
                        end: end.pos,
                    };

                    bounds.merge(&quadbez.outline_path().bounding_box().bounds_to_p2d_aabb());
                    prev = *end;
                }
                Segment::CubBezTo { cp1, cp2, end } => {
                    let cubbez = CubicBezier {
                        start: prev.pos,
                        cp1: *cp1,
                        cp2: *cp2,
                        end: end.pos,
                    };

                    bounds.merge(&cubbez.outline_path().bounding_box().bounds_to_p2d_aabb());
                    prev = *end;
                }
            }
        }

        bounds
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        self.hitboxes_w_segs_indices()
            .into_iter()
            .flat_map(|(_, hb)| hb)
            .collect()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        kurbo::BezPath::from_iter(self.to_kurbo_el_iter())
    }
}

impl Transformable for PenPath {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.start.translate(offset);
        self.segments.iter_mut().for_each(|segment| {
            segment.translate(offset);
        });
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.start.rotate(angle, center);
        self.segments.iter_mut().for_each(|segment| {
            segment.rotate(angle, center);
        });
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start.scale(scale);
        self.segments.iter_mut().for_each(|segment| {
            segment.scale(scale);
        });
    }
}

impl PenPath {
    /// A new pen path
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

    /// Checks whether bounds collide with the path. If it does, it returns the indices of the colliding segments
    ///
    /// `loosened` loosens the segments hitboxes by the value
    pub fn hittest(&self, hit: &Aabb, loosened: f64) -> Vec<usize> {
        self.hitboxes_w_segs_indices()
            .into_iter()
            .filter_map(|(i, seg_hitboxes)| {
                seg_hitboxes
                    .into_iter()
                    .any(|hitbox| hitbox.loosened(loosened).intersects(hit))
                    .then_some(i)
            })
            .collect()
    }

    fn hitboxes_w_segs_indices(&self) -> Vec<(usize, Vec<Aabb>)> {
        let mut hitboxes = Vec::with_capacity(self.segments.len());

        let mut prev = self.start;
        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                Segment::LineTo { end } => {
                    let n_splits = no_subsegments_for_segment_len((end.pos - prev.pos).magnitude());
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

                    let n_splits =
                        no_subsegments_for_segment_len(quadbez.outline_path().perimeter(0.25));

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

                    let n_splits =
                        no_subsegments_for_segment_len(cubbez.outline_path().perimeter(0.25));

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

    /// Convert to [kurbo::BezPath], flattened to the given precision.
    pub fn to_kurbo_flattened(&self, tolerance: f64) -> kurbo::BezPath {
        let elements = self.to_kurbo_el_iter();

        let mut bezpath = kurbo::BezPath::new();
        kurbo::flatten(elements, tolerance, |el| bezpath.push(el));

        bezpath
    }

    fn to_kurbo_el_iter(&self) -> impl Iterator<Item = kurbo::PathEl> + '_ {
        std::iter::once(kurbo::PathEl::MoveTo(self.start.pos.to_kurbo_point())).chain(
            self.segments.iter().map(|s| match s {
                Segment::LineTo { end } => kurbo::PathEl::LineTo(end.pos.to_kurbo_point()),
                Segment::QuadBezTo { cp, end } => {
                    kurbo::PathEl::QuadTo(cp.to_kurbo_point(), end.pos.to_kurbo_point())
                }
                Segment::CubBezTo { cp1, cp2, end } => kurbo::PathEl::CurveTo(
                    cp1.to_kurbo_point(),
                    cp2.to_kurbo_point(),
                    end.pos.to_kurbo_point(),
                ),
            }),
        )
    }
}

impl Extend<Segment> for PenPath {
    fn extend<T: IntoIterator<Item = Segment>>(&mut self, iter: T) {
        self.segments.extend(iter);
    }
}

/// Calculates the number subsegment elements (for hitboxes/ flattening of bezier curve)
/// for the given segment length, capped with a maximum no of hitbox elements
pub(crate) fn no_subsegments_for_segment_len(len: f64) -> u32 {
    // Maximum hitbox diagonal ( below the threshold )
    const MAX_HITBOX_DIAGONAL: f64 = 15.0;
    const MAX_SUBSEGMENT_ELEMENTS: u32 = 5;

    if len < MAX_HITBOX_DIAGONAL * f64::from(MAX_SUBSEGMENT_ELEMENTS) {
        (len / MAX_HITBOX_DIAGONAL).round().max(0.) as u32
    } else {
        // capping the no of elements for bigger len's,
        // avoiding huge amounts of hitboxes for large strokes that are drawn when zoomed out
        MAX_SUBSEGMENT_ELEMENTS
    }
}
