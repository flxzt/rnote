use super::geometry::Vector2Helpers;
use super::{color::Color, curves, shapes};

use serde::{Deserialize, Serialize};
use svg::node::element::{self, path};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "smoothoptions")]
pub struct SmoothOptions {
    /// An optional seed to generate reproducable strokes
    #[serde(rename = "seed")]
    pub seed: Option<u64>,
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "stroke_color")]
    pub stroke_color: Option<Color>,
    #[serde(rename = "fill_color")]
    pub fill_color: Option<Color>,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        Self {
            seed: None,
            width: Self::WIDTH_DEFAULT,
            stroke_color: Some(Self::COLOR_DEFAULT),
            fill_color: None,
        }
    }
}

impl SmoothOptions {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 1.0;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;
    /// The default color
    pub const COLOR_DEFAULT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}

pub fn compose_line(
    line: curves::Line,
    move_start: bool,
    _options: &SmoothOptions,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    if move_start {
        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((line.start[0], line.start[1])),
        ));
    }
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((line.end[0], line.end[1])),
    ));

    commands
}

pub fn compose_line_offsetted(
    line: curves::Line,
    start_offset_dist: f64,
    end_offset_dist: f64,
    move_start: bool,
    _options: &SmoothOptions,
) -> Vec<path::Command> {
    let direction_unit_norm = (line.end - line.start).unit_norm();
    let start_offset = direction_unit_norm * start_offset_dist;
    let end_offset = direction_unit_norm * end_offset_dist;

    let mut commands = Vec::new();
    if move_start {
        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((
                line.start[0] + start_offset[0],
                line.start[1] + start_offset[1],
            )),
        ));
    }
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((line.end[0] + end_offset[0], line.end[1] + end_offset[1])),
    ));

    commands
}

pub fn compose_line_variable_width(
    line: curves::Line,
    width_start: f64,
    width_end: f64,
    move_start: bool,
    options: &SmoothOptions,
) -> Vec<path::Command> {
    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let line_reverse = curves::Line {
        start: line.end,
        end: line.start,
    };
    let direction_unit_norm = (line.end - line.start).unit_norm();

    let mut commands = Vec::new();
    commands.append(&mut compose_line_offsetted(
        line,
        start_offset_dist,
        end_offset_dist,
        move_start,
        options,
    ));
    commands.push(path::Command::EllipticalArc(
        path::Position::Absolute,
        path::Parameters::from((
            end_offset_dist,
            end_offset_dist,
            0.0,
            0.0,
            0.0,
            (line.end + direction_unit_norm * (-end_offset_dist))[0],
            (line.end + direction_unit_norm * (-end_offset_dist))[1],
        )),
    ));
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (line.end + direction_unit_norm * (-end_offset_dist))[0],
            (line.end + direction_unit_norm * (-end_offset_dist))[1],
        )),
    ));
    commands.append(&mut compose_line_offsetted(
        line_reverse,
        end_offset_dist,
        start_offset_dist,
        false,
        options,
    ));
    commands.push(path::Command::EllipticalArc(
        path::Position::Absolute,
        path::Parameters::from((
            start_offset_dist,
            start_offset_dist,
            0.0,
            0.0,
            0.0,
            (line_reverse.end + direction_unit_norm * (start_offset_dist))[0],
            (line_reverse.end + direction_unit_norm * (start_offset_dist))[1],
        )),
    ));
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (line_reverse.end + direction_unit_norm * (start_offset_dist))[0],
            (line_reverse.end + direction_unit_norm * (start_offset_dist))[1],
        )),
    ));

    commands
}

pub fn compose_quadbez(
    quadbez: curves::QuadBezier,
    move_start: bool,
    _options: &SmoothOptions,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    if move_start {
        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((quadbez.start[0], quadbez.start[1])),
        ));
    }
    commands.push(path::Command::QuadraticCurve(
        path::Position::Absolute,
        path::Parameters::from((
            (quadbez.cp[0], quadbez.cp[1]),
            (quadbez.end[0], quadbez.end[1]),
        )),
    ));

    commands
}

pub fn compose_quadbez_offsetted(
    quadbez: curves::QuadBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    move_start: bool,
    _options: &SmoothOptions,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let start_unit_norm = (quadbez.cp - quadbez.start).unit_norm();
    let end_unit_norm = (quadbez.end - quadbez.cp).unit_norm();

    let start_offset = start_unit_norm * start_offset_dist;
    let end_offset = end_unit_norm * end_offset_dist;

    let added_unit_norms = start_unit_norm + end_unit_norm;

    // TODO: find better algo for the offset distance of the control point than the average between start and end offset
    let cp_offset_dist = (start_offset_dist + end_offset_dist) / 2.0;

    let cp_offset =
        (2.0 * cp_offset_dist * added_unit_norms) / added_unit_norms.dot(&added_unit_norms);

    if move_start {
        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((
                quadbez.start[0] + start_offset[0],
                quadbez.start[1] + start_offset[1],
            )),
        ));
    }
    commands.push(path::Command::QuadraticCurve(
        path::Position::Absolute,
        path::Parameters::from((
            (quadbez.cp[0] + cp_offset[0], quadbez.cp[1] + cp_offset[1]),
            (
                quadbez.end[0] + end_offset[0],
                quadbez.end[1] + end_offset[1],
            ),
        )),
    ));

    commands
}

/// Offsetted quad bezier approximation, see "precise offsetting of quadratic bezier curves"
pub fn compose_quadbez_offsetted_w_subdivision(
    quadbez: curves::QuadBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    move_start: bool,
    options: &SmoothOptions,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let (splitted_quads, split_t1, split_t2) = curves::split_offsetted_quadbez_critical_points(
        quadbez,
        start_offset_dist,
        end_offset_dist,
    );

    match (split_t1, split_t2) {
        (Some(split_t1), Some(split_t2)) => {
            let offset_dist_t1 = curves::quadbez_calc_offset_dist_at_t(
                quadbez,
                start_offset_dist,
                end_offset_dist,
                split_t1,
            );
            let offset_dist_t2 = curves::quadbez_calc_offset_dist_at_t(
                quadbez,
                start_offset_dist,
                end_offset_dist,
                split_t2,
            );

            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[0],
                start_offset_dist,
                offset_dist_t1,
                move_start,
                options,
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[1],
                offset_dist_t1,
                offset_dist_t2,
                false,
                options,
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[2],
                offset_dist_t2,
                end_offset_dist,
                false,
                options,
            ));
        }
        (Some(split_t1), None) => {
            let offset_dist_t1 = curves::quadbez_calc_offset_dist_at_t(
                quadbez,
                start_offset_dist,
                end_offset_dist,
                split_t1,
            );
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[0],
                start_offset_dist,
                offset_dist_t1,
                move_start,
                options,
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[1],
                offset_dist_t1,
                end_offset_dist,
                false,
                options,
            ));
        }
        (None, Some(split_t2)) => {
            let offset_dist_t2 = curves::quadbez_calc_offset_dist_at_t(
                quadbez,
                start_offset_dist,
                end_offset_dist,
                split_t2,
            );
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[0],
                start_offset_dist,
                offset_dist_t2,
                move_start,
                options,
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[1],
                offset_dist_t2,
                end_offset_dist,
                false,
                options,
            ));
        }
        (None, None) => {
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[0],
                start_offset_dist,
                end_offset_dist,
                move_start,
                options,
            ));
        }
    }

    commands
}

pub fn compose_quadbez_variable_width(
    quadbez: curves::QuadBezier,
    width_start: f64,
    width_end: f64,
    move_start: bool,
    options: &SmoothOptions,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let quadbez_reverse = curves::QuadBezier {
        start: quadbez.end,
        cp: quadbez.cp,
        end: quadbez.start,
    };

    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let start_unit_norm = (quadbez.cp - quadbez.start).unit_norm();
    let end_unit_norm = (quadbez.end - quadbez.cp).unit_norm();

    let start_offset = start_unit_norm * start_offset_dist;
    let end_offset = end_unit_norm * end_offset_dist;

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        quadbez,
        start_offset_dist,
        end_offset_dist,
        move_start,
        options,
    ));
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from(((quadbez.end - end_offset)[0], (quadbez.end - end_offset)[1])),
    ));

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        quadbez_reverse,
        end_offset_dist,
        start_offset_dist,
        false,
        options,
    ));
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (quadbez.start + start_offset)[0],
            (quadbez.start + start_offset)[1],
        )),
    ));

    commands
}

pub fn compose_cubbez(
    cubbez: curves::CubicBezier,
    move_start: bool,
    _options: &SmoothOptions,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    if move_start {
        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((cubbez.start[0], cubbez.start[1])),
        ));
    }
    commands.push(path::Command::CubicCurve(
        path::Position::Absolute,
        path::Parameters::from((
            (cubbez.cp1[0], cubbez.cp1[1]),
            (cubbez.cp2[0], cubbez.cp2[1]),
            (cubbez.end[0], cubbez.end[1]),
        )),
    ));

    commands
}

pub fn compose_cubbez_offsetted(
    cubbez: curves::CubicBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    move_start: bool,
    options: &SmoothOptions,
) -> Vec<path::Command> {
    let t = 0.5;
    let mid_offset_dist = start_offset_dist + (end_offset_dist - start_offset_dist) * t;

    let (first_cubic, second_cubic) = curves::split_cubbez(cubbez, t);
    let first_quad = curves::approx_cubbez_with_quadbez(first_cubic);
    let second_quad = curves::approx_cubbez_with_quadbez(second_cubic);

    let mut commands = Vec::new();

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        first_quad,
        start_offset_dist,
        mid_offset_dist,
        move_start,
        options,
    ));

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        second_quad,
        mid_offset_dist,
        end_offset_dist,
        false,
        options,
    ));

    commands
}

pub fn compose_cubbez_variable_width(
    cubbez: curves::CubicBezier,
    width_start: f64,
    width_end: f64,
    move_start: bool,
    options: &SmoothOptions,
) -> Vec<path::Command> {
    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let start_unit_norm = (cubbez.cp1 - cubbez.start).unit_norm();
    let end_unit_norm = (cubbez.end - cubbez.cp2).unit_norm();

    let start_offset = start_unit_norm * start_offset_dist;
    let end_offset = end_unit_norm * end_offset_dist;

    let cubbez_reverse = curves::CubicBezier {
        start: cubbez.end,
        cp1: cubbez.cp2,
        cp2: cubbez.cp1,
        end: cubbez.start,
    };

    // if the angle of the two offsets is > 90deg, calculating the norms went wrong, so reverse them.
    let angle = start_offset.angle(&end_offset).to_degrees();
    let angle_greater_90 = angle < -90.0 && angle > 90.0;

    let mut commands = compose_cubbez_offsetted(
        cubbez,
        start_offset_dist,
        end_offset_dist,
        move_start,
        options,
    );

    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from(((cubbez.end - end_offset)[0], (cubbez.end - end_offset)[1])),
    ));

    // If angle > 90.0 degrees, reverse the cubic_bezier vector (using the original cubic_bezier, but with offsets of the reversed)
    if angle_greater_90 {
        commands.append(&mut compose_cubbez_offsetted(
            cubbez,
            -end_offset_dist,
            -start_offset_dist,
            false,
            options,
        ));
    } else {
        commands.append(&mut compose_cubbez_offsetted(
            cubbez_reverse,
            end_offset_dist,
            start_offset_dist,
            false,
            options,
        ));
    }
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (cubbez.start + start_offset)[0],
            (cubbez.start + start_offset)[1],
        )),
    ));

    commands
}

pub fn compose_rectangle(
    rectangle: shapes::Rectangle,
    options: &SmoothOptions,
) -> element::Element {
    let color = if let Some(color) = options.stroke_color {
        color.to_css_color()
    } else {
        String::from("none")
    };
    let fill = if let Some(fill) = options.fill_color {
        fill.to_css_color()
    } else {
        String::from("none")
    };

    let (mins, maxs) = (-rectangle.cuboid.half_extents).mins_maxs(&rectangle.cuboid.half_extents);

    let transform_string = rectangle.transform.to_svg_transform_attr_str();

    svg::node::element::Rectangle::new()
        .set("transform", transform_string)
        .set("x", mins[0])
        .set("y", mins[1])
        .set("width", maxs[0] - mins[0])
        .set("height", maxs[1] - mins[1])
        .set("stroke", color)
        .set("stroke-width", options.width)
        .set("fill", fill)
        .into()
}

pub fn compose_ellipse(ellipse: shapes::Ellipse, options: &SmoothOptions) -> element::Element {
    let color = if let Some(color) = options.stroke_color {
        color.to_css_color()
    } else {
        String::from("none")
    };
    let fill = if let Some(fill) = options.fill_color {
        fill.to_css_color()
    } else {
        String::from("none")
    };

    let transform_string = ellipse.transform.to_svg_transform_attr_str();

    svg::node::element::Ellipse::new()
        .set("transform", transform_string)
        .set("cx", 0_f64)
        .set("cy", 0_f64)
        .set("rx", ellipse.radii[0])
        .set("ry", ellipse.radii[1])
        .set("stroke", color)
        .set("stroke-width", options.width)
        .set("fill", fill)
        .into()
}
