use crate::helpers::AABBHelpers;
use crate::shapes::CubicBezier;
use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;

use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

use super::Element;

/// A single segment (usually of a path), containing Elements to be able to being drawn with variable width
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "segment")]
pub enum Segment {
    #[serde(rename = "dot")]
    Dot {
        #[serde(rename = "element")]
        element: Element,
    },
    #[serde(rename = "line")]
    Line {
        #[serde(rename = "start")]
        start: Element,
        #[serde(rename = "end")]
        end: Element,
    },
    #[serde(rename = "quadbez")]
    QuadBez {
        #[serde(rename = "start")]
        start: Element,
        #[serde(rename = "cp")]
        cp: na::Vector2<f64>,
        #[serde(rename = "end")]
        end: Element,
    },
    #[serde(rename = "cubbez")]
    CubBez {
        #[serde(rename = "start")]
        start: Element,
        #[serde(rename = "cp1")]
        cp1: na::Vector2<f64>,
        #[serde(rename = "cp2")]
        cp2: na::Vector2<f64>,
        #[serde(rename = "end")]
        end: Element,
    },
}

impl Segment {
    /// Creates the fitting segment from the available elements.
    pub fn new_from_elements(
        prev: Option<Element>,
        start: Option<Element>,
        current: Element,
        ahead: Option<Element>,
    ) -> Self {
        match (prev, start, ahead) {
            (Some(prev), Some(start), Some(ahead)) => {
                if let Some(cubbez) =
                    CubicBezier::gen_w_catmull_rom(prev.pos, start.pos, current.pos, ahead.pos)
                {
                    Segment::CubBez {
                        start: Element {
                            pos: cubbez.start,
                            ..start
                        },
                        cp1: cubbez.cp1,
                        cp2: cubbez.cp2,
                        end: Element {
                            pos: cubbez.end,
                            ..current
                        },
                    }
                } else {
                    Segment::QuadBez {
                        start,
                        cp: (2.0 * prev.pos - start.pos),
                        end: current,
                    }
                }
            }
            (Some(prev), Some(start), None) => Segment::QuadBez {
                start,
                cp: (2.0 * prev.pos - start.pos),
                end: current,
            },
            (None, Some(start), Some(ahead)) => Segment::QuadBez {
                start,
                cp: (2.0 * current.pos - ahead.pos),
                end: current,
            },
            (None, Some(start), None) => Segment::Line {
                start,
                end: current,
            },
            _ => Segment::Dot { element: current },
        }
    }
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
