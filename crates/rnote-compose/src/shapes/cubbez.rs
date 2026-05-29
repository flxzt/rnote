// Imports
use super::line::Line;
use super::quadbez::QuadraticBezier;
use crate::ext::{DPose2Ext, KurboShapeExt, Vector2Ext};
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use p2d::glamx::prelude::DPose2;
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "cubic_bezier")]
/// A cubic bezier curve.
pub struct CubicBezier {
    #[serde(rename = "start", with = "crate::serialize::glam_vector2_dp3")]
    /// Start coordinate.
    pub start: Vector2,
    #[serde(rename = "cp1", with = "crate::serialize::glam_vector2_dp3")]
    /// First control point coordinate.
    pub cp1: Vector2,
    #[serde(rename = "cp2", with = "crate::serialize::glam_vector2_dp3")]
    /// Second control point coordinate.
    pub cp2: Vector2,
    #[serde(rename = "end", with = "crate::serialize::glam_vector2_dp3")]
    /// End coordinate.
    pub end: Vector2,
}

impl Transformable for CubicBezier {
    fn translate(&mut self, offset: Vector2) {
        self.start += offset;
        self.cp1 += offset;
        self.cp2 += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        let pose = DPose2::IDENTITY.append_rotation_wrt_center(angle, center);
        self.start = pose.transform_point(self.start);
        self.cp1 = pose.transform_point(self.cp1);
        self.cp2 = pose.transform_point(self.cp2);
        self.end = pose.transform_point(self.end);
    }

    fn scale(&mut self, scale: Vector2) {
        self.start *= scale;
        self.cp1 *= scale;
        self.cp2 *= scale;
        self.end *= scale;
    }
}

impl Shapeable for CubicBezier {
    fn bounds(&self) -> p2d::bounding_volume::Aabb {
        self.outline_path().bounding_box().bounds_to_p2d_aabb()
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        let n_splits = super::hitbox_elems_for_shape_len(self.outline_path().perimeter(0.25));

        self.approx_with_lines(n_splits)
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }

    fn outline_path(&self) -> kurbo::BezPath {
        kurbo::CubicBez::new(
            self.start.to_kurbo_point(),
            self.cp1.to_kurbo_point(),
            self.cp2.to_kurbo_point(),
            self.end.to_kurbo_point(),
        )
        .to_path(0.25)
    }
}

impl CubicBezier {
    /// Attempts to create a new cubic curve with the catmull-rom spline algorithm.
    /// Subsequent curves ( meaning, advancing the elements by one) have a smooth transition between them.
    ///
    /// See 'Conversion between Cubic Bezier Curves and Catmull-Rom Splines'.
    pub fn new_w_catmull_rom(
        first: Vector2,
        second: Vector2,
        third: Vector2,
        forth: Vector2,
    ) -> Option<Self> {
        // Tension factor
        let tension = 1.0;

        // Creating cubic bezier with catmull-rom
        let start = second;
        let cp1 = second + (third - first) / (6.0 * tension);
        let cp2 = third - (forth - second) / (6.0 * tension);
        let end = third;

        let cubbez = CubicBezier {
            start,
            cp1,
            cp2,
            end,
        };

        // returning None when the cubbez does not have a length to prevent NaN when calculating the normals for segments with variable width
        if (cubbez.end - cubbez.start).length() == 0.0 {
            return None;
        }

        Some(cubbez)
    }

    /// Split a cubic bezier into two at t where t in [0.0 - 1.0].
    pub fn split(&self, t: f64) -> (CubicBezier, CubicBezier) {
        let a0 = self.start;
        let a1 = self.cp1;
        let a2 = self.cp2;
        let a3 = self.end;

        let b1 = a0.lerp(a1, t);
        let a12 = a1.lerp(a2, t);
        let b2 = b1.lerp(a12, t);
        let c2 = a2.lerp(a3, t);
        let c1 = a12.lerp(c2, t);
        let b3 = b2.lerp(c1, t);

        (
            CubicBezier {
                start: a0,
                cp1: b1,
                cp2: b2,
                end: b3,
            },
            CubicBezier {
                start: b3,
                cp1: c1,
                cp2: c2,
                end: a3,
            },
        )
    }

    /// Approximate a cubic with a quadratic bezier curve.
    pub fn approx_with_quadbez(&self) -> QuadraticBezier {
        let start = self.start;
        let cp = self.cp1.lerp(self.cp2, 0.5);
        let end = self.end;

        QuadraticBezier { start, cp, end }
    }

    /// Approximate a cubic bezier with lines, given the number of splits.
    pub fn approx_with_lines(&self, n_splits: i32) -> Vec<Line> {
        let mut lines = Vec::new();

        for i in 0..n_splits {
            let start_t = f64::from(i) / f64::from(n_splits);
            let end_t = f64::from(i + 1) / f64::from(n_splits);

            lines.push(Line {
                start: cubbez_calc(self.start, self.cp1, self.cp2, self.end, start_t),
                end: cubbez_calc(self.start, self.cp1, self.cp2, self.end, end_t),
            })
        }

        lines
    }
}

/// Calculate a point on a cubic curve given t ranging [0.0, 1.0].
pub fn cubbez_calc(p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, t: f64) -> Vector2 {
    let t_square = t * t;
    let t_cube = t_square * t;
    let one_minus_t = 1. - t;
    let one_minus_t_square = one_minus_t * one_minus_t;
    let one_minus_t_cube = one_minus_t_square * one_minus_t;
    p0 * one_minus_t_cube
        + p1 * 3.0 * one_minus_t_square * t
        + p2 * 3.0 * one_minus_t * t_square
        + p3 * t_cube
}
