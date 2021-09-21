use crate::{math, options::Options};
use svg::node::element::{self};

/// A line from the start to stop coordinates, in absolute values.
fn line(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &Options,
    move_: bool,
    overlay: bool,
) -> element::path::Data {
    let len = (end - start).magnitude();

    let roughness_gain = if len < 200.0 {
        1.0
    } else if len > 500.0 {
        0.4
    } else {
        -0.0016668 * len + 1.233334
    };

    let mut offset = options.max_randomness_offset.unwrap_or(0.0);
    if offset * offset * 100.0 > len.sqrt() {
        offset = len / 10.0;
    };
    let half_offset = offset / 2.0;

    let diverge_point = 0.2 + math::random_f64_0to1() * 0.2;

    let mid_disp_x =
        options.bowing.unwrap() * options.max_randomness_offset.unwrap() * (end[1] - start[1])
            / 200.0;
    let mid_disp_y =
        options.bowing.unwrap() * options.max_randomness_offset.unwrap() * (end[0] - start[0])
            / 200.0;
    let mid_disp_x = offset_opt(mid_disp_x, options, roughness_gain);
    let mid_disp_y = offset_opt(mid_disp_y, options, roughness_gain);

    let random_half = || offset_opt(half_offset, options, roughness_gain);
    let random_full = || offset_opt(offset, options, roughness_gain);

    let mut data = element::path::Data::new();

    if move_ {
        if overlay {
            let x = start[0]
                + if options.preserve_vertices.unwrap() {
                    0.0
                } else {
                    random_half()
                };
            let y = start[1]
                + if options.preserve_vertices.unwrap() {
                    0.0
                } else {
                    random_half()
                };

            data = data.move_to((x, y));
        } else {
            let x = start[0]
                + if options.preserve_vertices.unwrap() {
                    0.0
                } else {
                    offset_opt(offset, options, roughness_gain)
                };
            let y = start[1]
                + if options.preserve_vertices.unwrap() {
                    0.0
                } else {
                    offset_opt(offset, options, roughness_gain)
                };

            data = data.move_to((x, y));
        }
    }

    if overlay {
        let x2 = end[0]
            + if options.preserve_vertices.unwrap() {
                0.0
            } else {
                random_half()
            };
        let y2 = end[1]
            + if options.preserve_vertices.unwrap() {
                0.0
            } else {
                random_half()
            };

        data = data.cubic_curve_to((
            (
                mid_disp_x + start[0] + (end[0] - start[0]) * diverge_point + random_half(),
                mid_disp_y + start[1] + (end[1] - start[1]) * diverge_point + random_half(),
            ),
            (
                mid_disp_x + start[0] + 2.0 * (end[0] - start[0]) * diverge_point + random_half(),
                mid_disp_y + start[1] + 2.0 * (end[1] - start[1]) * diverge_point + random_half(),
            ),
            (x2, y2),
        ));
    } else {
        let x2 = end[0]
            + if options.preserve_vertices.unwrap() {
                0.0
            } else {
                random_full()
            };
        let y2 = end[1]
            + if options.preserve_vertices.unwrap() {
                0.0
            } else {
                random_full()
            };

        data = data.cubic_curve_to((
            (
                mid_disp_x + start[0] + (end[0] - start[0]) * diverge_point + random_full(),
                mid_disp_y + start[1] + (end[1] - start[1]) * diverge_point + random_full(),
            ),
            (
                mid_disp_x + start[0] + 2.0 * (end[0] - start[0]) * diverge_point + random_full(),
                mid_disp_y + start[1] + 2.0 * (end[1] - start[1]) * diverge_point + random_full(),
            ),
            (x2, y2),
        ));
    }

    data
}

pub(crate) fn double_line(start: na::Vector2<f64>, end: na::Vector2<f64>, options: &Options, _filling: bool) -> element::path::Data {
    let filling = false;

    let single_stroke = if filling { options.disable_multistroke_fill.unwrap() } else { options.disable_multistroke.unwrap() };

    let first_stroke = line(start, end, options, true, false);
    if single_stroke {
        return first_stroke;
    }
    let _second_stroke = line(start, end, options, true, false);

    // TODO ADD SECOND STROKE
    return first_stroke;
}

fn offset(min: f64, max: f64, options: &Options, roughness_gain: f64) -> f64 {
    options.roughness.unwrap() * roughness_gain * (math::random_f64_0to1() * (max - min) + min)
}

fn offset_opt(x: f64, options: &Options, roughness_gain: f64) -> f64 {
    offset(-x, x, options, roughness_gain)
}
