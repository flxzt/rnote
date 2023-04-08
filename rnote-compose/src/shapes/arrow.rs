use kurbo::PathEl;
use na::Rotation2;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

use crate::helpers::Vector2Helpers;
use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;

use super::Line;

/// All doc-comments of this file and [`ArrowBuilder`] rely on the following
/// graphic:
/// ```
///         tip
///         /|\
///        / | \
///       /  |  \
///    lline |  rline
///          |
///          |
///          |
///         start
/// ```
/// Where `lline`, `tip`, `start` and `rline` represent a vector of the arrow.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename = "arrow")]
pub struct Arrow {
    /// The start of the arrow
    pub start: na::Vector2<f64>,

    /// The tip of the arow
    pub tip: na::Vector2<f64>,
}

impl TransformBehaviour for Arrow {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.start += offset;
        self.tip += offset;
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let isometry = {
            let mut isometry = na::Isometry2::identity();
            isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);
            isometry
        };

        self.start = (isometry * na::Point2::from(self.start)).coords;
        self.tip = (isometry * na::Point2::from(self.tip)).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.tip = self.tip.component_mul(&scale);
    }
}

impl ShapeBehaviour for Arrow {
    fn bounds(&self) -> Aabb {
        self.internal_compute_bounds(None)
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let n_splits = super::hitbox_elems_for_shape_len((self.tip - self.start).norm());

        self.split(n_splits)
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }
}

impl Arrow {
    /// The tip lines (rline & lline) default length, is the actual length when
    /// no stroke width is associated with the arrow, if there is, it is used as
    /// the base length for the actual width calculation.
    const TIP_LINES_DEFAULT_LENGTH: f64 = 32.0;

    /// The angle for `rline` and `lline` to the stem of the arrow.
    const TIP_LINES_STEM_ACUTE_ANGLE: f64 = (13.0 / 16.0) * std::f64::consts::PI;

    /// The default direction vector (the stem) if the stem has length 0.
    const DEFAULT_DIRECTION_VECTOR: na::Vector2<f64> = na::Vector2::new(1.0, 0.0);

    /// Creating a new arrow with the given start and tip vectors.
    pub fn new(start: na::Vector2<f64>, tip: na::Vector2<f64>) -> Self {
        Self { start, tip }
    }

    /// Splits the stem of the arrow into the given number of lines.
    pub fn split(&self, n_splits: i32) -> Vec<Line> {
        (0..n_splits)
            .map(|i| {
                let sub_start = self
                    .start
                    .lerp(&self.tip, f64::from(i) / f64::from(n_splits));
                let sub_end = self
                    .start
                    .lerp(&self.tip, f64::from(i + 1) / f64::from(n_splits));

                Line {
                    start: sub_start,
                    end: sub_end,
                }
            })
            .collect::<Vec<Line>>()
    }

    /// convert the arrow to kurbo-elements.
    pub fn to_kurbo(&self, stroke_width: Option<f64>) -> ArrowKurbo {
        let main = kurbo::Line::new(self.start.to_kurbo_point(), self.tip.to_kurbo_point());
        let tip_triangle = {
            let tip = self.tip.to_kurbo_point();
            let lline = self.compute_lline(stroke_width).to_kurbo_point();
            let rline = self.compute_rline(stroke_width).to_kurbo_point();

            kurbo::BezPath::from_vec(vec![
                PathEl::MoveTo(lline),
                PathEl::LineTo(tip),
                PathEl::LineTo(rline),
            ])
        };

        ArrowKurbo {
            stem: main,
            tip_triangle,
        }
    }

    /// Computes and returns `lline`.
    /// Optionally add the stroke width to adjust the length of the line.
    pub fn compute_lline(&self, stroke_width: Option<f64>) -> na::Vector2<f64> {
        let vec_a =
            self.compute_stem_direction_vector() * Self::compute_tip_lines_length(stroke_width);
        let rotation_matrix = Rotation2::new(Self::TIP_LINES_STEM_ACUTE_ANGLE);

        rotation_matrix * vec_a + self.tip
    }

    /// Computes and returns `rline`.
    /// Optionally add the stroke width to adjust the length of the line.
    pub fn compute_rline(&self, stroke_width: Option<f64>) -> na::Vector2<f64> {
        let vec_b =
            self.compute_stem_direction_vector() * Self::compute_tip_lines_length(stroke_width);
        let rotation_matrix = Rotation2::new(Self::TIP_LINES_STEM_ACUTE_ANGLE);

        rotation_matrix * vec_b + self.tip
    }

    /// Computes the bounds of the arrow in respect to the given stroke width.
    pub fn internal_compute_bounds(&self, stroke_width: Option<f64>) -> Aabb {
        let points: Vec<na::Point2<f64>> = {
            let lline = self.compute_lline(stroke_width);
            let rline = self.compute_rline(stroke_width);

            [lline, rline, self.start, self.tip]
                .into_iter()
                .map(|vector| na::Point2::new(vector.x, vector.y))
                .collect()
        };

        Aabb::from_points(&points)
    }

    /// Computes and returns the normalized direction vector from `start` to
    /// `tip`.
    fn compute_stem_direction_vector(&self) -> na::Vector2<f64> {
        let direction_vector = self.tip - self.start;

        if direction_vector.norm() == 0.0 {
            Self::DEFAULT_DIRECTION_VECTOR
        } else {
            direction_vector / direction_vector.norm()
        }
    }

    /// Computes and returns the length of the tip lines with the given stroke
    /// width.
    fn compute_tip_lines_length(stroke_width: Option<f64>) -> f64 {
        let factor = stroke_width.unwrap_or(0.0);
        Self::TIP_LINES_DEFAULT_LENGTH * (1.0 + 0.1 * factor)
    }
}

/// A helper struct which holds the kurbo-elements of the arrow.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrowKurbo {
    /// This holds the line from `start` -> `tip`.
    pub stem: kurbo::Line,

    /// This holds the line from `lline` -> `tip` -> `rline`.
    pub tip_triangle: kurbo::BezPath,
}
