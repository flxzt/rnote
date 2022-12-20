use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

use crate::helpers::{KurboHelpers, Vector2Helpers};
use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;

use super::line::Line;
use super::CubicBezier;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "quadratic_bezier")]
/// A quadratic bezier curve
pub struct QuadraticBezier {
    #[serde(rename = "start")]
    /// The curve start
    pub start: na::Vector2<f64>,
    #[serde(rename = "cp")]
    /// The curve control point
    pub cp: na::Vector2<f64>,
    #[serde(rename = "end")]
    /// The curve end
    pub end: na::Vector2<f64>,
}

impl TransformBehaviour for QuadraticBezier {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.start += offset;
        self.cp += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = (isometry * na::Point2::from(self.start)).coords;
        self.cp = (isometry * na::Point2::from(self.cp)).coords;
        self.end = (isometry * na::Point2::from(self.end)).coords;
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.start = self.start.component_mul(&scale);
        self.cp = self.cp.component_mul(&scale);
        self.end = self.end.component_mul(&scale);
    }
}

impl ShapeBehaviour for QuadraticBezier {
    fn bounds(&self) -> p2d::bounding_volume::Aabb {
        self.to_kurbo().bounding_box().bounds_as_p2d_aabb()
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        // TODO: should be depending on the actual curve length
        let n_splits = super::hitbox_elems_for_shape_len(self.to_kurbo().perimeter(0.1));

        self.approx_with_lines(n_splits)
            .into_iter()
            .map(|line| line.bounds())
            .collect()
    }
}

impl QuadraticBezier {
    /// Returns offset distance of a quadratic bezier at t with range [0.0, 1.0]. Currently a linear interpolation between the start and end offset.
    /// TODO: finding a better algorithm based on the actual curve length
    pub fn quadbez_calc_offset_dist_at_t(
        &self,
        start_offset_dist: f64,
        end_offset_dist: f64,
        t: f64,
    ) -> f64 {
        start_offset_dist + (end_offset_dist - start_offset_dist) * t
    }

    /// Split a quadratic bezier curve into two quadratics, at interpolation value z: between 0.0 and 1.0
    pub fn split(&self, z: f64) -> (QuadraticBezier, QuadraticBezier) {
        let p0 = self.start;
        let p1 = self.cp;
        let p2 = self.end;

        let first_splitted = QuadraticBezier {
            start: p0,
            cp: z * p1 - (z - 1.0) * p0,
            end: z.powi(2) * p2 - 2.0 * z * (z - 1.0) * p1 + (z - 1.0).powi(2) * p0,
        };

        let second_splitted = QuadraticBezier {
            start: z.powi(2) * p2 - 2.0 * z * (z - 1.0) * p1 + (z - 1.0).powi(2) * p0,
            cp: z * p2 - (z - 1.0) * p1,
            end: p2,
        };

        (first_splitted, second_splitted)
    }

    /// convert to a cubic bezier ( raising the order is without losses)
    pub fn to_cubic_bezier(&self) -> CubicBezier {
        CubicBezier {
            start: self.start,
            cp1: self.start + (2.0 / 3.0) * (self.cp - self.start),
            cp2: self.end + (2.0 / 3.0) * (self.cp - self.end),
            end: self.end,
        }
    }

    /// Approximating a quadratic bezier with lines, given the number of splits distributed evenly based on the t value.
    pub fn approx_with_lines(&self, n_splits: i32) -> Vec<Line> {
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

    /// to kurbo
    pub fn to_kurbo(&self) -> kurbo::QuadBez {
        kurbo::QuadBez::new(
            self.start.to_kurbo_point(),
            self.cp.to_kurbo_point(),
            self.end.to_kurbo_point(),
        )
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
