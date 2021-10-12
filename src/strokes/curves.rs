use serde::{Deserialize, Serialize};

use super::Element;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Line {
    pub start: na::Vector2<f64>,
    pub end: na::Vector2<f64>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct QuadBezier {
    pub start: na::Vector2<f64>,
    pub cp: na::Vector2<f64>,
    pub end: na::Vector2<f64>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct CubicBezier {
    pub start: na::Vector2<f64>,
    pub cp1: na::Vector2<f64>,
    pub cp2: na::Vector2<f64>,
    pub end: na::Vector2<f64>,
}

pub fn vector2_unit_tang(vec: na::Vector2<f64>) -> na::Vector2<f64> {
    if vec.magnitude() > 0.0 {
        vec.normalize()
    } else {
        na::Vector2::<f64>::from_element(0.0)
    }
}

pub fn vector2_unit_norm(vec: na::Vector2<f64>) -> na::Vector2<f64> {
    let rot_90deg = na::Rotation2::new(std::f64::consts::PI / 2.0);

    let normalized = if vec.magnitude() > 0.0 {
        vec.normalize()
    } else {
        return na::Vector2::<f64>::from_element(0.0);
    };

    rot_90deg * normalized
}

/// Bezier Curves

// Coefficient a of quadratic bezier in polynomial form: C' = a * t^2 + b * t + c
#[allow(dead_code)]
fn quad_bezier_coeff_a(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
) -> na::Vector2<f64> {
    p2 - 2.0 * p1 + p0
}

// Coefficient b of quadratic bezier in polynomial form: C' = a * t^2 + b * t + c
#[allow(dead_code)]
fn quad_bezier_coeff_b(p0: na::Vector2<f64>, p1: na::Vector2<f64>) -> na::Vector2<f64> {
    2.0 * p1 - 2.0 * p0
}

// Coefficient c of quadratic bezier in polynomial form: C' = a * t^2 + b * t + c
#[allow(dead_code)]
fn quad_bezier_coeff_c(p0: na::Vector2<f64>) -> na::Vector2<f64> {
    p0
}

// calculating the bezier curve for t: between 0.0 and 1.0
#[allow(dead_code)]
fn quad_bezier_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    quad_bezier_coeff_a(p0, p1, p2) * t.powi(2)
        + quad_bezier_coeff_b(p0, p1) * t
        + quad_bezier_coeff_c(p0)
}

// Coefficient a of quadratic bezier derivation in polynomial form: C' = a * t + b
#[allow(dead_code)]
fn quad_bezier_derive_coeff_a(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
) -> na::Vector2<f64> {
    2.0 * (p2 - 2.0 * p1 + p0)
}

// Coefficient b of quadratic bezier derivation in polynomial form: C' = a * t + b
#[allow(dead_code)]
fn quad_bezier_derive_coeff_b(p0: na::Vector2<f64>, p1: na::Vector2<f64>) -> na::Vector2<f64> {
    2.0 * p1 - 2.0 * p0
}

// calculating the derivative of the bezier curve for t: between 0.0 and 1.0
#[allow(dead_code)]
fn quad_bezier_derive_calc(
    p0: na::Vector2<f64>,
    p1: na::Vector2<f64>,
    p2: na::Vector2<f64>,
    t: f64,
) -> na::Vector2<f64> {
    quad_bezier_derive_coeff_a(p0, p1, p2) * t + quad_bezier_derive_coeff_b(p0, p1)
}

fn quad_solve_critical_points(a: na::Vector2<f64>, b: na::Vector2<f64>, d: f64) -> (f64, f64) {
    let t1 = (-(a[0] * b[0] + a[1] + b[1])
        + ((a[0] * b[0] + a[1] * b[1]).powi(2)
            - (a[0].powi(2) + a[1].powi(2))
                * (b[0].powi(2) + b[1].powi(2)
                    - (d.powi(2) * (b[0] * a[1] - a[0] * b[1]).powi(2)).cbrt()))
        .sqrt())
        / (a[0].powi(2) + a[1].powi(2));

    let t2 = (-(a[0] * b[0] + a[1] + b[1])
        - ((a[0] * b[0] + a[1] * b[1]).powi(2)
            - (a[0].powi(2) + a[1].powi(2))
                * (b[0].powi(2) + b[1].powi(2)
                    - (d.powi(2) * (b[0] * a[1] - a[0] * b[1]).powi(2)).cbrt()))
        .sqrt())
        / (a[0].powi(2) + a[1].powi(2));

    (t1, t2)
}

// See 'Conversion between Cubic Bezier Curves and Catmull-Rom Splines'
pub fn gen_cubic_bezier_w_catmull_rom(
    first: &Element,
    second: &Element,
    third: &Element,
    forth: &Element,
) -> CubicBezier {
    // Tension factor (tau)
    let tension = 1.0;

    // Creating cubic bezier with catmull-rom
    let start = second.inputdata.pos();
    let cp1 =
        second.inputdata.pos() + (third.inputdata.pos() - first.inputdata.pos()) / (6.0 * tension);
    let cp2 =
        third.inputdata.pos() - (forth.inputdata.pos() - second.inputdata.pos()) / (6.0 * tension);
    let end = third.inputdata.pos();

    let cubic_bezier = CubicBezier {
        start,
        cp1,
        cp2,
        end,
    };

    cubic_bezier
}

pub fn gen_line(first: &Element, second: &Element, offset: na::Vector2<f64>) -> Option<Line> {
    let line = Line {
        start: first.inputdata.pos() + offset,
        end: second.inputdata.pos() + offset,
    };

    let start_end_len = (line.end - line.start).magnitude();

    // returns early to prevent NaN when calculating the vector norm.
    if start_end_len == 0.0 {
        return None;
    }

    Some(line)
}

pub fn split_quad_bezier_critical_points(
    quad_to_split: QuadBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
) -> Vec<QuadBezier> {
    let mut quads = Vec::new();

    let max_offset_dist = start_offset_dist.max(end_offset_dist);

    let coeff_a =
        quad_bezier_derive_coeff_a(quad_to_split.start, quad_to_split.cp, quad_to_split.end);
    let coeff_b = quad_bezier_derive_coeff_b(quad_to_split.start, quad_to_split.cp);

    // Calculate critical points (local curvature equals than offset witdh)
    let (mut t1, mut t2) = quad_solve_critical_points(coeff_a, coeff_b, max_offset_dist);

    if t2 < t1 {
        let tmp = t1;
        t1 = t2;
        t2 = tmp;
    }

    if t1 > 0.0 && t1 < 1.0 {
        let (t1_first, t1_second) = split_quad_bezier(quad_to_split, t1);
        quads.push(t1_first);

        if t2 > 0.0 && t2 < 1.0 {
            let (t2_first, t2_second) = split_quad_bezier(t1_second, t2);
            quads.push(t2_first);
            quads.push(t2_second);
        }
    }

    quads
}

// Split a quadratic bezier curve into two, interpolation value z: between 0.0 and 1.0
pub fn split_quad_bezier(quad_to_split: QuadBezier, z: f64) -> (QuadBezier, QuadBezier) {
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

pub fn split_cubic_bezier(cubic_bezier: CubicBezier, t: f64) -> (CubicBezier, CubicBezier) {
    let a0 = cubic_bezier.start;
    let a1 = cubic_bezier.cp1;
    let a2 = cubic_bezier.cp2;
    let a3 = cubic_bezier.end;

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

pub fn approx_cubic_with_quad(cubic_bezier: CubicBezier) -> QuadBezier {
    let start = cubic_bezier.start;
    let cp = cubic_bezier.cp1.lerp(&cubic_bezier.cp2, 0.5);
    let end = cubic_bezier.end;

    QuadBezier { start, cp, end }
}
