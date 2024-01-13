// Imports
use super::Line;
use crate::ext::Vector2Ext;
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use kurbo::{PathEl, Shape};
use na::Rotation2;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

/// All doc-comments of this file and [ArrowBuilder][crate::builders] rely on the following
/// graphic:
///
/// ```text
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
///
/// Where `lline`, `tip`, `start` and `rline` represent a vector of the arrow.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename = "arrow")]
pub struct Arrow {
    /// Start of the arrow.
    pub start: na::Vector2<f64>,

    /// Tip of the arow.
    pub tip: na::Vector2<f64>,
}

impl Transformable for Arrow {
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

        self.start = isometry.transform_point(&self.start.into()).coords;
        self.tip = isometry.transform_point(&self.tip.into()).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.tip = self.tip.component_mul(&scale);
    }
}

impl Shapeable for Arrow {
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

    fn outline_path(&self) -> kurbo::BezPath {
        self.to_kurbo(None)
    }
}

impl Arrow {
    /// The tip lines (rline & lline) default length, is the actual length when
    /// no stroke width is associated with the arrow, if there is, it is used as
    /// the base length for the actual width calculation.
    const TIP_LINES_DEFAULT_LENGTH: f64 = 10.0;

    /// The angle for `rline` and `lline` to the stem of the arrow.
    const TIP_LINES_STEM_OBTUSE_ANGLE: f64 = (13.0 / 16.0) * std::f64::consts::PI;

    /// The default direction vector (the stem) if the stem has length 0.
    const DEFAULT_DIRECTION_VECTOR: na::Vector2<f64> = na::Vector2::new(1.0, 0.0);

    /// Creating a new arrow with the given start and tip vectors.
    pub fn new(start: na::Vector2<f64>, tip: na::Vector2<f64>) -> Self {
        Self { start, tip }
    }

    /// Split the stem of the arrow into the given number of lines.
    pub fn split(&self, n_splits: u32) -> Vec<Line> {
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

    /// Convert to kurbo shape.
    pub fn to_kurbo(&self, stroke_width: Option<f64>) -> kurbo::BezPath {
        let mut bez_path =
            kurbo::Line::new(self.start.to_kurbo_point(), self.tip.to_kurbo_point()).to_path(0.25);
        let tip = self.tip.to_kurbo_point();
        let lline = self.compute_lline(stroke_width).to_kurbo_point();
        let rline = self.compute_rline(stroke_width).to_kurbo_point();

        bez_path.extend([
            PathEl::MoveTo(lline),
            PathEl::LineTo(tip),
            PathEl::LineTo(rline),
        ]);

        bez_path
    }

    /// Compute the `lline` of the arrow tip.
    ///
    /// Optionally add the stroke width to adjust the length of the line.
    pub fn compute_lline(&self, stroke_width: Option<f64>) -> na::Vector2<f64> {
        let vec_a =
            self.compute_stem_direction_vector() * Self::compute_tip_lines_length(stroke_width);
        let rotation_matrix = Rotation2::new(Self::TIP_LINES_STEM_OBTUSE_ANGLE);

        rotation_matrix * vec_a + self.tip
    }

    /// Compute the `rline` of the arrow tip.
    ///
    /// Optionally add the stroke width to adjust the length of the line.
    pub fn compute_rline(&self, stroke_width: Option<f64>) -> na::Vector2<f64> {
        let vec_b =
            self.compute_stem_direction_vector() * Self::compute_tip_lines_length(stroke_width);
        let rotation_matrix = Rotation2::new(-Self::TIP_LINES_STEM_OBTUSE_ANGLE);

        rotation_matrix * vec_b + self.tip
    }

    /// Compute the bounds of the arrow in respect to the given stroke width.
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

    /// Compute the normalized direction vector from `start` to `tip`.
    fn compute_stem_direction_vector(&self) -> na::Vector2<f64> {
        let direction_vector = self.tip - self.start;

        if direction_vector.norm() == 0.0 {
            Self::DEFAULT_DIRECTION_VECTOR
        } else {
            direction_vector / direction_vector.norm()
        }
    }

    /// Compute the length of the tip lines.
    ///
    /// Optionally add the stroke width to adjust the length of the line.
    fn compute_tip_lines_length(stroke_width: Option<f64>) -> f64 {
        let factor = stroke_width.unwrap_or(0.0);
        Self::TIP_LINES_DEFAULT_LENGTH * (1.0 + 0.18 * factor)
    }
}
