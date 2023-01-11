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
use p2d::bounding_volume::{Aabb, BoundingVolume};

// Composes lines with variable width. Must be drawn with only a fill
fn compose_lines_variable_width(
    lines: &[Line],
    width_start: f64,
    width_end: f64,
    _options: &SmoothOptions,
) -> kurbo::BezPath {
    let mut bez_path = kurbo::BezPath::new();

    if lines.is_empty() {
        return bez_path;
    }
    let n_lines = lines.len() as u32;

    let offset_coords = lines.iter().enumerate().map(|(i, line)| {
        let line_start_width =
            width_start + (width_end - width_start) * (f64::from(i as i32) / f64::from(n_lines));
        let line_end_width = width_start
            + (width_end - width_start) * (f64::from(i as i32 + 1) / f64::from(n_lines));

        let direction_unit_norm = (line.end - line.start).orth_unit();

        (
            [
                line.start + direction_unit_norm * line_start_width * 0.5,
                line.end + direction_unit_norm * line_end_width * 0.5,
            ],
            [
                line.start - direction_unit_norm * line_start_width * 0.5,
                line.end - direction_unit_norm * line_end_width * 0.5,
            ],
        )
    });

    let mut pos_offset_coords = offset_coords
        .clone()
        .flat_map(|offset_coords| offset_coords.0)
        .collect::<Vec<na::Vector2<f64>>>()
        .into_iter();

    let neg_offset_coords = offset_coords
        .flat_map(|offset_coords| offset_coords.1)
        .rev()
        .collect::<Vec<na::Vector2<f64>>>()
        .into_iter();

    let start_offset_dist = width_start * 0.5;
    let end_offset_dist = width_end * 0.5;

    let first_line = lines.first().unwrap();
    let last_line = lines.last().unwrap();

    let start_arc_rotation = na::Vector2::y().angle_ahead(&(first_line.end - first_line.start));
    let end_arc_rotation = na::Vector2::y().angle_ahead(&(last_line.end - last_line.start));

    // Start cap
    bez_path.extend(
        kurbo::Arc {
            center: first_line.start.to_kurbo_point(),
            radii: kurbo::Vec2::new(start_offset_dist, start_offset_dist),
            start_angle: 0.0,
            sweep_angle: std::f64::consts::PI,
            x_rotation: start_arc_rotation + std::f64::consts::PI,
        }
        .into_path(0.1)
        .into_iter(),
    );

    // Positive offset path
    if let Some(first_pos_offset_coord) = pos_offset_coords.next() {
        bez_path.push(kurbo::PathEl::MoveTo(
            first_pos_offset_coord.to_kurbo_point(),
        ));

        for pos_offset_coord in pos_offset_coords {
            bez_path.push(kurbo::PathEl::LineTo(pos_offset_coord.to_kurbo_point()));
        }
    }

    // Negative offset path (already reversed)
    for pos_offset_coord in neg_offset_coords {
        bez_path.push(kurbo::PathEl::LineTo(pos_offset_coord.to_kurbo_point()));
    }

    // End cap
    bez_path.extend(
        kurbo::Arc {
            center: last_line.end.to_kurbo_point(),
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
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
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
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
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
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
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
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
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
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
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

impl Composer<SmoothOptions> for PenPath {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();

        let mut prev = self.start;
        for seg in self.segments.iter() {
            let bez_path = {
                match seg {
                    Segment::LineTo { end } => {
                        let (width_start, width_end) = (
                            options
                                .pressure_curve
                                .apply(options.stroke_width, prev.pressure),
                            options
                                .pressure_curve
                                .apply(options.stroke_width, end.pressure),
                        );

                        let bez_path = compose_lines_variable_width(
                            &[Line {
                                start: prev.pos,
                                end: end.pos,
                            }],
                            width_start,
                            width_end,
                            options,
                        );

                        prev = *end;
                        bez_path
                    }
                    Segment::QuadBezTo { cp, end } => {
                        let (width_start, width_end) = (
                            options
                                .pressure_curve
                                .apply(options.stroke_width, prev.pressure),
                            options
                                .pressure_curve
                                .apply(options.stroke_width, end.pressure),
                        );

                        let n_splits = 5;

                        let quadbez = QuadraticBezier {
                            start: prev.pos,
                            cp: *cp,
                            end: end.pos,
                        };

                        let lines = quadbez.approx_with_lines(n_splits);

                        let bez_path =
                            compose_lines_variable_width(&lines, width_start, width_end, options);

                        prev = *end;
                        bez_path
                    }
                    Segment::CubBezTo { cp1, cp2, end } => {
                        let (width_start, width_end) = (
                            options
                                .pressure_curve
                                .apply(options.stroke_width, prev.pressure),
                            options
                                .pressure_curve
                                .apply(options.stroke_width, end.pressure),
                        );

                        let n_splits = 5;

                        let cubbez = CubicBezier {
                            start: prev.pos,
                            cp1: *cp1,
                            cp2: *cp2,
                            end: end.pos,
                        };
                        let lines = cubbez.approx_with_lines(n_splits);

                        let bez_path =
                            compose_lines_variable_width(&lines, width_start, width_end, options);

                        prev = *end;
                        bez_path
                    }
                }
            };

            if let Some(fill_color) = options.stroke_color {
                // Outlines for debugging
                //let stroke_brush = cx.solid_brush(piet::Color::RED);
                //cx.stroke(bez_path.clone(), &stroke_brush, 0.4);

                let fill_brush = cx.solid_brush(fill_color.into());
                cx.fill(bez_path, &fill_brush);
            }
        }

        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for crate::Shape {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        match self {
            crate::Shape::Line(line) => line.composed_bounds(options),
            crate::Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            crate::Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.composed_bounds(options),
            crate::Shape::CubicBezier(cubbez) => cubbez.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        match self {
            crate::Shape::Line(line) => line.draw_composed(cx, options),
            crate::Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            crate::Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.draw_composed(cx, options),
            crate::Shape::CubicBezier(cubbez) => cubbez.draw_composed(cx, options),
        }
    }
}
