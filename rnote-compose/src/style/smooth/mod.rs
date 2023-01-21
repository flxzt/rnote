mod smoothoptions;

// Re-exports
pub use smoothoptions::SmoothOptions;

use super::Composer;
use crate::helpers::Vector2Helpers;
use crate::penpath::Segment;
use crate::shapes::Line;
use crate::shapes::QuadraticBezier;
use crate::shapes::Rectangle;
use crate::shapes::ShapeBehaviour;
use crate::shapes::{cubbez, CubicBezier};
use crate::shapes::{quadbez, Ellipse};
use crate::PenPath;

use p2d::bounding_volume::{Aabb, BoundingVolume};

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

        let start_point = (
            self.start.pos,
            options
                .pressure_curve
                .apply(options.stroke_width, self.start.pressure),
        );
        let mut path_points = vec![start_point];

        let mut prev = self.start;
        for seg in self.segments.iter() {
            match seg {
                Segment::LineTo { end } => {
                    let width_end = options
                        .pressure_curve
                        .apply(options.stroke_width, end.pressure);

                    path_points.append(&mut vec![(end.pos, width_end)]);
                    prev = *end;
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

                    let mut quadbez_points = (1..n_splits)
                        .map(|i| {
                            let t = f64::from(i) / f64::from(n_splits);

                            (
                                quadbez::quadbez_calc(prev.pos, *cp, end.pos, t),
                                width_start + (width_end - width_start) * t,
                            )
                        })
                        .collect::<Vec<(na::Vector2<f64>, f64)>>();

                    path_points.append(&mut quadbez_points);
                    prev = *end;
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

                    let mut cubbez_points = (1..n_splits)
                        .map(|i| {
                            let t = f64::from(i) / f64::from(n_splits);

                            (
                                cubbez::cubbez_calc(prev.pos, *cp1, *cp2, end.pos, t),
                                width_start + (width_end - width_start) * t,
                            )
                        })
                        .collect::<Vec<(na::Vector2<f64>, f64)>>();

                    path_points.append(&mut cubbez_points);
                    prev = *end;
                }
            }
        }

        //debug_points(cx, &points_var_width);
        draw_path_variable_width(cx, &path_points, options);

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

// draws a path with variable width.
fn draw_path_variable_width(
    cx: &mut impl piet::RenderContext,
    path_points: &[(na::Vector2<f64>, f64)],
    options: &SmoothOptions,
) {
    let n_points = path_points.len();
    let mut bez_path = kurbo::BezPath::new();

    if n_points < 2 {
        return;
    }

    let Some(color) = options.stroke_color else {
        return;
    };
    let fill_brush = cx.solid_brush(color.into());

    let first_point = *path_points.first().unwrap();
    let second_point = *path_points.get(1).unwrap();
    let last_point = *path_points.last().unwrap();

    let start_unit_norm = (second_point.0 - first_point.0).orth_unit();

    let mut pos_offset_coords = Vec::with_capacity(n_points);
    let mut neg_offset_coords = Vec::with_capacity(n_points);

    pos_offset_coords.push(first_point.0 + start_unit_norm * first_point.1 * 0.5);
    neg_offset_coords.push(first_point.0 - start_unit_norm * first_point.1 * 0.5);

    let mut points_iter = path_points.into_iter();
    let mut prev = *points_iter.next().unwrap();
    for point in points_iter {
        let direction_unit_norm = (point.0 - prev.0).orth_unit();

        pos_offset_coords.push(point.0 + direction_unit_norm * point.1 * 0.5);
        neg_offset_coords.push(point.0 - direction_unit_norm * point.1 * 0.5);

        prev = *point;
    }

    let start_arc_radius = first_point.1 * 0.5;
    let end_arc_radius = last_point.1 * 0.5;

    // Start cap
    let start_cap = kurbo::Ellipse::new(
        first_point.0.to_kurbo_point(),
        kurbo::Vec2::new(start_arc_radius, start_arc_radius),
        0.0,
    );

    let mut pos_offset_coords_iter = pos_offset_coords.into_iter();
    let neg_offset_coords_iter = neg_offset_coords.into_iter().rev();

    // Positive offset path
    if let Some(first_pos_offset_coord) = pos_offset_coords_iter.next() {
        bez_path.push(kurbo::PathEl::MoveTo(
            first_pos_offset_coord.to_kurbo_point(),
        ));

        bez_path.extend(
            pos_offset_coords_iter
                .into_iter()
                .map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())),
        );
    }

    // Negative offset path (already reversed)
    bez_path.extend(neg_offset_coords_iter.map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())));

    bez_path.close_path();

    // End cap
    let end_cap = kurbo::Ellipse::new(
        last_point.0.to_kurbo_point(),
        kurbo::Vec2::new(end_arc_radius, end_arc_radius),
        0.0,
    );

    // Draw
    cx.fill(bez_path.clone(), &fill_brush);
    cx.fill(start_cap, &fill_brush);
    cx.fill(end_cap, &fill_brush);

    // debugging
    //draw_debug_path(cx, path_points);
    //draw_debug_points(cx, path_points);
    // outlines
    //cx.stroke(bez_path.clone(), &piet::Color::RED, 0.2);
}

#[allow(unused)]
fn draw_debug_points(cx: &mut impl piet::RenderContext, points: &[(na::Vector2<f64>, f64)]) {
    let mut color = piet::Color::rgba(1.0, 1.0, 0.0, 1.0);
    for (i, (point, _pressure)) in points.into_iter().enumerate() {
        let stroke_brush = cx.solid_brush(piet::Color::RED);

        cx.fill(kurbo::Circle::new(point.to_kurbo_point(), 0.5), &color);

        // Shift the color so the point order becomes visible
        let color_step = (2.0 * std::f64::consts::PI) / (8 as f64);
        let rgb_offset = (2.0 / 3.0) * std::f64::consts::PI;
        let color_offset = (5.0 / 4.0) * std::f64::consts::PI + 0.4;

        color = piet::Color::rgba(
            0.5 * (i as f64 * color_step + 0.0 * rgb_offset + color_offset).sin() + 0.5,
            0.5 * (i as f64 * color_step + 1.0 * rgb_offset + color_offset).sin() + 0.5,
            0.5 * (i as f64 * color_step + 2.0 * rgb_offset + color_offset).sin() + 0.5,
            1.0,
        )
    }
}

#[allow(unused)]
fn draw_debug_path(cx: &mut impl piet::RenderContext, points: &[(na::Vector2<f64>, f64)]) {
    let mut points_iter = points
        .into_iter()
        .map(|p| kurbo::Point::new(p.0[0], p.0[1]));

    if let Some(prev) = points_iter.next() {
        let bez_path = kurbo::BezPath::from_iter(
            std::iter::once(kurbo::PathEl::MoveTo(prev))
                .chain(points_iter.map(|p| kurbo::PathEl::LineTo(p))),
        );

        cx.stroke(bez_path, &piet::Color::FUCHSIA, 1.0);
    }
}
