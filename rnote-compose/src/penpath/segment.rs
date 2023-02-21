use serde::{Deserialize, Serialize};

use super::Element;
use crate::transform::TransformBehaviour;

/// A single segment (usually of a path)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "segment")]
pub enum Segment {
    #[serde(rename = "lineto", alias = "line")]
    /// A line to segment
    LineTo {
        #[serde(rename = "end")]
        /// The line end
        end: Element,
    },
    #[serde(rename = "quadbezto", alias = "quadbez")]
    /// A quadratic bezier to segment
    QuadBezTo {
        #[serde(rename = "cp", with = "crate::serialize::na_vector2_f64_dp3")]
        /// The quadratic curve control point
        cp: na::Vector2<f64>,
        #[serde(rename = "end")]
        /// The quadratic curve end
        end: Element,
    },
    #[serde(rename = "cubbezto", alias = "cubbez")]
    /// A cubic bezier to segment.
    CubBezTo {
        #[serde(rename = "cp1", with = "crate::serialize::na_vector2_f64_dp3")]
        /// The cubic curve first control point
        cp1: na::Vector2<f64>,
        #[serde(rename = "cp2", with = "crate::serialize::na_vector2_f64_dp3")]
        /// The cubic curve second control point
        cp2: na::Vector2<f64>,
        #[serde(rename = "end")]
        /// The cubic curve end
        end: Element,
    },
}

impl TransformBehaviour for Segment {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        match self {
            Self::LineTo { end } => {
                end.pos += offset;
            }
            Self::QuadBezTo { cp, end } => {
                *cp += offset;
                end.pos += offset;
            }
            Self::CubBezTo { cp1, cp2, end } => {
                *cp1 += offset;
                *cp2 += offset;
                end.pos += offset;
            }
        }
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        match self {
            Self::LineTo { end } => {
                end.pos = (isometry * na::Point2::from(end.pos)).coords;
            }
            Self::QuadBezTo { cp, end } => {
                *cp = (isometry * na::Point2::from(*cp)).coords;
                end.pos = (isometry * na::Point2::from(end.pos)).coords;
            }
            Self::CubBezTo { cp1, cp2, end } => {
                *cp1 = (isometry * na::Point2::from(*cp1)).coords;
                *cp2 = (isometry * na::Point2::from(*cp2)).coords;
                end.pos = (isometry * na::Point2::from(end.pos)).coords;
            }
        }
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        match self {
            Self::LineTo { end } => {
                end.pos = end.pos.component_mul(&scale);
            }
            Self::QuadBezTo { cp, end } => {
                *cp = cp.component_mul(&scale);
                end.pos = end.pos.component_mul(&scale);
            }
            Self::CubBezTo { cp1, cp2, end } => {
                *cp1 = cp1.component_mul(&scale);
                *cp2 = cp2.component_mul(&scale);
                end.pos = end.pos.component_mul(&scale);
            }
        }
    }
}

impl Segment {
    /// All segments have an end
    pub fn end(&self) -> Element {
        match self {
            Segment::LineTo { end, .. } => *end,
            Segment::QuadBezTo { end, .. } => *end,
            Segment::CubBezTo { end, .. } => *end,
        }
    }
}
