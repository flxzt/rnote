pub mod roughoptions;
mod roughshapes;

use svg::node::element::{self, path};

use roughoptions::Options;

use super::{curves, shapes};
use crate::compose;

/* The rough module.
This is a port of the [Rough.js](https://roughjs.com/) javascript library to Rust.
Rough.js is a small (<9kB gzipped) graphics library that lets you draw in a sketchy, hand-drawn-like, style.
The library defines primitives to draw lines, curves, arcs, polygons, circles, and ellipses. It also supports drawing SVG paths.
*/

/// Generating a single line element
pub fn line(options: &Options, line: curves::Line) -> element::Path {
    let mut rng = compose::new_rng_default_pcg64(options.seed);

    let commands = if !options.disable_multistroke {
        roughshapes::doubleline(line.start, line.end, options, &mut rng)
    } else {
        roughshapes::line(line.start, line.end, options, &mut rng, true, false)
    };

    options.apply_to_line(element::Path::new().set("d", path::Data::from(commands)))
}

/// Generating a cubic bezier curve
pub fn cubic_bezier(options: &Options, cubbez: curves::CubicBezier) -> element::Path {
    let mut rng = compose::new_rng_default_pcg64(options.seed);

    let commands = roughshapes::cubic_bezier(
        cubbez.start,
        cubbez.cp1,
        cubbez.cp2,
        cubbez.end,
        options,
        &mut rng,
    );

    options.apply_to_line(element::Path::new().set("d", path::Data::from(commands)))
}

/// Generating a rectangle
pub fn rectangle(options: &Options, rectangle: shapes::Rectangle) -> element::Group {
    let mut rng = compose::new_rng_default_pcg64(options.seed);

    let mut commands = Vec::new();
    // Applying the transform at the end
    let top_left = -rectangle.cuboid.half_extents;
    let bottom_right = rectangle.cuboid.half_extents;

    if !options.disable_multistroke {
        commands.append(&mut roughshapes::doubleline(
            top_left,
            na::vector![bottom_right[0], top_left[1]],
            options,
            &mut rng,
        ));
        commands.append(&mut roughshapes::doubleline(
            na::vector![bottom_right[0], top_left[1]],
            bottom_right,
            options,
            &mut rng,
        ));
        commands.append(&mut roughshapes::doubleline(
            bottom_right,
            na::vector![top_left[0], bottom_right[1]],
            options,
            &mut rng,
        ));
        commands.append(&mut roughshapes::doubleline(
            na::vector![top_left[0], bottom_right[1]],
            top_left,
            options,
            &mut rng,
        ));
    } else {
        commands.append(&mut roughshapes::line(
            top_left,
            na::vector![bottom_right[0], top_left[1]],
            options,
            &mut rng,
            true,
            false,
        ));
        commands.append(&mut roughshapes::line(
            na::vector![bottom_right[0], top_left[1]],
            bottom_right,
            options,
            &mut rng,
            true,
            false,
        ));
        commands.append(&mut roughshapes::line(
            bottom_right,
            na::vector![top_left[0], bottom_right[1]],
            options,
            &mut rng,
            true,
            false,
        ));
        commands.append(&mut roughshapes::line(
            na::vector![top_left[0], bottom_right[1]],
            top_left,
            options,
            &mut rng,
            true,
            false,
        ));
    }

    let rect = options.apply_to_rect(element::Path::new().set("d", path::Data::from(commands)));

    let fill_points = vec![
        na::vector![top_left[0], top_left[1]],
        na::vector![bottom_right[0], top_left[1]],
        na::vector![bottom_right[0], bottom_right[1]],
        na::vector![top_left[0], bottom_right[1]],
    ];
    let fill_polygon = fill_polygon(options, fill_points);

    let transform_string = rectangle.transform.transform_as_svg_transform_attr();

    element::Group::new()
        .set("transform", transform_string)
        .add(fill_polygon)
        .add(rect)
}

/// Generating a fill polygon
pub fn fill_polygon(options: &Options, coords: Vec<na::Vector2<f64>>) -> element::Path {
    let mut rng = compose::new_rng_default_pcg64(options.seed);

    let mut commands = Vec::new();
    commands.append(&mut roughshapes::fill_polygon(coords, options, &mut rng));

    options.apply_to_fill_polygon_solid(element::Path::new().set("d", path::Data::from(commands)))
}

/// Generating a ellipse
pub fn ellipse(options: &Options, ellipse: shapes::Ellipse) -> element::Group {
    let mut rng = compose::new_rng_default_pcg64(options.seed);

    let ellipse_result = roughshapes::ellipse(
        na::vector![0.0, 0.0],
        ellipse.radii[0],
        ellipse.radii[1],
        options,
        &mut rng,
    );

    let transform_string = ellipse.transform.transform_as_svg_transform_attr();

    let ellipse = options.apply_to_ellipse(
        element::Path::new()
            .set("transform", transform_string)
            .set("d", path::Data::from(ellipse_result.commands)),
    );

    let fill_polygon = fill_polygon(options, ellipse_result.estimated_points);

    element::Group::new().add(fill_polygon).add(ellipse)
}
