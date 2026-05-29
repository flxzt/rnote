// Imports
use super::Line;
use crate::Transformable;
use crate::ext::{DPose2Ext, Vector2Ext};
use crate::shapes::Shapeable;
use kurbo::{PathEl, Shape};
use p2d::bounding_volume::Aabb;
use p2d::glamx::prelude::{DPose2, DRot2};
use p2d::math::Vector2;
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
    pub start: Vector2,

    /// Tip of the arow.
    pub tip: Vector2,
}

impl Transformable for Arrow {
    fn translate(&mut self, offset: Vector2) {
        self.start += offset;
        self.tip += offset;
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        let pose = DPose2::from_rotation_wrt_center(angle, center);
        self.start = pose.transform_point(self.start);
        self.tip = pose.transform_point(self.tip);
    }

    fn scale(&mut self, scale: Vector2) {
        self.start *= scale;
        self.tip *= scale;
    }
}

impl Shapeable for Arrow {
    fn bounds(&self) -> Aabb {
        self.internal_compute_bounds(None)
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let n_splits = super::hitbox_elems_for_shape_len((self.tip - self.start).length());
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

    /// Creating a new arrow with the given start and tip vectors.
    pub fn new(start: Vector2, tip: Vector2) -> Self {
        Self { start, tip }
    }

    /// Split the stem of the arrow into the given number of lines.
    pub fn split(&self, n_splits: i32) -> Vec<Line> {
        (0..n_splits)
            .map(|i| {
                let sub_start = self
                    .start
                    .lerp(self.tip, f64::from(i) / f64::from(n_splits));
                let sub_end = self
                    .start
                    .lerp(self.tip, f64::from(i + 1) / f64::from(n_splits));

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
    pub fn compute_lline(&self, stroke_width: Option<f64>) -> Vector2 {
        let vec_a =
            self.compute_stem_direction_vector() * Self::compute_tip_lines_length(stroke_width);
        DRot2::new(Self::TIP_LINES_STEM_OBTUSE_ANGLE) * vec_a + self.tip
    }

    /// Compute the `rline` of the arrow tip.
    ///
    /// Optionally add the stroke width to adjust the length of the line.
    pub fn compute_rline(&self, stroke_width: Option<f64>) -> Vector2 {
        let vec_b =
            self.compute_stem_direction_vector() * Self::compute_tip_lines_length(stroke_width);
        DRot2::new(-Self::TIP_LINES_STEM_OBTUSE_ANGLE) * vec_b + self.tip
    }

    /// Compute the bounds of the arrow in respect to the given stroke width.
    pub fn internal_compute_bounds(&self, stroke_width: Option<f64>) -> Aabb {
        let lline = self.compute_lline(stroke_width);
        let rline = self.compute_rline(stroke_width);
        Aabb::from_points([lline, rline, self.start, self.tip])
    }

    /// Compute the normalized direction vector from `start` to `tip`.
    fn compute_stem_direction_vector(&self) -> Vector2 {
        let direction_vector = self.tip - self.start;
        if direction_vector.length() == 0.0 {
            Vector2::X
        } else {
            direction_vector / direction_vector.length()
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
