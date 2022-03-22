use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use crate::shapes::ShapeBehaviour;
use crate::transform::TransformBehaviour;

use super::line::Line;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "quadratic_bezier")]
pub struct QuadraticBezier {
    #[serde(rename = "start")]
    pub start: na::Vector2<f64>,
    #[serde(rename = "cp")]
    pub cp: na::Vector2<f64>,
    #[serde(rename = "end")]
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
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        let mut aabb = AABB::new(na::Point2::from(self.start), na::Point2::from(self.end));
        aabb.take_point(na::Point2::from(self.cp));
        aabb
    }
}

impl QuadraticBezier {
    // Returns offset distance of a quadratic bezier at t where t > 0.0, < 1.0
    pub fn quadbez_calc_offset_dist_at_t(
        &self,
        start_offset_dist: f64,
        end_offset_dist: f64,
        t: f64,
    ) -> f64 {
        start_offset_dist + (end_offset_dist - start_offset_dist) * t
    }

    /// Calcs quadratic bezier t at angle condition ( in rad ) to minimize max error when flattening the curve.
    /// returns the t for the angle condition. is between 0.0 and 1.0 if the condition is met and the quadbez should be splitted.
    /// See "precise offsetting of quadratic bezier curves, Section 3.3 split curve by angle"
    pub fn calc_quadbez_angle_condition(&self, angle: f64) -> f64 {
        let m = angle.tan();

        let a = quadbez_coeff_a(self.start, self.cp, self.end);
        let b = quadbez_coeff_b(self.start, self.cp);

        (m * (b[0].powi(2) + b[1].powi(2)))
            / ((a[0] * b[1] - a[1] * b[0]).abs() - m * (a[0] * b[0] + a[1] * b[1]))
    }

    /// splitting offsetted quadratic bezier curve at critical points where offset dist < curvature radius minimize cusps
    /// returns the splitted quad beziers, and possible split points t1, t2
    /// See "precise offsetting of quadratic bezier curves, Section 3.4 Handling cusps"
    pub fn split_offsetted_at_critical_points(
        &self,
        start_offset_dist: f64,
        end_offset_dist: f64,
    ) -> (Vec<QuadraticBezier>, Option<f64>, Option<f64>) {
        let mut quads = Vec::new();

        let max_offset_dist = start_offset_dist.max(end_offset_dist);

        let coeff_a = quad_bezier_derive_coeff_a(self.start, self.cp, self.end);
        let coeff_b = quadbez_derive_coeff_b(self.start, self.cp);

        // Calculate critical points (local curvature less or equals offset witdh)
        let (mut t1, mut t2) = quadbez_solve_critical_points(coeff_a, coeff_b, max_offset_dist);

        if t2 < t1 {
            std::mem::swap(&mut t1, &mut t2);
        }
        let mut option_t1 = None;
        let mut option_t2 = None;

        if t1 > 0.0 && t1 < 1.0 {
            let (t1_first, t1_second) = self.split(t1);
            quads.push(t1_first);
            option_t1 = Some(t1);

            if t2 > 0.0 && t2 < 1.0 {
                let (t2_first, t2_second) = t1_second.split(t2);
                quads.push(t2_first);
                quads.push(t2_second);

                option_t2 = Some(t2);
            } else {
                quads.push(t1_second);
            }
        } else {
            quads.push(*self);
        }

        (quads, option_t1, option_t2)
    }

    /// Split a quadratic bezier curve into two quadratics, interpolation value z: between 0.0 and 1.0
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

    /// Approximating a quadratic bezier with lines, given the number of splits
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
}

// Coefficient a of quadratic bezier in polynomial form: C = a * t^2 + b * t + c
fn quadbez_coeff_a(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
) -> na::Vector2<f64> {
    p2 - 2.0 * p1 + p0
}

// Coefficient b of quadratic bezier in polynomial form: C = a * t^2 + b * t + c
fn quadbez_coeff_b(p0: na::Vector2<f64>, p1: na::Vector2<f64>) -> na::Vector2<f64> {
    2.0 * p1 - 2.0 * p0
}

// Coefficient c of quadratic bezier in polynomial form: C = a * t^2 + b * t + c
#[allow(dead_code)]
fn quadbez_coeff_c(p0: na::Vector2<f64>) -> na::Vector2<f64> {
    p0
}

// calculating the value of a bezier curve with its support points, for t: between 0.0 and 1.0
#[allow(dead_code)]
fn quadbez_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    quadbez_coeff_a(p0, p1, p2) * t.powi(2) + quadbez_coeff_b(p0, p1) * t + quadbez_coeff_c(p0)
}

// Coefficient a of quadratic bezier derivation in polynomial form: C' = a * t + b
fn quad_bezier_derive_coeff_a(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
) -> na::Vector2<f64> {
    2.0 * p2 - 4.0 * p1 + 2.0 * p0
}

// Coefficient b of quadratic bezier derivation in polynomial form: C' = a * t + b
fn quadbez_derive_coeff_b(p0: na::Vector2<f64>, p1: na::Vector2<f64>) -> na::Vector2<f64> {
    2.0 * p1 - 2.0 * p0
}

// calculating the derivative of the bezier curve for t: between 0.0 and 1.0
#[allow(dead_code)]
fn quadbez_derive_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    quad_bezier_derive_coeff_a(p0, p1, p2) * t + quadbez_derive_coeff_b(p0, p1)
}

/// Returns (t1, t2) with t1, t2 between 0.0 and 1.0
fn quadbez_solve_critical_points(
    a: na::Vector2<f64>,
    b: na::Vector2<f64>,
    dist: f64,
) -> (f64, f64) {
    let t1 = (-(a[0] * b[0] + a[1] + b[1])
        + ((a[0] * b[0] + a[1] * b[1]).powi(2)
            - (a[0].powi(2) + a[1].powi(2))
                * (b[0].powi(2) + b[1].powi(2)
                    - (dist.powi(2) * (b[0] * a[1] - a[0] * b[1]).powi(2)).cbrt()))
        .sqrt())
        / (a[0].powi(2) + a[1].powi(2));

    let t2 = (-(a[0] * b[0] + a[1] + b[1])
        - ((a[0] * b[0] + a[1] * b[1]).powi(2)
            - (a[0].powi(2) + a[1].powi(2))
                * (b[0].powi(2) + b[1].powi(2)
                    - (dist.powi(2) * (b[0] * a[1] - a[0] * b[1]).powi(2)).cbrt()))
        .sqrt())
        / (a[0].powi(2) + a[1].powi(2));

    (t1, t2)
}
