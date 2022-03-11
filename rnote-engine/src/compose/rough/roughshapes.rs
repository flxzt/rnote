use super::roughoptions::RoughOptions;
use rand::Rng;
use svg::node::element::path;

fn offset<R>(
    min: f64,
    max: f64,
    options: &RoughOptions,
    rng: &mut R,
    roughness_gain: Option<f64>,
) -> f64
where
    R: Rng + ?Sized,
{
    let roughness_gain = roughness_gain.unwrap_or(1.0);
    options.roughness * roughness_gain * (rng.gen_range(0.0..1.0) * (max - min) + min)
}

fn offset_opt<R>(x: f64, options: &RoughOptions, rng: &mut R, roughness_gain: Option<f64>) -> f64
where
    R: Rng + ?Sized,
{
    offset(-x, x, options, rng, roughness_gain)
}

pub(super) fn line<R>(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    move_to: bool,
    overlay: bool,
    options: &RoughOptions,
    rng: &mut R,
) -> Vec<path::Command>
where
    R: Rng + ?Sized,
{
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

    let diverge_point = 0.2 + rng.gen_range(0.0..1.0) * 0.2;

    let mid_disp_x = options.bowing * options.max_randomness_offset * (end[1] - start[1]) / 200.0;
    let mid_disp_y = options.bowing * options.max_randomness_offset * (start[0] - end[0]) / 200.0;
    let mid_disp_x = offset_opt(mid_disp_x, options, rng, Some(roughness_gain));
    let mid_disp_y = offset_opt(mid_disp_y, options, rng, Some(roughness_gain));

    let mut commands = Vec::new();

    if move_to {
        if overlay {
            let x = start[0]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(half_offset, options, rng, Some(roughness_gain))
                };
            let y = start[1]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(half_offset, options, rng, Some(roughness_gain))
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
                    offset_opt(offset, options, rng, Some(roughness_gain))
                };
            let y = start[1]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(offset, options, rng, Some(roughness_gain))
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
                offset_opt(half_offset, options, rng, Some(roughness_gain))
            };
        let y2 = end[1]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(half_offset, options, rng, Some(roughness_gain))
            };

        commands.push(path::Command::CubicCurve(
            path::Position::Absolute,
            path::Parameters::from((
                (
                    mid_disp_x
                        + start[0]
                        + (end[0] - start[0]) * diverge_point
                        + offset_opt(half_offset, options, rng, Some(roughness_gain)),
                    mid_disp_y
                        + start[1]
                        + (end[1] - start[1]) * diverge_point
                        + offset_opt(half_offset, options, rng, Some(roughness_gain)),
                ),
                (
                    mid_disp_x
                        + start[0]
                        + 2.0 * (end[0] - start[0]) * diverge_point
                        + offset_opt(half_offset, options, rng, Some(roughness_gain)),
                    mid_disp_y
                        + start[1]
                        + 2.0 * (end[1] - start[1]) * diverge_point
                        + offset_opt(half_offset, options, rng, Some(roughness_gain)),
                ),
                (x2, y2),
            )),
        ));
    } else {
        let x2 = end[0]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(offset, options, rng, Some(roughness_gain))
            };
        let y2 = end[1]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(offset, options, rng, Some(roughness_gain))
            };

        commands.push(path::Command::CubicCurve(
            path::Position::Absolute,
            path::Parameters::from((
                (
                    mid_disp_x
                        + start[0]
                        + (end[0] - start[0]) * diverge_point
                        + offset_opt(offset, options, rng, Some(roughness_gain)),
                    mid_disp_y
                        + start[1]
                        + (end[1] - start[1]) * diverge_point
                        + offset_opt(offset, options, rng, Some(roughness_gain)),
                ),
                (
                    mid_disp_x
                        + start[0]
                        + 2.0 * (end[0] - start[0]) * diverge_point
                        + offset_opt(offset, options, rng, Some(roughness_gain)),
                    mid_disp_y
                        + start[1]
                        + 2.0 * (end[1] - start[1]) * diverge_point
                        + offset_opt(offset, options, rng, Some(roughness_gain)),
                ),
                (x2, y2),
            )),
        ));
    }

    commands
}

pub(super) fn doubleline<R>(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &RoughOptions,
    rng: &mut R,
) -> Vec<path::Command>
where
    R: Rng + ?Sized,
{
    let mut commands = line(start, end, true, false, options, rng);

    let mut second_options = options.clone();
    second_options.seed = Some(rng.gen::<u64>());

    commands.append(&mut line(start, end, true, true, &second_options, rng));

    commands
}

pub(super) fn cubic_bezier<R>(
    start: na::Vector2<f64>,
    first: na::Vector2<f64>,
    second: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &RoughOptions,
    rng: &mut R,
) -> Vec<path::Command>
where
    R: Rng + ?Sized,
{
    let mut commands = Vec::new();

    let ros = [
        options.max_randomness_offset,
        options.max_randomness_offset + 0.3,
    ];

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
                    offset_opt(ros[0], options, rng, None),
                    offset_opt(ros[0], options, rng, None)
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
                end[0] + offset_opt(ros[i], options, rng, None),
                end[1] + offset_opt(ros[i], options, rng, None)
            ]
        };

        commands.push(path::Command::CubicCurve(
            path::Position::Absolute,
            path::Parameters::from((
                (
                    first[0] + offset_opt(ros[i], options, rng, None),
                    first[1] + offset_opt(ros[i], options, rng, None),
                ),
                (
                    second[0] + offset_opt(ros[i], options, rng, None),
                    second[1] + offset_opt(ros[i], options, rng, None),
                ),
                (end_[0], end_[1]),
            )),
        ));
    }

    commands
}

pub(super) fn fill_polygon<R>(
    points: Vec<na::Vector2<f64>>,
    _options: &RoughOptions,
    _rng: &mut R,
) -> Vec<path::Command>
where
    R: Rng + ?Sized,
{
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

pub(super) fn ellipse<R>(
    center: na::Vector2<f64>,
    radius_x: f64,
    radius_y: f64,
    options: &RoughOptions,
    rng: &mut R,
) -> EllipseResult
where
    R: Rng + ?Sized,
{
    let mut commands = Vec::new();

    // generate ellipse parameters
    let psq = (std::f64::consts::PI * 2.0 * ((radius_x).powi(2) + (radius_y).powi(2)).sqrt() / 2.0)
        .sqrt();
    let stepcount = options
        .curve_stepcount
        .max((options.curve_stepcount / (200.0_f64).sqrt()) * psq);

    let increment = (std::f64::consts::PI * 2.0) / stepcount;
    let curve_fitrandomness = 1.0 - options.curve_fitting;

    let radius_x = radius_x + offset_opt(radius_x * curve_fitrandomness, options, rng, None);
    let radius_y = radius_y + offset_opt(radius_y * curve_fitrandomness, options, rng, None);

    // creating ellipse
    let overlap_1 = increment
        * self::offset(
            0.1,
            self::offset(0.4, 1.0, options, rng, None),
            options,
            rng,
            None,
        );

    let (all_points_1, core_points_1) = compute_ellipse_points(
        increment, center, radius_x, radius_y, 1.0, overlap_1, options, rng,
    );

    commands.append(&mut curve(all_points_1, None, options, rng));

    if !options.disable_multistroke {
        let (all_points_2, _) = compute_ellipse_points(
            increment, center, radius_x, radius_y, 1.5, 0.0, options, rng,
        );

        commands.append(&mut curve(all_points_2, None, options, rng));
    }

    EllipseResult {
        estimated_points: core_points_1,
        commands,
    }
}

pub(super) fn curve<R>(
    points: Vec<na::Vector2<f64>>,
    close_point: Option<na::Vector2<f64>>,
    options: &RoughOptions,
    rng: &mut R,
) -> Vec<path::Command>
where
    R: Rng + ?Sized,
{
    let mut commands = Vec::new();
    let len = points.len();

    if len > 3 {
        let s = 1.0 - options.curve_tightness;

        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((points[1][0], points[1][1])),
        ));

        let mut i = 1;
        while i + 2 < len {
            let _b0 = points[i];
            let b1 = na::vector![
                points[i][0] + (s + points[i + 1][0] - s * points[i - 1][0]) / 6.0,
                points[i][1] + (s + points[i + 1][1] - s * points[i - 1][1]) / 6.0
            ];
            let b2 = na::vector![
                points[i + 1][0] + (s * points[i][0] - s * points[i + 2][0]) / 6.0,
                points[i + 1][1] + (s * points[i][1] - s * points[i + 2][1]) / 6.0
            ];
            let b3 = points[i + 1];

            /*             commands.push(path::Command::Move(
                path::Position::Absolute,
                path::Parameters::from((b0[0], b0[1])),
            )); */

            commands.push(path::Command::CubicCurve(
                path::Position::Absolute,
                path::Parameters::from(((b1[0], b1[1]), (b2[0], b2[1]), (b3[0], b3[1]))),
            ));

            i += 1;
        }
        if let Some(close_point) = close_point {
            if close_point.len() == 2 {
                commands.push(path::Command::Line(
                    path::Position::Absolute,
                    path::Parameters::from((
                        close_point[0]
                            + offset_opt(options.max_randomness_offset, options, rng, None),
                        close_point[1]
                            + offset_opt(options.max_randomness_offset, options, rng, None),
                    )),
                ));
            }
        }
    } else if len == 3 {
        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((points[1][0], points[1][1])),
        ));
        commands.push(path::Command::CubicCurve(
            path::Position::Absolute,
            path::Parameters::from((
                (points[1][0], points[1][1]),
                (points[2][0], points[2][1]),
                (points[2][0], points[2][1]),
            )),
        ));
    } else if len == 2 {
        commands.append(&mut doubleline(points[0], points[1], options, rng));
    }

    commands
}

#[derive(Debug, Clone)]
pub struct EllipseResult {
    pub estimated_points: Vec<na::Vector2<f64>>,
    pub commands: Vec<path::Command>,
}

// Returns (all_points, core_points)
pub(super) fn compute_ellipse_points<R>(
    increment: f64,
    center: na::Vector2<f64>,
    radius_x: f64,
    radius_y: f64,
    offset: f64,
    overlap: f64,
    options: &RoughOptions,
    rng: &mut R,
) -> (Vec<na::Vector2<f64>>, Vec<na::Vector2<f64>>)
where
    R: Rng + ?Sized,
{
    let mut core_points = Vec::new();
    let mut all_points = Vec::new();

    let rad_offset = offset_opt(0.5, options, rng, None) - std::f64::consts::PI / 2.0;
    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + 0.9 * radius_x * (rad_offset - increment),
        offset_opt(offset, options, rng, None)
            + center[1]
            + 0.9 * radius_y * (rad_offset - increment)
    ]);

    let mut angle = rad_offset;
    while angle < (std::f64::consts::PI * 2.0 + rad_offset - 0.01) {
        let point = na::vector![
            offset_opt(offset, options, rng, None) + center[0] + radius_x * angle.cos(),
            offset_opt(offset, options, rng, None) + center[1] + radius_y * angle.sin()
        ];

        all_points.push(point);
        core_points.push(point);

        angle += increment;
    }

    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + radius_x * (rad_offset + std::f64::consts::PI * 2.0 + overlap * 0.5).cos(),
        offset_opt(offset, options, rng, None)
            + center[1]
            + radius_y * (rad_offset + std::f64::consts::PI * 2.0 + overlap * 0.5).sin()
    ]);
    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + 0.98 * radius_x * (rad_offset + overlap).cos(),
        offset_opt(offset, options, rng, None)
            + center[1]
            + 0.98 * radius_y * (rad_offset + overlap).sin()
    ]);
    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + 0.9 * radius_x * (rad_offset + overlap * 0.5).cos(),
        offset_opt(offset, options, rng, None)
            + center[1]
            + 0.9 * radius_y * (rad_offset + overlap * 0.5).sin()
    ]);

    (all_points, core_points)
}
