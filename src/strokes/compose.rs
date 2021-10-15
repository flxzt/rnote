use gtk4::{gio, glib};
use svg::node::element::path;

use super::curves;

#[allow(dead_code)]
pub fn add_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(r#"<\?xml[^\?>]*\?>"#).unwrap();
    if !re.is_match(svg) {
        let string = String::from(r#"<?xml version="1.0" standalone="no"?>"#) + "\n" + svg;
        string
    } else {
        String::from(svg)
    }
}

pub fn remove_xml_header(svg: &str) -> String {
    let re = regex::Regex::new(r#"<\?xml[^\?>]*\?>"#).unwrap();
    String::from(re.replace_all(svg, ""))
}

#[allow(dead_code)]
pub fn strip_svg_root(svg: &str) -> String {
    let re = regex::Regex::new(r#"<svg[^>]*>|<[^/svg]*/svg>"#).unwrap();
    String::from(re.replace_all(svg, ""))
}

pub const SVG_WRAP_TEMPL_STR: &str = r#"
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

pub fn wrap_svg(
    data: &str,
    bounds: Option<p2d::bounding_volume::AABB>,
    viewbox: Option<p2d::bounding_volume::AABB>,
    xml_header: bool,
    preserve_aspectratio: bool,
) -> String {
    let mut cx = tera::Context::new();

    let (x, y, width, height) = if let Some(bounds) = bounds {
        let x = format!("{}", bounds.mins[0].floor() as i32);
        let y = format!("{}", bounds.mins[1].floor() as i32);
        let width = format!("{}", (bounds.maxs[0] - bounds.mins[0]).ceil() as i32);
        let height = format!("{}", (bounds.maxs[1] - bounds.mins[1]).ceil() as i32);

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
            "viewBox=\"{} {} {} {}\"",
            viewbox.mins[0].floor() as i32,
            viewbox.mins[1].floor() as i32,
            (viewbox.maxs[0] - viewbox.mins[0]).ceil() as i32,
            (viewbox.maxs[1] - viewbox.mins[1]).ceil() as i32
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

    let output = tera::Tera::one_off(SVG_WRAP_TEMPL_STR, &cx, false)
        .expect("failed to create svg from template");

    output
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
            log::warn!("intrinsic_size_in_pixels() failed in svg_intrinsic_size()");
            None
        };

        intrinsic_size
    } else {
        None
    }
}

pub fn compose_linear_offsetted(
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

pub fn compose_linear_variable_width(
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
    commands.append(&mut compose_linear_offsetted(
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
    commands.append(&mut compose_linear_offsetted(
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

    commands
}

pub fn compose_quadbez_offsetted(
    quad_bezier: curves::QuadBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    move_start: bool,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let start_unit_norm = curves::vector2_unit_norm(quad_bezier.cp - quad_bezier.start);
    let end_unit_norm = curves::vector2_unit_norm(quad_bezier.end - quad_bezier.cp);

    let start_offset = start_unit_norm * start_offset_dist;
    let end_offset = end_unit_norm * end_offset_dist;

    let added_unit_norms = start_unit_norm + end_unit_norm;

    // Not sure if lerping is mathematically correct?
    let cp_offset_dist = (start_offset_dist + end_offset_dist) / 2.0;

    let cp_offset =
        (2.0 * cp_offset_dist * added_unit_norms) / added_unit_norms.dot(&added_unit_norms);

    if move_start {
        commands.push(path::Command::Move(
            path::Position::Absolute,
            path::Parameters::from((
                quad_bezier.start[0] + start_offset[0],
                quad_bezier.start[1] + start_offset[1],
            )),
        ));
    }
    commands.push(path::Command::QuadraticCurve(
        path::Position::Absolute,
        path::Parameters::from((
            (
                quad_bezier.cp[0] + cp_offset[0],
                quad_bezier.cp[1] + cp_offset[1],
            ),
            (
                quad_bezier.end[0] + end_offset[0],
                quad_bezier.end[1] + end_offset[1],
            ),
        )),
    ));

    commands
}

// Offsetted quad bezier approximation, see "precise offsetting of quadratic bezier curves"
pub fn compose_quadbez_offsetted_w_subdivision(
    quad_bezier: curves::QuadBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    move_start: bool,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let (splitted_quads, split_t1, split_t2) =
        curves::split_quadbez_critical_points(quad_bezier, start_offset_dist, end_offset_dist);

    match (split_t1, split_t2) {
        (Some(split_t1), Some(split_t2)) => {
            let offset_dist_t1 = curves::quadbez_calc_offset_dist_at_t(
                quad_bezier,
                start_offset_dist,
                end_offset_dist,
                split_t1,
            );
            let offset_dist_t2 = curves::quadbez_calc_offset_dist_at_t(
                quad_bezier,
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
                quad_bezier,
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
                quad_bezier,
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
    quad_bezier: curves::QuadBezier,
    width_start: f64,
    width_end: f64,
    move_start: bool,
) -> Vec<path::Command> {
    let mut commands = Vec::new();

    let quad_bezier_reverse = curves::QuadBezier {
        start: quad_bezier.end,
        cp: quad_bezier.cp,
        end: quad_bezier.start,
    };

    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let start_unit_norm = curves::vector2_unit_norm(quad_bezier.cp - quad_bezier.start);
    let end_unit_norm = curves::vector2_unit_norm(quad_bezier.end - quad_bezier.cp);

    let start_offset = start_unit_norm * start_offset_dist;
    let end_offset = end_unit_norm * end_offset_dist;

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        quad_bezier,
        start_offset_dist,
        end_offset_dist,
        move_start,
    ));
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (quad_bezier.end - end_offset)[0],
            (quad_bezier.end - end_offset)[1],
        )),
    ));

    commands.append(&mut compose_quadbez_offsetted_w_subdivision(
        quad_bezier_reverse,
        end_offset_dist,
        start_offset_dist,
        false,
    ));
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (quad_bezier.start + start_offset)[0],
            (quad_bezier.start + start_offset)[1],
        )),
    ));

    commands
}

pub fn compose_cubbez_offsetted(
    cubic_bezier: curves::CubicBezier,
    start_offset_dist: f64,
    end_offset_dist: f64,
    move_start: bool,
) -> Vec<path::Command> {
    let t = 0.5;
    let mid_offset_dist = start_offset_dist + (end_offset_dist - start_offset_dist) * t;

    let (first_cubic, second_cubic) = curves::split_cubbez(cubic_bezier, t);
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
    cubic_bezier: curves::CubicBezier,
    width_start: f64,
    width_end: f64,
    move_start: bool,
) -> Vec<path::Command> {
    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let start_unit_norm = curves::vector2_unit_norm(cubic_bezier.cp1 - cubic_bezier.start);
    let end_unit_norm = curves::vector2_unit_norm(cubic_bezier.end - cubic_bezier.cp2);

    let start_offset = start_unit_norm * start_offset_dist;
    let end_offset = end_unit_norm * end_offset_dist;

    let cubic_bezier_reverse = curves::CubicBezier {
        start: cubic_bezier.end,
        cp1: cubic_bezier.cp2,
        cp2: cubic_bezier.cp1,
        end: cubic_bezier.start,
    };

    // if the angle of the two offsets is > 90deg, calculating the norms went wrong, so reverse them.
    let angle = start_offset.angle(&end_offset).to_degrees();
    let angle_greater_90 = if angle < -90.0 && angle > 90.0 {
        true
    } else {
        false
    };

    let mut commands =
        compose_cubbez_offsetted(cubic_bezier, start_offset_dist, end_offset_dist, move_start);

    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (cubic_bezier.end - end_offset)[0],
            (cubic_bezier.end - end_offset)[1],
        )),
    ));

    // If angle > 90.0 degrees, reverse the cubic_bezier vector (using the original cubic_bezier, but with offsets of the reversed)
    if angle_greater_90 {
        commands.append(&mut compose_cubbez_offsetted(
            cubic_bezier,
            -end_offset_dist,
            -start_offset_dist,
            false,
        ));
    } else {
        commands.append(&mut compose_cubbez_offsetted(
            cubic_bezier_reverse,
            end_offset_dist,
            start_offset_dist,
            false,
        ));
    }
    commands.push(path::Command::Line(
        path::Position::Absolute,
        path::Parameters::from((
            (cubic_bezier.start + start_offset)[0],
            (cubic_bezier.start + start_offset)[1],
        )),
    ));

    commands
}
