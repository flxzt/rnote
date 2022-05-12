use crate::helpers::{AABBHelpers, KurboHelpers};
use crate::shapes::{CubicBezier, Line, QuadraticBezier, ShapeBehaviour};
use crate::transform::TransformBehaviour;

use kurbo::Shape;
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

use super::Element;

/// A single segment (usually of a path), containing elements to be able to being drawn with variable width
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "segment")]
pub enum Segment {
    #[serde(rename = "dot")]
    /// A dot segment.
    Dot {
        #[serde(rename = "element")]
        /// The element of the dot
        element: Element,
    },
    #[serde(rename = "line")]
    /// A line segment
    Line {
        #[serde(rename = "start")]
        /// The line start
        start: Element,
        #[serde(rename = "end")]
        /// The line end
        end: Element,
    },
    #[serde(rename = "quadbez")]
    /// A quadratic bezier segment
    QuadBez {
        #[serde(rename = "start")]
        /// The quadratic curve start
        start: Element,
        #[serde(rename = "cp")]
        /// The quadratic curve control point
        cp: na::Vector2<f64>,
        #[serde(rename = "end")]
        /// The quadratic curve end
        end: Element,
    },
    #[serde(rename = "cubbez")]
    /// A cubic bezier segment.
    CubBez {
        #[serde(rename = "start")]
        /// The cubic curve start
        start: Element,
        #[serde(rename = "cp1")]
        /// The cubic curve first control point
        cp1: na::Vector2<f64>,
        #[serde(rename = "cp2")]
        /// The cubic curve second control point
        cp2: na::Vector2<f64>,
        #[serde(rename = "end")]
        /// The cubic curve end
        end: Element,
    },
}

impl ShapeBehaviour for Segment {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        match self {
            Self::Dot { element: pos } => {
                AABB::new_positive(na::Point2::from(pos.pos), na::Point2::from(pos.pos))
                    .loosened(0.5)
            }
            Self::Line { start, end } => {
                AABB::new_positive(na::Point2::from(start.pos), na::Point2::from(end.pos))
            }
            Self::QuadBez { start, cp, end } => {
                let quadbez = QuadraticBezier {
                    start: start.pos,
                    cp: *cp,
                    end: end.pos,
                };

                quadbez.to_kurbo().bounding_box().bounds_as_p2d_aabb()
            }
            Self::CubBez {
                start,
                cp1,
                cp2,
                end,
            } => {
                let cubbez = CubicBezier {
                    start: start.pos,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: end.pos,
                };

                cubbez.to_kurbo().bounding_box().bounds_as_p2d_aabb()
            }
        }
    }

    fn hitboxes(&self) -> Vec<AABB> {
        match self {
            Segment::Dot { element } => vec![AABB::from_half_extents(
                na::Point2::from(element.pos),
                na::Vector2::repeat(0.5),
            )],
            Segment::Line { start, end } => {
                let n_splits = hitbox_elems_for_segment_len((end.pos - start.pos).magnitude());

                let line = Line {
                    start: start.pos,
                    end: end.pos,
                };

                line.split(n_splits)
                    .into_iter()
                    .map(|line| line.bounds())
                    .collect()
            }
            Segment::QuadBez { start, cp, end } => {
                let quadbez = QuadraticBezier {
                    start: start.pos,
                    cp: *cp,
                    end: end.pos,
                };

                // TODO: basing this off of the actual curve len
                let n_splits = hitbox_elems_for_segment_len(quadbez.to_kurbo().perimeter(0.1));

                quadbez
                    .approx_with_lines(n_splits)
                    .into_iter()
                    .map(|line| line.bounds())
                    .collect()
            }
            Segment::CubBez {
                start,
                cp1,
                cp2,
                end,
            } => {
                let cubbez = CubicBezier {
                    start: start.pos,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: end.pos,
                };

                // TODO: basing this off of the actual curve len
                let n_splits = hitbox_elems_for_segment_len(cubbez.to_kurbo().perimeter(0.1));

                cubbez
                    .approx_with_lines(n_splits)
                    .into_iter()
                    .map(|line| line.bounds())
                    .collect()
            }
        }
    }
}

impl TransformBehaviour for Segment {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        match self {
            Self::Dot { element } => {
                element.pos += offset;
            }
            Self::Line { start, end } => {
                start.pos += offset;
                end.pos += offset;
            }
            Self::QuadBez { start, cp, end } => {
                start.pos += offset;
                *cp += offset;
                end.pos += offset;
            }
            Self::CubBez {
                start,
                cp1,
                cp2,
                end,
            } => {
                start.pos += offset;
                *cp1 += offset;
                *cp2 += offset;
                end.pos += offset;
            }
        }
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        match self {
            Self::Dot { element } => {
                element.pos = (isometry * na::Point2::from(element.pos)).coords;
            }
            Self::Line { start, end } => {
                start.pos = (isometry * na::Point2::from(start.pos)).coords;
                end.pos = (isometry * na::Point2::from(end.pos)).coords;
            }
            Self::QuadBez { start, cp, end } => {
                start.pos = (isometry * na::Point2::from(start.pos)).coords;
                *cp = (isometry * na::Point2::from(*cp)).coords;
                end.pos = (isometry * na::Point2::from(end.pos)).coords;
            }
            Self::CubBez {
                start,
                cp1,
                cp2,
                end,
            } => {
                start.pos = (isometry * na::Point2::from(start.pos)).coords;
                *cp1 = (isometry * na::Point2::from(*cp1)).coords;
                *cp2 = (isometry * na::Point2::from(*cp2)).coords;
                end.pos = (isometry * na::Point2::from(end.pos)).coords;
            }
        }
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        match self {
            Self::Dot { element } => {
                element.pos = element.pos.component_mul(&scale);
            }
            Self::Line { start, end } => {
                start.pos = start.pos.component_mul(&scale);
                end.pos = end.pos.component_mul(&scale);
            }
            Self::QuadBez { start, cp, end } => {
                start.pos = start.pos.component_mul(&scale);
                *cp = cp.component_mul(&scale);
                end.pos = end.pos.component_mul(&scale);
            }
            Self::CubBez {
                start,
                cp1,
                cp2,
                end,
            } => {
                start.pos = start.pos.component_mul(&scale);
                *cp1 = cp1.component_mul(&scale);
                *cp2 = cp2.component_mul(&scale);
                end.pos = end.pos.component_mul(&scale);
            }
        }
    }
}

impl Segment {
    /// All segment choices have a start
    pub fn start(&self) -> Element {
        match self {
            Segment::Dot { element } => *element,
            Segment::Line { start, .. } => *start,
            Segment::QuadBez { start, .. } => *start,
            Segment::CubBez { start, .. } => *start,
        }
    }

    /// All segment choices have an end
    pub fn end(&self) -> Element {
        match self {
            Segment::Dot { element } => *element,
            Segment::Line { end, .. } => *end,
            Segment::QuadBez { end, .. } => *end,
            Segment::CubBez { end, .. } => *end,
        }
    }
}

/// Calculates the number hitbox elems for the given length capped with a maximum no of hitbox elemens
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
