use crate::helpers::AABBHelpers;
use crate::shapes::{cubbez, quadbez, ShapeBehaviour, Line};
use crate::transform::TransformBehaviour;

use cubbez::CubicBezier;
use p2d::bounding_volume::{BoundingVolume, AABB};
use quadbez::QuadraticBezier;
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
                let mut aabb =
                    AABB::new_positive(na::Point2::from(start.pos), na::Point2::from(end.pos));
                aabb.take_point(na::Point2::from(*cp));
                aabb
            }
            Self::CubBez {
                start,
                cp1,
                cp2,
                end,
            } => {
                let mut aabb =
                    AABB::new_positive(na::Point2::from(start.pos), na::Point2::from(end.pos));
                aabb.take_point(na::Point2::from(*cp1));
                aabb.take_point(na::Point2::from(*cp2));
                aabb
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
                let line = Line {start: start.pos, end: end.pos};
                line.hitboxes()
            }
            Segment::QuadBez { start, cp, end } => {
                let quad_bez = QuadraticBezier {
                    start: start.pos,
                    cp: *cp,
                    end: end.pos,
                };
                quad_bez.hitboxes()
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
                cubbez.hitboxes()
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
