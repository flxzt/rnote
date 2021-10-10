use crate::{options::Options, utils};
use svg::node::element::path;

fn offset(min: f64, max: f64, options: &mut Options, roughness_gain: f64) -> f64 {
    options.roughness * roughness_gain * (utils::random_next(options) * (max - min) + min)
}

fn offset_opt(x: f64, options: &mut Options, roughness_gain: f64) -> f64 {
    offset(-x, x, options, roughness_gain)
}

pub(crate) fn line(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &mut Options,
    move_to: bool,
    overlay: bool,
) -> Vec<path::Command> {
    let len = (end - start).magnitude();

    let roughness_gain = if len < 200.0 {
        1.0
    } else if len > 500.0 {
        0.4
    } else {
        -0.0016668 * len + 1.233334
    };

    let mut offset = options.max_randomness_offset;
    if offset * offset * 100.0 > (len * len) {
        offset = len / 10.0;
    };
    let half_offset = offset * 0.5;

    let diverge_point = 0.2 + utils::random_f64_0to1(options.seed) * 0.2;

    let mid_disp_x = options.bowing * options.max_randomness_offset * (end[1] - start[1]) / 200.0;
    let mid_disp_y = options.bowing * options.max_randomness_offset * (start[0] - end[0]) / 200.0;
    let mid_disp_x = offset_opt(mid_disp_x, options, roughness_gain);
    let mid_disp_y = offset_opt(mid_disp_y, options, roughness_gain);

    let mut commands = Vec::new();

    if move_to {
        if overlay {
            let x = start[0]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(half_offset, options, roughness_gain)
                };
            let y = start[1]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(half_offset, options, roughness_gain)
                };

            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((x, y)),
            ));
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

            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((x, y)),
            ));
        }
    }

    if overlay {
        let x2 = end[0]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(half_offset, options, roughness_gain)
            };
        let y2 = end[1]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(half_offset, options, roughness_gain)
            };

        commands.push(path::Command::CubicCurve(
            path::Position::Absolute,
            path::Parameters::from((
                (
                    mid_disp_x
                        + start[0]
                        + (end[0] - start[0]) * diverge_point
                        + offset_opt(half_offset, options, roughness_gain),
                    mid_disp_y
                        + start[1]
                        + (end[1] - start[1]) * diverge_point
                        + offset_opt(half_offset, options, roughness_gain),
                ),
                (
                    mid_disp_x
                        + start[0]
                        + 2.0 * (end[0] - start[0]) * diverge_point
                        + offset_opt(half_offset, options, roughness_gain),
                    mid_disp_y
                        + start[1]
                        + 2.0 * (end[1] - start[1]) * diverge_point
                        + offset_opt(half_offset, options, roughness_gain),
                ),
                (x2, y2),
            )),
        ));
    } else {
        let x2 = end[0]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(offset, options, roughness_gain)
            };
        let y2 = end[1]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(offset, options, roughness_gain)
            };

        commands.push(path::Command::CubicCurve(
            path::Position::Absolute,
            path::Parameters::from((
                (
                    mid_disp_x
                        + start[0]
                        + (end[0] - start[0]) * diverge_point
                        + offset_opt(offset, options, roughness_gain),
                    mid_disp_y
                        + start[1]
                        + (end[1] - start[1]) * diverge_point
                        + offset_opt(offset, options, roughness_gain),
                ),
                (
                    mid_disp_x
                        + start[0]
                        + 2.0 * (end[0] - start[0]) * diverge_point
                        + offset_opt(offset, options, roughness_gain),
                    mid_disp_y
                        + start[1]
                        + 2.0 * (end[1] - start[1]) * diverge_point
                        + offset_opt(offset, options, roughness_gain),
                ),
                (x2, y2),
            )),
        ));
    }

    commands
}

pub fn doubleline(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &mut Options,
) -> Vec<path::Command> {
    let mut commands = line(start, end, options, true, false);

    let mut second_options = options.clone();
    if let Some(seed) = options.seed {
        second_options.seed = Some(utils::random_u64_full(Some(seed)));
        //second_options.seed = None;
    };

    commands.append(&mut line(start, end, &mut second_options, true, true));

    commands
}

pub fn cubic_bezier(
    start: na::Vector2<f64>,
    first: na::Vector2<f64>,
    second: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &mut Options,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

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
            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((start[0], start[1])),
            ));
        } else {
            let delta = if options.preserve_vertices {
                na::vector![0.0, 0.0]
            } else {
                na::vector![
                    offset_opt(ros[0], options, roughness_gain),
                    offset_opt(ros[0], options, roughness_gain)
                ]
            };

            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((start[0] + delta[0], start[1] + delta[1])),
            ));
        }

        let end_ = if options.preserve_vertices {
            na::vector![end[0], end[1]]
        } else {
            na::vector![
                end[0] + offset_opt(ros[i], options, roughness_gain),
                end[1] + offset_opt(ros[i], options, roughness_gain)
            ]
        };

        commands.push(path::Command::CubicCurve(
            path::Position::Absolute,
            path::Parameters::from((
                (
                    first[0] + offset_opt(ros[i], options, roughness_gain),
                    first[1] + offset_opt(ros[i], options, roughness_gain),
                ),
                (
                    second[0] + offset_opt(ros[i], options, roughness_gain),
                    second[1] + offset_opt(ros[i], options, roughness_gain),
                ),
                (end_[0], end_[1]),
            )),
        ));
    }

    commands
}

pub fn fill_polygon(points: Vec<na::Vector2<f64>>, _options: &mut Options) -> Vec<path::Command> {
    let mut commands = Vec::new();

    for (i, point) in points.iter().enumerate() {
        if i == 0 {
            commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((point[0], point[1])),
            ));
        } else {
            commands.push(path::Command::Line(
                path::Position::Absolute,
                path::Parameters::from((point[0], point[1])),
            ));
        }
    }
    commands.push(path::Command::Close);

    commands
}
