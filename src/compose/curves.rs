use serde::{Deserialize, Serialize};

use crate::strokes::strokebehaviour::{self, StrokeBehaviour};

use super::geometry;
use super::shapes::Rectangle;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "line")]
pub struct Line {
    #[serde(rename = "start")]
    pub start: na::Vector2<f64>,
    #[serde(rename = "end")]
    pub end: na::Vector2<f64>,
}

impl StrokeBehaviour for Line {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.start += offset;
        self.end += offset;
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        let mut isometry = na::Isometry2::identity();
        isometry.append_rotation_wrt_point_mut(&na::UnitComplex::new(angle), &center);

        self.start = (isometry * na::Point2::from(self.start)).coords;
        self.end = (isometry * na::Point2::from(self.end)).coords;
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        let mid = (self.end + self.start) / 2.0;
        let half_vec = (self.end - self.start) / 2.0;

        self.start = mid - half_vec.component_mul(&scale);
        self.end = mid + half_vec.component_mul(&scale);
    }

    fn shear(&mut self, shear: nalgebra::Vector2<f64>) {
        let mid = (self.end + self.start) / 2.0;
        let half_vec = (self.end - self.start) / 2.0;

        let mut shear_matrix = na::Matrix3::<f64>::identity();
        shear_matrix[(0, 1)] = shear[0].tan();
        shear_matrix[(1, 0)] = shear[1].tan();

        let half_vec: na::Vector2<f64> = na::Point2::from_homogeneous(
            shear_matrix * na::Point2::from(half_vec).to_homogeneous(),
        )
        .unwrap()
        .coords;

        self.start = mid - half_vec;
        self.end = mid + half_vec;
    }
}

impl Line {
    pub fn global_aabb(&self) -> p2d::bounding_volume::AABB {
        geometry::aabb_new_positive(na::Point2::from(self.start), na::Point2::from(self.end))
    }

    pub fn line_w_width_to_rect(self, width: f64) -> Rectangle {
        let vec = self.end - self.start;
        let magn = vec.magnitude();
        let angle = na::Rotation2::rotation_between(&na::Vector2::x(), &vec).angle();

        Rectangle {
            cuboid: p2d::shape::Cuboid::new(na::vector![magn / 2.0, width / 2.0]),
            transform: strokebehaviour::StrokeTransform::new_w_isometry(na::Isometry2::new(
                self.start + vec / 2.0,
                angle,
            )),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct QuadBezier {
    pub start: na::Vector2<f64>,
    pub cp: na::Vector2<f64>,
    pub end: na::Vector2<f64>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct CubicBezier {
    pub start: na::Vector2<f64>,
    pub cp1: na::Vector2<f64>,
    pub cp2: na::Vector2<f64>,
    pub end: na::Vector2<f64>,
}

/// Bezier Curves

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
fn quadbez_coeff_c(p0: na::Vector2<f64>) -> na::Vector2<f64> {
    p0
}

// calculating the bezier curve for t: between 0.0 and 1.0
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
fn quadbez_derive_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    quad_bezier_derive_coeff_a(p0, p1, p2) * t + quadbez_derive_coeff_b(p0, p1)
}

fn cubbez_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    p3: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    let transform_matrix = na::matrix![
        1.0, 0.0, 0.0, 0.0;
        -3.0, 3.0, 0.0, 0.0;
        3.0, -6.0, 3.0, 0.0;
        -1.0, 3.0, -3.0, 1.0
    ];
    let p_matrix = na::matrix![
        p0[0], p0[1];
        p1[0], p1[1];
        p2[0], p2[1];
        p3[0], p3[1]
    ];

    (na::vector![1.0, t, t.powi(2), t.powi(3)].transpose() * transform_matrix * p_matrix)
        .transpose()
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

// See 'Conversion between Cubic Bezier Curves and Catmull-Rom Splines'
pub fn gen_cubbez_w_catmull_rom(
    first: na::Vector2<f64>,
    second: na::Vector2<f64>,
    third: na::Vector2<f64>,
    forth: na::Vector2<f64>,
) -> Option<CubicBezier> {
    // Tension factor (tau)
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

    let start_to_end = cubbez.end - cubbez.start;
    // returns early to prevent NaN when calculating the normals.
    if start_to_end.magnitude() == 0.0 {
        return None;
    }

    Some(cubbez)
}

pub fn gen_line(first: na::Vector2<f64>, second: na::Vector2<f64>) -> Option<Line> {
    let line = Line {
        start: first,
        end: second,
    };

    let start_to_end = line.end - line.start;

    // returns early to prevent NaN when calculating the normals.
    if start_to_end.magnitude() == 0.0 {
        return None;
    }

    Some(line)
}

/// Calcs quadratic bezier t at angle condition ( in rad ) to minimize max error when flattening the curve.
/// returns the t for the angle condition. is between 0.0 and 1.0 if the condition is met and the quadbez should be splitted.
/// See "precise offsetting of quadratic bezier curves, Section 3.3 split curve by angle"
pub fn calc_quadbez_angle_condition(quad_to_split: QuadBezier, angle: f64) -> f64 {
    let m = angle.tan();

    let a = quadbez_coeff_a(quad_to_split.start, quad_to_split.cp, quad_to_split.end);
    let b = quadbez_coeff_b(quad_to_split.start, quad_to_split.cp);

    (m * (b[0].powi(2) + b[1].powi(2)))
        / ((a[0] * b[1] - a[1] * b[0]).abs() - m * (a[0] * b[0] + a[1] * b[1]))
}

/// splitting offsetted quadratic bezier curve at critical points where offset dist < curvature radius minimize cusps
/// returns the splitted quad beziers, and possible split points t1, t2
/// See "precise offsetting of quadratic bezier curves, Section 3.4 Handling cusps"
pub fn split_offsetted_quadbez_critical_points(
    quad_to_split: QuadBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
) -> (Vec<QuadBezier>, Option<f64>, Option<f64>) {
    let mut quads = Vec::new();

    let max_offset_dist = start_offset_dist.max(end_offset_dist);

    let coeff_a =
        quad_bezier_derive_coeff_a(quad_to_split.start, quad_to_split.cp, quad_to_split.end);
    let coeff_b = quadbez_derive_coeff_b(quad_to_split.start, quad_to_split.cp);

    // Calculate critical points (local curvature less or equals offset witdh)
    let (mut t1, mut t2) = quadbez_solve_critical_points(coeff_a, coeff_b, max_offset_dist);

    if t2 < t1 {
        std::mem::swap(&mut t1, &mut t2);
    }
    let mut option_t1 = None;
    let mut option_t2 = None;

    if t1 > 0.0 && t1 < 1.0 {
        let (t1_first, t1_second) = split_quadbez(quad_to_split, t1);
        quads.push(t1_first);
        option_t1 = Some(t1);

        if t2 > 0.0 && t2 < 1.0 {
            let (t2_first, t2_second) = split_quadbez(t1_second, t2);
            quads.push(t2_first);
            quads.push(t2_second);

            option_t2 = Some(t2);
        } else {
            quads.push(t1_second);
        }
    } else {
        quads.push(quad_to_split);
    }

    (quads, option_t1, option_t2)
}

/// Split a quadratic bezier curve into two quadratics, interpolation value z: between 0.0 and 1.0
pub fn split_quadbez(quad_to_split: QuadBezier, z: f64) -> (QuadBezier, QuadBezier) {
    let p0 = quad_to_split.start;
    let p1 = quad_to_split.cp;
    let p2 = quad_to_split.end;

    let first_splitted = QuadBezier {
        start: p0,
        cp: z * p1 - (z - 1.0) * p0,
        end: z.powi(2) * p2 - 2.0 * z * (z - 1.0) * p1 + (z - 1.0).powi(2) * p0,
    };

    let second_splitted = QuadBezier {
        start: z.powi(2) * p2 - 2.0 * z * (z - 1.0) * p1 + (z - 1.0).powi(2) * p0,
        cp: z * p2 - (z - 1.0) * p1,
        end: p2,
    };

    (first_splitted, second_splitted)
}

/// Split a cubic bezier into two at t where t > 0.0, < 1.0
pub fn split_cubbez(cubbez: CubicBezier, t: f64) -> (CubicBezier, CubicBezier) {
    let a0 = cubbez.start;
    let a1 = cubbez.cp1;
    let a2 = cubbez.cp2;
    let a3 = cubbez.end;

    let b1 = a0.lerp(&a1, t);
    let a12 = a1.lerp(&a2, t);
    let b2 = b1.lerp(&a12, t);
    let c2 = a2.lerp(&a3, t);
    let c1 = a12.lerp(&c2, t);
    let b3 = b2.lerp(&c1, t);

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

/// Approximating a cubic bezier with a quadratic bezier
pub fn approx_cubbez_with_quadbez(cubbez: CubicBezier) -> QuadBezier {
    let start = cubbez.start;
    let cp = cubbez.cp1.lerp(&cubbez.cp2, 0.5);
    let end = cubbez.end;

    QuadBezier { start, cp, end }
}

/// Approximating a cubic bezier with lines, given the number of splits
pub fn approx_cubbez_with_lines(cubbez: CubicBezier, n_splits: i32) -> Vec<Line> {
    let mut lines = Vec::new();

    for i in 0..n_splits {
        let start_t = f64::from(i) / f64::from(n_splits);
        let end_t = f64::from(i + 1) / f64::from(n_splits);

        lines.push(Line {
            start: cubbez_calc(cubbez.start, cubbez.cp1, cubbez.cp2, cubbez.end, start_t),
            end: cubbez_calc(cubbez.start, cubbez.cp1, cubbez.cp2, cubbez.end, end_t),
        })
    }

    lines
}

/// Approximating a cubic bezier with lines, splitted based on critical points and the angle condition
pub fn approx_offsetted_cubbez_with_lines_w_subdivision(
    cubbez: CubicBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    angle_split: f64,
) -> Vec<Line> {
    let t_mid = 0.5;
    let mid_offset_dist = start_offset_dist + (end_offset_dist - start_offset_dist) * t_mid;
    let mut lines = Vec::new();

    let (first_cubic, second_cubic) = split_cubbez(cubbez, t_mid);
    let first_quad = approx_cubbez_with_quadbez(first_cubic);
    let second_quad = approx_cubbez_with_quadbez(second_cubic);

    let mut quads_to_approx = vec![];

    let (mut first_quads, _, _) =
        split_offsetted_quadbez_critical_points(first_quad, start_offset_dist, mid_offset_dist);
    quads_to_approx.append(&mut first_quads);

    let (mut second_quads, _, _) =
        split_offsetted_quadbez_critical_points(second_quad, mid_offset_dist, end_offset_dist);
    quads_to_approx.append(&mut second_quads);

    for mut quad_to_approx in quads_to_approx {
        // Abort after 10 iterations
        let mut i = 0;

        while i < 10 {
            let t = calc_quadbez_angle_condition(quad_to_approx, angle_split);

            if (0.0..1.0).contains(&t) {
                let (first, second) = split_quadbez(quad_to_approx, t);

                lines.push(Line {
                    start: first.start,
                    end: first.end,
                });

                quad_to_approx = second;
            } else {
                lines.push(Line {
                    start: quad_to_approx.start,
                    end: quad_to_approx.end,
                });

                // Break if angle conditions is no longer met
                break;
            }
            i += 1;
        }
    }

    lines
}

// Returns offset distance of a quadratic bezier at t where t > 0.0, < 1.0
pub fn quadbez_calc_offset_dist_at_t(
    _quad: QuadBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    t: f64,
) -> f64 {
    start_offset_dist + (end_offset_dist - start_offset_dist) * t
}
