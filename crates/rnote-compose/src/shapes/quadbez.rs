// Imports
use super::line::Line;
use super::CubicBezier;
use crate::ext::{KurboShapeExt, Vector2Ext};
use crate::shapes::Shapeable;
use crate::transform::Transformable;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "quadratic_bezier")]
/// A quadratic bezier curve.
pub struct QuadraticBezier {
    #[serde(rename = "start", with = "crate::serialize::na_vector2_f64_dp3")]
    /// Start coordinates.
    pub start: na::Vector2<f64>,
    #[serde(rename = "cp", with = "crate::serialize::na_vector2_f64_dp3")]
    /// Control point coordinates.
    pub cp: na::Vector2<f64>,
    #[serde(rename = "end", with = "crate::serialize::na_vector2_f64_dp3")]
    /// End coordinates.
    pub end: na::Vector2<f64>,
}

impl Transformable for QuadraticBezier {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.start += offset;
        self.cp += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = isometry.transform_point(&self.start.into()).coords;
        self.cp = isometry.transform_point(&self.cp.into()).coords;
        self.end = isometry.transform_point(&self.end.into()).coords;
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.cp = self.cp.component_mul(&scale);
        self.end = self.end.component_mul(&scale);
    }
}

impl Shapeable for QuadraticBezier {
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
        kurbo::QuadBez::new(
            self.start.to_kurbo_point(),
            self.cp.to_kurbo_point(),
            self.end.to_kurbo_point(),
        )
        .to_path(0.25)
    }
}

impl QuadraticBezier {
    /// Split itself into two quadratic bezier curves, at interpolation value z ranging [0.0 - 1.0].
    pub fn split(&self, z: f64) -> (QuadraticBezier, QuadraticBezier) {
        let p0 = self.start;
        let p1 = self.cp;
        let p2 = self.end;

        let first_split = QuadraticBezier {
            start: p0,
            cp: z * p1 - (z - 1.0) * p0,
            end: z.powi(2) * p2 - 2.0 * z * (z - 1.0) * p1 + (z - 1.0).powi(2) * p0,
        };

        let second_split = QuadraticBezier {
            start: z.powi(2) * p2 - 2.0 * z * (z - 1.0) * p1 + (z - 1.0).powi(2) * p0,
            cp: z * p2 - (z - 1.0) * p1,
            end: p2,
        };

        (first_split, second_split)
    }

    /// Convert to a cubic bezier (raising the order of a bezier curve is without losses).
    pub fn to_cubic_bezier(&self) -> CubicBezier {
        CubicBezier {
            start: self.start,
            cp1: self.start + (2.0 / 3.0) * (self.cp - self.start),
            cp2: self.end + (2.0 / 3.0) * (self.cp - self.end),
            end: self.end,
        }
    }

    /// Approximate with lines, given the number of splits.
    pub fn approx_with_lines(&self, n_splits: u32) -> Vec<Line> {
        let mut lines = Vec::new();

        for i in 0..n_splits {
            let start_t = f64::from(i) / f64::from(n_splits);
            let end_t = f64::from(i + 1) / f64::from(n_splits);

            lines.push(Line {
                start: quadbez_calc(self.start, self.cp, self.end, start_t),
                end: quadbez_calc(self.start, self.cp, self.end, end_t),
            })
        }

        lines
    }
}

/// Coefficient a of quadratic bezier in polynomial form: C = a * t^2 + b * t + c
fn quadbez_coeff_a(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
) -> na::Vector2<f64> {
    p2 - 2.0 * p1 + p0
}

/// Coefficient b of quadratic bezier in polynomial form: C = a * t^2 + b * t + c
fn quadbez_coeff_b(p0: na::Vector2<f64>, p1: na::Vector2<f64>) -> na::Vector2<f64> {
    2.0 * p1 - 2.0 * p0
}

/// Coefficient c of quadratic bezier in polynomial form: C = a * t^2 + b * t + c
#[allow(unused)]
fn quadbez_coeff_c(p0: na::Vector2<f64>) -> na::Vector2<f64> {
    p0
}

/// calculating the value of a bezier curve with its support points, for t: between 0.0 and 1.0
#[allow(unused)]
pub fn quadbez_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    quadbez_coeff_a(p0, p1, p2) * t.powi(2) + quadbez_coeff_b(p0, p1) * t + quadbez_coeff_c(p0)
}

/// Coefficient a of quadratic bezier derivation in polynomial form: C' = a * t + b
fn quad_bezier_derive_coeff_a(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
) -> na::Vector2<f64> {
    2.0 * p2 - 4.0 * p1 + 2.0 * p0
}

/// Coefficient b of quadratic bezier derivation in polynomial form: C' = a * t + b
fn quadbez_derive_coeff_b(p0: na::Vector2<f64>, p1: na::Vector2<f64>) -> na::Vector2<f64> {
    2.0 * p1 - 2.0 * p0
}

#[allow(unused)]
/// calculating the derivative of the bezier curve for t: between 0.0 and 1.0
pub fn quadbez_derive_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    quad_bezier_derive_coeff_a(p0, p1, p2) * t + quadbez_derive_coeff_b(p0, p1)
}
