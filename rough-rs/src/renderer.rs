use crate::{math, options::Options};
use svg::node::element::{self};

fn offset(min: f64, max: f64, options: &Options, _roughness_gain: f64) -> f64 {
    let roughness_gain = 1.0;
    options.roughness * roughness_gain * (math::random_f64_0to1(options.seed) * (max - min) + min)
}

fn offset_opt(x: f64, options: &Options, _roughness_gain: f64) -> f64 {
    let roughness_gain = 1.0;
    offset(-x, x, options, roughness_gain)
}

pub(crate) fn line(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &Options,
    move_to: bool,
    overlay: bool,
) -> element::Path {
    let len = (end - start).magnitude();

    let roughness_gain = if len < 200.0 {
        1.0
    } else if len > 500.0 {
        0.4
    } else {
        -0.0016668 * len + 1.233334
    };

    let mut offset = options.max_randomness_offset;
    if offset * offset * 100.0 > len.sqrt() {
        offset = len / 10.0;
    };
    let half_offset = offset / 2.0;

    let diverge_point = 0.2 + math::random_f64_0to1(options.seed) * 0.2;

    let mid_disp_x = options.bowing * options.max_randomness_offset * (end[1] - start[1]) / 200.0;
    let mid_disp_y = options.bowing * options.max_randomness_offset * (start[0] - end[0]) / 200.0;
    let mid_disp_x = offset_opt(mid_disp_x, options, roughness_gain);
    let mid_disp_y = offset_opt(mid_disp_y, options, roughness_gain);

    let random_half = || offset_opt(half_offset, options, roughness_gain);
    let random_full = || offset_opt(offset, options, roughness_gain);

    let mut data = element::path::Data::new();

    if move_to {
        if overlay {
            let x = start[0]
                + if options.preserve_vertices {
                    0.0
                } else {
                    random_half()
                };
            let y = start[1]
                + if options.preserve_vertices {
                    0.0
                } else {
                    random_half()
                };

            data = data.move_to((x, y));
        } else {
            let x = start[0]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(offset, options, roughness_gain)
                };
            let y = start[1]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(offset, options, roughness_gain)
                };

            data = data.move_to((x, y));
        }
    }

    if overlay {
        let x2 = end[0]
            + if options.preserve_vertices {
                0.0
            } else {
                random_half()
            };
        let y2 = end[1]
            + if options.preserve_vertices {
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
            + if options.preserve_vertices {
                0.0
            } else {
                random_full()
            };
        let y2 = end[1]
            + if options.preserve_vertices {
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

    element::Path::new().set("d", data)
}

pub(crate) fn cubic_bezier(
    start: na::Vector2<f64>,
    first: na::Vector2<f64>,
    second: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &Options,
) -> element::Path {
    let mut data = element::path::Data::new();

    let ros = [
        options.max_randomness_offset,
        options.max_randomness_offset + 0.3,
    ];
    let roughness_gain = 1.0;

    let iterations = if options.disable_multistroke {
        1_usize
    } else {
        2_usize
    };
    for i in 0..iterations {
        if i == 0 {
            data = data.move_to((start[0], start[1]));
        } else {
            let delta = if options.preserve_vertices {
                na::vector![0.0, 0.0]
            } else {
                na::vector![
                    offset_opt(ros[0], options, roughness_gain),
                    offset_opt(ros[0], options, roughness_gain)
                ]
            };

            data = data.move_to((start[0] + delta[0], start[1] + delta[1]));
        }

        let end_ = if options.preserve_vertices {
            na::vector![end[0], end[1]]
        } else {
            na::vector![
                end[0] + offset_opt(ros[i], options, roughness_gain),
                end[1] + offset_opt(ros[i], options, roughness_gain)
            ]
        };

        data = data.cubic_curve_to((
            (
                first[0] + offset_opt(ros[i], options, roughness_gain),
                first[1] + offset_opt(ros[i], options, roughness_gain),
            ),
            (
                second[0] + offset_opt(ros[i], options, roughness_gain),
                second[1] + offset_opt(ros[i], options, roughness_gain),
            ),
            (end_[0], end_[1]),
        ))
    }

    element::Path::new().set("d", data)
}
