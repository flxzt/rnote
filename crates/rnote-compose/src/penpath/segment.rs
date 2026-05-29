// Imports
use super::Element;
use crate::Transformable;
use crate::ext::DPose2Ext;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
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
        #[serde(rename = "cp", with = "crate::serialize::glam_vector2_dp3")]
        /// The quadratic curve control point.
        cp: Vector2,
        #[serde(rename = "end")]
        /// The quadratic curve end.
        end: Element,
    },
    #[serde(rename = "cubbezto", alias = "cubbez")]
    /// A cubic-bezier-to segment.
    CubBezTo {
        #[serde(rename = "cp1", with = "crate::serialize::glam_vector2_dp3")]
        /// The cubic curve first control point.
        cp1: Vector2,
        #[serde(rename = "cp2", with = "crate::serialize::glam_vector2_dp3")]
        /// The cubic curve second control point.
        cp2: Vector2,
        #[serde(rename = "end")]
        /// The cubic curve end.
        end: Element,
    },
}

impl Transformable for Segment {
    fn translate(&mut self, offset: Vector2) {
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

    fn rotate(&mut self, angle: f64, center: Vector2) {
        let pose = DPose2::IDENTITY.append_rotation_wrt_center(angle, center);
        match self {
            Self::LineTo { end } => {
                end.pos = pose.transform_point(end.pos);
            }
            Self::QuadBezTo { cp, end } => {
                *cp = pose.transform_point(*cp);
                end.pos = pose.transform_point(end.pos);
            }
            Self::CubBezTo { cp1, cp2, end } => {
                *cp1 = pose.transform_point(*cp1);
                *cp2 = pose.transform_point(*cp2);
                end.pos = pose.transform_point(end.pos);
            }
        }
    }

    fn scale(&mut self, scale: Vector2) {
        match self {
            Self::LineTo { end } => {
                end.pos *= scale;
            }
            Self::QuadBezTo { cp, end } => {
                *cp *= scale;
                end.pos *= scale;
            }
            Self::CubBezTo { cp1, cp2, end } => {
                *cp1 *= scale;
                *cp2 *= scale;
                end.pos *= scale;
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
