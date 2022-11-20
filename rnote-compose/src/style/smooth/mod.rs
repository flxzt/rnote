mod smoothoptions;

// Re-exports
pub use smoothoptions::SmoothOptions;

use super::Composer;
use crate::helpers::Vector2Helpers;
use crate::penpath::Segment;
use crate::shapes::CubicBezier;
use crate::shapes::Ellipse;
use crate::shapes::Line;
use crate::shapes::QuadraticBezier;
use crate::shapes::Rectangle;
use crate::shapes::ShapeBehaviour;
use crate::PenPath;

use kurbo::Shape;
use p2d::bounding_volume::{BoundingVolume, AABB};

// Composes lines with variable width. Must be drawn with only a fill.
// Each lines has a start and end width.
fn compose_lines_variable_width(
    lines: &[(Line, f64, f64)],
    _options: &SmoothOptions,
) -> kurbo::BezPath {
    let mut bez_path = kurbo::BezPath::new();

    if lines.is_empty() {
        return bez_path;
    }

    let first_line = lines.first().unwrap();
    let last_line = lines.last().unwrap();

    let offset_coords = lines.iter().map(|(line, width_start, width_end)| {
        let direction_unit_norm = (line.end - line.start).orth_unit();

        (
            [
                line.start + direction_unit_norm * *width_start * 0.5,
                line.end + direction_unit_norm * *width_end * 0.5,
            ],
            [
                line.start - direction_unit_norm * *width_start * 0.5,
                line.end - direction_unit_norm * *width_end * 0.5,
            ],
        )
    });

    let mut pos_offset_coords = offset_coords
        .clone()
        .map(|offset_coords| offset_coords.0)
        .flatten();

    let neg_offset_coords = offset_coords
        .map(|offset_coords| offset_coords.1)
        .flatten()
        .rev();

    let start_offset_dist = first_line.1 * 0.5;
    let end_offset_dist = last_line.2 * 0.5;

    let first_line = lines.first().unwrap();
    let last_line = lines.last().unwrap();

    let start_arc_rotation = na::Vector2::y().angle_ahead(&(first_line.0.end - first_line.0.start));
    let end_arc_rotation = na::Vector2::y().angle_ahead(&(last_line.0.end - last_line.0.start));

    // Start cap
    bez_path.extend(
        kurbo::Arc {
            center: first_line.0.start.to_kurbo_point(),
            radii: kurbo::Vec2::new(start_offset_dist, start_offset_dist),
            start_angle: 0.0,
            sweep_angle: std::f64::consts::PI,
            x_rotation: start_arc_rotation + std::f64::consts::PI,
        }
        .into_path(0.1)
        .into_iter(),
    );

    // Body
    // Positive offset path
    if let Some(f) = pos_offset_coords.next() {
        bez_path.push(kurbo::PathEl::MoveTo(f.to_kurbo_point()));

        bez_path.extend(pos_offset_coords.map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())));
    }

    // Negative offset path (already reversed)
    bez_path.extend(neg_offset_coords.map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())));

    bez_path.close_path();

    // End cap
    bez_path.extend(
        kurbo::Arc {
            center: last_line.0.end.to_kurbo_point(),
            radii: kurbo::Vec2::new(end_offset_dist, end_offset_dist),
            start_angle: 0.0,
            sweep_angle: std::f64::consts::PI,
            x_rotation: end_arc_rotation,
        }
        .into_path(0.1)
        .into_iter(),
    );

    bez_path
}

impl Composer<SmoothOptions> for Line {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let line = self.to_kurbo();

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(line, &stroke_brush, options.stroke_width);
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Rectangle {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let shape = self.to_kurbo();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(shape.clone(), &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(shape, &stroke_brush, options.stroke_width);
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Ellipse {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let ellipse = self.to_kurbo();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(ellipse, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(ellipse, &stroke_brush, options.stroke_width);
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for QuadraticBezier {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let quadbez = self.to_kurbo();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(quadbez, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(quadbez, &stroke_brush, options.stroke_width);
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for CubicBezier {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let cubbez = self.to_kurbo();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(cubbez, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(cubbez, &stroke_brush, options.stroke_width);
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Segment {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();

        let bez_path = match self {
            Segment::Dot { element } => {
                let radii = na::Vector2::from_element(
                    options
                        .pressure_curve
                        .apply(options.stroke_width * 0.5, element.pressure),
                );

                kurbo::Ellipse::new(element.pos.to_kurbo_point(), radii.to_kurbo_vec(), 0.0)
                    .into_path(0.1)
            }
            Segment::Line { start, end } => {
                let (width_start, width_end) = (
                    options
                        .pressure_curve
                        .apply(options.stroke_width, start.pressure),
                    options
                        .pressure_curve
                        .apply(options.stroke_width, end.pressure),
                );

                compose_lines_variable_width(
                    &[(
                        Line {
                            start: start.pos,
                            end: end.pos,
                        },
                        width_start,
                        width_end,
                    )],
                    options,
                )
            }
            Segment::QuadBez { start, cp, end } => {
                let (width_start, width_end) = (
                    options
                        .pressure_curve
                        .apply(options.stroke_width, start.pressure),
                    options
                        .pressure_curve
                        .apply(options.stroke_width, end.pressure),
                );

                let n_splits = 5;

                let quadbez = QuadraticBezier {
                    start: start.pos,
                    cp: *cp,
                    end: end.pos,
                };

                let lines = quadbez.approx_with_lines(n_splits);
                let n_lines = lines.len();

                let lines = lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, l)| {
                        (
                            l,
                            // Lerp the width
                            width_start + (width_end - width_start) * (i as f64) / n_lines as f64,
                            width_start
                                + (width_end - width_start) * ((i + 1) as f64) / n_lines as f64,
                        )
                    })
                    .collect::<Vec<(Line, f64, f64)>>();

                compose_lines_variable_width(&lines, options)
            }
            Segment::CubBez {
                start,
                cp1,
                cp2,
                end,
            } => {
                let (width_start, width_end) = (
                    options
                        .pressure_curve
                        .apply(options.stroke_width, start.pressure),
                    options
                        .pressure_curve
                        .apply(options.stroke_width, end.pressure),
                );

                let n_splits = 5;

                let cubbez = CubicBezier {
                    start: start.pos,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: end.pos,
                };
                let lines = cubbez.approx_with_lines(n_splits);
                let n_lines = lines.len();

                let lines = lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, l)| {
                        (
                            l,
                            // Lerp the width
                            width_start + (width_end - width_start) * (i as f64) / n_lines as f64,
                            width_start
                                + (width_end - width_start) * ((i + 1) as f64) / n_lines as f64,
                        )
                    })
                    .collect::<Vec<(Line, f64, f64)>>();

                compose_lines_variable_width(&lines, options)
            }
        };

        if let Some(fill_color) = options.stroke_color {
            // Outlines for debugging
            //let stroke_brush = cx.solid_brush(piet::Color::RED);
            //cx.stroke(bez_path.clone(), &stroke_brush, 0.4);

            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(bez_path, &fill_brush);
        }

        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for PenPath {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.iter()
            .map(|segment| segment.composed_bounds(options))
            .fold(AABB::new_invalid(), |acc, x| acc.merged(&x))
    }

    // The pen path should be rendered as if each segment is rendered individually. But we still have some optimizations to reduce the complexity of the drawn shape,
    // e.g. skipping start and end caps for each segment.
    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();

        let lines = self
            .iter()
            .flat_map(|s| match s {
                Segment::Dot { .. } => {
                    vec![]
                }
                Segment::Line { start, end } => {
                    let (width_start, width_end) = (
                        options
                            .pressure_curve
                            .apply(options.stroke_width, start.pressure),
                        options
                            .pressure_curve
                            .apply(options.stroke_width, end.pressure),
                    );

                    vec![(
                        Line {
                            start: start.pos,
                            end: end.pos,
                        },
                        width_start,
                        width_end,
                    )]
                }
                Segment::QuadBez { start, cp, end } => {
                    let (width_start, width_end) = (
                        options
                            .pressure_curve
                            .apply(options.stroke_width, start.pressure),
                        options
                            .pressure_curve
                            .apply(options.stroke_width, end.pressure),
                    );

                    let n_splits = 5;

                    let quadbez = QuadraticBezier {
                        start: start.pos,
                        cp: *cp,
                        end: end.pos,
                    };

                    let lines = quadbez.approx_with_lines(n_splits);
                    let n_lines = lines.len();

                    lines
                        .into_iter()
                        .enumerate()
                        .map(|(i, l)| {
                            (
                                l,
                                // Lerp the width
                                width_start
                                    + (width_end - width_start) * (i as f64) / n_lines as f64,
                                width_start
                                    + (width_end - width_start) * ((i + 1) as f64) / n_lines as f64,
                            )
                        })
                        .collect::<Vec<(Line, f64, f64)>>()
                }
                Segment::CubBez {
                    start,
                    cp1,
                    cp2,
                    end,
                } => {
                    let (width_start, width_end) = (
                        options
                            .pressure_curve
                            .apply(options.stroke_width, start.pressure),
                        options
                            .pressure_curve
                            .apply(options.stroke_width, end.pressure),
                    );

                    let n_splits = 5;

                    let cubbez = CubicBezier {
                        start: start.pos,
                        cp1: *cp1,
                        cp2: *cp2,
                        end: end.pos,
                    };
                    let lines = cubbez.approx_with_lines(n_splits);
                    let n_lines = lines.len();

                    lines
                        .into_iter()
                        .enumerate()
                        .map(|(i, l)| {
                            (
                                l,
                                // Lerp the width
                                width_start
                                    + (width_end - width_start) * (i as f64) / n_lines as f64,
                                width_start
                                    + (width_end - width_start) * ((i + 1) as f64) / n_lines as f64,
                            )
                        })
                        .collect::<Vec<(Line, f64, f64)>>()
                }
            })
            .collect::<Vec<(Line, f64, f64)>>();

        let bez_path = compose_lines_variable_width(&lines, options);

        if let Some(fill_color) = options.stroke_color {
            // Outlines for debugging
            //let stroke_brush = cx.solid_brush(piet::Color::RED);
            //cx.stroke(bez_path.clone(), &stroke_brush, 0.4);

            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(bez_path, &fill_brush);
        }

        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for crate::Shape {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        match self {
            crate::Shape::Line(line) => line.composed_bounds(options),
            crate::Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            crate::Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.composed_bounds(options),
            crate::Shape::CubicBezier(cubbez) => cubbez.composed_bounds(options),
            crate::Shape::Segment(segment) => segment.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        match self {
            crate::Shape::Line(line) => line.draw_composed(cx, options),
            crate::Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            crate::Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.draw_composed(cx, options),
            crate::Shape::CubicBezier(cubbez) => cubbez.draw_composed(cx, options),
            crate::Shape::Segment(segment) => segment.draw_composed(cx, options),
        }
    }
}
