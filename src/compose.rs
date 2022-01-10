use gtk4::{gio, glib};
use svg::node::element::path;

use super::curves;

const XML_HEADER_REGEX: &str = r#"<\?xml[^\?>]*\?>"#;
const SVG_ROOT_REGEX: &str = r#"<svg[^>]*>|<[^/svg]*/svg>"#;

#[allow(dead_code)]
pub fn check_xml_header(svg: &str) -> bool {
    let re = regex::Regex::new(XML_HEADER_REGEX).unwrap();
    re.is_match(svg)
}

#[allow(dead_code)]
pub fn add_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(XML_HEADER_REGEX).unwrap();
    if !re.is_match(svg) {
        String::from(r#"<?xml version="1.0" standalone="no"?>"#) + "\n" + svg
    } else {
        String::from(svg)
    }
}

#[allow(dead_code)]
pub fn remove_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(XML_HEADER_REGEX).unwrap();
    String::from(re.replace_all(svg, ""))
}

#[allow(dead_code)]
pub fn check_svg_root(svg: &str) -> bool {
    let re = regex::Regex::new(SVG_ROOT_REGEX).unwrap();
    re.is_match(svg)
}

pub fn wrap_svg_root(
    data: &str,
    bounds: Option<p2d::bounding_volume::AABB>,
    viewbox: Option<p2d::bounding_volume::AABB>,
    xml_header: bool,
    preserve_aspectratio: bool,
) -> String {
    const SVG_WRAP_TEMPL_STR: &str = r#"
<svg
  x="{{x}}"
  y="{{y}}"
  width="{{width}}"
  height="{{height}}"
  {{viewbox}}
  preserveAspectRatio="{{preserve_aspectratio}}"
  xmlns="http://www.w3.org/2000/svg"
  xmlns:svg="http://www.w3.org/2000/svg"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  >
  {{data}}
</svg>
"#;
    let mut cx = tera::Context::new();

    let (x, y, width, height) = if let Some(bounds) = bounds {
        let x = format!("{:.3}", bounds.mins[0]);
        let y = format!("{:.3}", bounds.mins[1]);
        let width = format!("{:.3}", bounds.extents()[0]);
        let height = format!("{:.3}", bounds.extents()[1]);

        (x, y, width, height)
    } else {
        (
            String::from("0"),
            String::from("0"),
            String::from("100%"),
            String::from("100%"),
        )
    };

    let viewbox = if let Some(viewbox) = viewbox {
        format!(
            "viewBox=\"{:.3} {:.3} {:.3} {:.3}\"",
            viewbox.mins[0],
            viewbox.mins[1],
            viewbox.extents()[0],
            viewbox.extents()[1]
        )
    } else {
        String::from("")
    };
    let preserve_aspectratio = if preserve_aspectratio {
        String::from("xMidyMid")
    } else {
        String::from("none")
    };

    cx.insert("xml_header", &xml_header);
    cx.insert("data", data);
    cx.insert("x", &x);
    cx.insert("y", &y);
    cx.insert("width", &width);
    cx.insert("height", &height);
    cx.insert("viewbox", &viewbox);
    cx.insert("preserve_aspectratio", &preserve_aspectratio);

    tera::Tera::one_off(SVG_WRAP_TEMPL_STR, &cx, false).expect("failed to create svg from template")
}

#[allow(dead_code)]
pub fn strip_svg_root(svg: &str) -> String {
    let re = regex::Regex::new(SVG_ROOT_REGEX).unwrap();
    String::from(re.replace_all(svg, ""))
}

/// patterns are rendered rather slow, so this should be used carefully!
pub fn wrap_svg_pattern(data: &str, id: &str, bounds: p2d::bounding_volume::AABB) -> String {
    const SVG_PATTERN_TEMPL_STR: &str = r#"
<defs>
    <pattern
        id="{{id}}"
        x="{{x}}"
        y="{{y}}"
        width="{{width}}"
        height="{{height}}"
        patternUnits="userSpaceOnUse"
        >
        {{data}}
    </pattern>
</defs>
"#;
    let mut cx = tera::Context::new();
    let x = format!("{:3}", bounds.mins[0]);
    let y = format!("{:3}", bounds.mins[1]);
    let width = format!("{:3}", bounds.extents()[0]);
    let height = format!("{:3}", bounds.extents()[1]);
    cx.insert("id", &id);
    cx.insert("x", &x);
    cx.insert("y", &y);
    cx.insert("width", &width);
    cx.insert("height", &height);
    cx.insert("data", &data);

    tera::Tera::one_off(SVG_PATTERN_TEMPL_STR, &cx, false)
        .expect("failed to create svg from template")
}

/// wraps the data in a group, and translates and scales them with the transform attribute
pub fn wrap_svg_group(
    data: &str,
    offset: na::Vector2<f64>,
    scalevector: na::Vector2<f64>,
) -> String {
    const SVG_GROUP_TEMPL_STR: &str = r#"
<g transform="
  translate({{translate_x}} {{translate_y}})
  scale({{scale_x}} {{scale_y}})
  "
>
  {{data}}
</g>
"#;
    let mut cx = tera::Context::new();
    let translate_x = format!("{:3}", offset[0]);
    let translate_y = format!("{:3}", offset[1]);
    let scale_x = format!("{:3}", scalevector[0]);
    let scale_y = format!("{:3}", scalevector[1]);
    cx.insert("translate_x", &translate_x);
    cx.insert("translate_y", &translate_y);
    cx.insert("scale_x", &scale_x);
    cx.insert("scale_y", &scale_y);
    cx.insert("data", data);

    tera::Tera::one_off(SVG_GROUP_TEMPL_STR, &cx, false)
        .expect("failed to create svg from template")
}

pub fn svg_intrinsic_size(svg: &str) -> Option<na::Vector2<f64>> {
    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg.as_bytes()));
    if let Ok(handle) = librsvg::Loader::new()
        .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(&stream, None, None)
    {
        let renderer = librsvg::CairoRenderer::new(&handle);

        let intrinsic_size = if let Some(size) = renderer.intrinsic_size_in_pixels() {
            Some(na::vector![size.0, size.1])
        } else {
            log::debug!("intrinsic_size_in_pixels() returns None in svg_intrinsic_size()");
            None
        };

        intrinsic_size
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn compose_line(line: curves::Line, move_start: bool) -> Vec<path::Command> {
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
) -> Vec<path::Command> {
    let direction_unit_norm = curves::vector2_unit_norm(line.end - line.start);
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
) -> Vec<path::Command> {
    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let line_reverse = curves::Line {
        start: line.end,
        end: line.start,
    };
    let direction_unit_norm = curves::vector2_unit_norm(line.end - line.start);

    let mut commands = Vec::new();
    commands.append(&mut compose_line_offsetted(
        line,
        start_offset_dist,
        end_offset_dist,
        move_start,
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

#[allow(dead_code)]
pub fn compose_quadbez(quadbez: curves::QuadBezier, move_start: bool) -> Vec<path::Command> {
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
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let start_unit_norm = curves::vector2_unit_norm(quadbez.cp - quadbez.start);
    let end_unit_norm = curves::vector2_unit_norm(quadbez.end - quadbez.cp);

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
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[1],
                offset_dist_t1,
                offset_dist_t2,
                false,
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[2],
                offset_dist_t2,
                end_offset_dist,
                false,
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
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[1],
                offset_dist_t1,
                end_offset_dist,
                false,
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
            ));
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[1],
                offset_dist_t2,
                end_offset_dist,
                false,
            ));
        }
        (None, None) => {
            commands.append(&mut compose_quadbez_offsetted(
                splitted_quads[0],
                start_offset_dist,
                end_offset_dist,
                move_start,
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
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let quadbez_reverse = curves::QuadBezier {
        start: quadbez.end,
        cp: quadbez.cp,
        end: quadbez.start,
    };

    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let start_unit_norm = curves::vector2_unit_norm(quadbez.cp - quadbez.start);
    let end_unit_norm = curves::vector2_unit_norm(quadbez.end - quadbez.cp);

    let start_offset = start_unit_norm * start_offset_dist;
    let end_offset = end_unit_norm * end_offset_dist;

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        quadbez,
        start_offset_dist,
        end_offset_dist,
        move_start,
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

pub fn compose_cubbez(cubbez: curves::CubicBezier, move_start: bool) -> Vec<path::Command> {
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
    ));

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        second_quad,
        mid_offset_dist,
        end_offset_dist,
        false,
    ));

    commands
}

pub fn compose_cubbez_variable_width(
    cubbez: curves::CubicBezier,
    width_start: f64,
    width_end: f64,
    move_start: bool,
) -> Vec<path::Command> {
    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let start_unit_norm = curves::vector2_unit_norm(cubbez.cp1 - cubbez.start);
    let end_unit_norm = curves::vector2_unit_norm(cubbez.end - cubbez.cp2);

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

    let mut commands =
        compose_cubbez_offsetted(cubbez, start_offset_dist, end_offset_dist, move_start);

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
        ));
    } else {
        commands.append(&mut compose_cubbez_offsetted(
            cubbez_reverse,
            end_offset_dist,
            start_offset_dist,
            false,
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
