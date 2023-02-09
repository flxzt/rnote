mod smoothoptions;

// Re-exports
pub use smoothoptions::SmoothOptions;

use super::Composer;
use crate::helpers::Vector2Helpers;
use crate::penpath::Segment;
use crate::shapes::Arrow;
use crate::shapes::CubicBezier;
use crate::shapes::Ellipse;
use crate::shapes::Line;
use crate::shapes::QuadraticBezier;
use crate::shapes::Rectangle;
use crate::shapes::ShapeBehaviour;
use crate::PenPath;

use p2d::bounding_volume::{Aabb, BoundingVolume};

impl Composer<SmoothOptions> for Arrow {
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
                //cx.stroke(bez_path.clone(), &stroke_brush, 0.2);

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
            crate::Shape::Arrow(arrow) => arrow.composed_bounds(options),
            crate::Shape::Line(line) => line.composed_bounds(options),
            crate::Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            crate::Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.composed_bounds(options),
            crate::Shape::CubicBezier(cubbez) => cubbez.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        match self {
            crate::Shape::Arrow(arrow) => arrow.draw_composed(cx, options),
            crate::Shape::Line(line) => line.draw_composed(cx, options),
            crate::Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            crate::Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.draw_composed(cx, options),
            crate::Shape::CubicBezier(cubbez) => cubbez.draw_composed(cx, options),
        }
    }
}

// Composes lines with variable width. Must be drawn with only a fill
fn compose_lines_variable_width(
    lines: &[Line],
    start_width: f64,
    end_width: f64,
    _options: &SmoothOptions,
) -> kurbo::BezPath {
    let n_lines = lines.len();
    if n_lines == 0 {
        return kurbo::BezPath::new();
    }

    let (pos_offset_coords, neg_offset_coords): (Vec<_>, Vec<_>) = lines
        .iter()
        .enumerate()
        .flat_map(|(i, line)| {
            let line_start_width = start_width
                + (end_width - start_width) * (f64::from(i as i32) / f64::from(n_lines as u32));
            let line_end_width = start_width
                + (end_width - start_width) * (f64::from(i as i32 + 1) / f64::from(n_lines as u32));

            let dir_orth_unit = (line.end - line.start).orth_unit();

            [
                (
                    line.start + dir_orth_unit * line_start_width * 0.5,
                    line.start - dir_orth_unit * line_start_width * 0.5,
                ),
                (
                    line.end + dir_orth_unit * line_end_width * 0.5,
                    line.end - dir_orth_unit * line_end_width * 0.5,
                ),
            ]
        })
        .unzip();

    let first_line = lines.first().unwrap();
    let last_line = lines.last().unwrap();
    let start_dir_unit = (first_line.end - first_line.start).normalize();
    let end_dir_unit = (last_line.end - last_line.start).normalize();
    let start_pos_offset_coord = pos_offset_coords.first().unwrap().to_owned();
    let end_pos_offset_coord = pos_offset_coords.last().unwrap().to_owned();
    let start_neg_offset_coord = neg_offset_coords.first().unwrap().to_owned();
    let end_neg_offset_coord = neg_offset_coords.last().unwrap().to_owned();

    let mut bez_path = kurbo::BezPath::new();

    // Start cap
    bez_path.move_to(start_neg_offset_coord.to_kurbo_point());
    bez_path.curve_to(
        (start_neg_offset_coord - start_dir_unit * start_width * (2.0 / 3.0)).to_kurbo_point(),
        (start_pos_offset_coord - start_dir_unit * start_width * (2.0 / 3.0)).to_kurbo_point(),
        start_pos_offset_coord.to_kurbo_point(),
    );

    // Positive offset path
    bez_path.extend(
        pos_offset_coords
            .into_iter()
            .map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())),
    );

    // End cap
    bez_path.curve_to(
        (end_pos_offset_coord + end_dir_unit * end_width * (2.0 / 3.0)).to_kurbo_point(),
        (end_neg_offset_coord + end_dir_unit * end_width * (2.0 / 3.0)).to_kurbo_point(),
        end_neg_offset_coord.to_kurbo_point(),
    );

    // Negative offset path (needs to be reversed)
    bez_path.extend(
        neg_offset_coords
            .into_iter()
            .rev()
            .map(|c| kurbo::PathEl::LineTo(c.to_kurbo_point())),
    );
    bez_path.close_path();

    bez_path
}
