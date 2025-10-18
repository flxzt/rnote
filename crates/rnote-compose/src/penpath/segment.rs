// Imports
use super::Element;
use crate::{
    point_utils,
    transform::{MirrorOrientation, Transformable},
};
use serde::{Deserialize, Serialize};

/// A single segment, usually of a pen path.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "segment")]
pub enum Segment {
    #[serde(rename = "lineto", alias = "line")]
    /// A line-to segment.
    LineTo {
        #[serde(rename = "end")]
        /// The line end.
        end: Element,
    },
    #[serde(rename = "quadbezto", alias = "quadbez")]
    /// A quadratic-bezier-to segment.
    QuadBezTo {
        #[serde(rename = "cp", with = "crate::serialize::na_vector2_f64_dp3")]
        /// The quadratic curve control point.
        cp: na::Vector2<f64>,
        #[serde(rename = "end")]
        /// The quadratic curve end.
        end: Element,
    },
    #[serde(rename = "cubbezto", alias = "cubbez")]
    /// A cubic-bezier-to segment.
    CubBezTo {
        #[serde(rename = "cp1", with = "crate::serialize::na_vector2_f64_dp3")]
        /// The cubic curve first control point.
        cp1: na::Vector2<f64>,
        #[serde(rename = "cp2", with = "crate::serialize::na_vector2_f64_dp3")]
        /// The cubic curve second control point.
        cp2: na::Vector2<f64>,
        #[serde(rename = "end")]
        /// The cubic curve end.
        end: Element,
    },
}

impl Transformable for Segment {
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
                end.pos = isometry.transform_point(&end.pos.into()).coords;
            }
            Self::QuadBezTo { cp, end } => {
                *cp = isometry.transform_point(&(*cp).into()).coords;
                end.pos = isometry.transform_point(&end.pos.into()).coords;
            }
            Self::CubBezTo { cp1, cp2, end } => {
                *cp1 = isometry.transform_point(&(*cp1).into()).coords;
                *cp2 = isometry.transform_point(&(*cp2).into()).coords;
                end.pos = isometry.transform_point(&end.pos.into()).coords;
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

    fn mirror(&mut self, centerline: f64, orientation: MirrorOrientation) {
        match self {
            Segment::LineTo { end } => {
                end.mirror(centerline, orientation);
            }
            Segment::QuadBezTo { cp, end } => {
                point_utils::mirror_point(cp, centerline, orientation);
                end.mirror(centerline, orientation);
            }
            Segment::CubBezTo { cp1, cp2, end } => {
                point_utils::mirror_point(cp1, centerline, orientation);
                point_utils::mirror_point(cp2, centerline, orientation);
                end.mirror(centerline, orientation);
            }
        }
    }
}

impl Segment {
    /// The end element of a segment.
    ///
    /// All segment variants have an end element.
    pub fn end(&self) -> Element {
        match self {
            Segment::LineTo { end, .. } => *end,
            Segment::QuadBezTo { end, .. } => *end,
            Segment::CubBezTo { end, .. } => *end,
        }
    }
}
