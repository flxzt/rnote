// Modules
mod smoothoptions;

// Re-exports
pub use smoothoptions::{LineCap, LineStyle, SmoothOptions};

// Imports
use super::Composer;
use crate::PenPath;
use crate::ext::Vector2Ext;
use crate::penpath::{self, Segment};
use crate::shapes::{
    Arrow, CubicBezier, Ellipse, Line, Polygon, Polyline, QuadraticBezier, Rectangle, Shapeable,
};
use kurbo::Shape;
use p2d::bounding_volume::{Aabb, BoundingVolume};

impl Composer<SmoothOptions> for Line {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();
        let line = self.outline_path();

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                line,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Arrow {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.internal_compute_bounds(Some(options.stroke_width))
            .loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.save().unwrap();

        if let Some(stroke_color) = options.stroke_color {
            let arrow = self.to_kurbo(Some(options.stroke_width));
            cx.stroke_styled(
                arrow,
                &Into::<piet::Color>::into(stroke_color),
                options.stroke_width,
                &options.piet_stroke_style,
            );
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
        let shape = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(shape.clone(), &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                shape,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
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
        let ellipse = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(&ellipse, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                ellipse,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
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
        let quadbez = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(&quadbez, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                quadbez,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
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
        let cubbez = self.outline_path();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(&cubbez, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke_styled(
                cubbez,
                &stroke_brush,
                options.stroke_width,
                &options.piet_stroke_style,
            );
        }
        cx.restore().unwrap();
    }
}

impl Composer<SmoothOptions> for Polyline {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let Some(color) = options.stroke_color else {
            return;
        };
        let n_points = self.path.len();
        let single_pos = self.path.iter().all(|p| *p == self.start);

        // Single element/position polylines need special treatment to be rendered
        if n_points == 0 || single_pos {
            cx.fill(
                kurbo::Circle::new(self.start.to_kurbo_point(), options.stroke_width),
                &Into::<piet::Color>::into(color),
            );
        } else {
            let style = options
                .piet_stroke_style
                .clone()
                .line_cap(piet::LineCap::Butt)
                .line_join(piet::LineJoin::Bevel);
            cx.stroke_styled(
                self.outline_path(),
                &Into::<piet::Color>::into(color),
                options.stroke_width,
                &style,
            );
        }
    }
}

impl Composer<SmoothOptions> for Polygon {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let Some(color) = options.stroke_color else {
            return;
        };
        let n_points = self.path.len();
        let single_pos = self.path.iter().all(|p| *p == self.start);

        // Single element/position polylines need special treatment to be rendered
        if n_points == 0 || single_pos {
            cx.fill(
                kurbo::Circle::new(self.start.to_kurbo_point(), options.stroke_width),
                &Into::<piet::Color>::into(color),
            );
        } else {
            let outline_path = self.outline_path();
            if let Some(fill_color) = options.fill_color {
                cx.fill(&outline_path, &Into::<piet::Color>::into(fill_color));
            }
            let style = options
                .piet_stroke_style
                .clone()
                .line_cap(piet::LineCap::Butt)
                .line_join(piet::LineJoin::Bevel);

            cx.stroke_styled(
                &outline_path,
                &Into::<piet::Color>::into(color),
                options.stroke_width,
                &style,
            );
        }
    }
}

impl Composer<SmoothOptions> for PenPath {
    fn composed_bounds(&self, options: &SmoothOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let Some(color) = options.stroke_color else {
            return;
        };

        let mut full_path = kurbo::BezPath::new();
        let mut single_pos = true;
        let mut prev = self.start;

        cx.save().unwrap();

        for seg in self.segments.iter() {
            if seg.end().pos == self.start.pos {
                continue;
            } else {
                single_pos = false;
            }

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

                        let quadbez = QuadraticBezier {
                            start: prev.pos,
                            cp: *cp,
                            end: end.pos,
                        };
                        let n_splits = penpath::no_subsegments_for_segment_len(
                            quadbez.outline_path().perimeter(0.25),
                        )
                        .max(2);
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

                        let cubbez = CubicBezier {
                            start: prev.pos,
                            cp1: *cp1,
                            cp2: *cp2,
                            end: end.pos,
                        };
                        let n_splits = penpath::no_subsegments_for_segment_len(
                            cubbez.outline_path().perimeter(0.25),
                        )
                        .max(2);
                        let lines = cubbez.approx_with_lines(n_splits);
                        let bez_path =
                            compose_lines_variable_width(&lines, width_start, width_end, options);

                        prev = *end;
                        bez_path
                    }
                }
            };

            // Outlines for debugging
            //let stroke_brush = cx.solid_brush(piet::Color::RED);
            //cx.stroke(bez_path.clone(), &stroke_brush, 0.2);

            full_path.extend(bez_path);
        }

        cx.fill(full_path, &Into::<piet::Color>::into(color));

        // Single element/position strokes need special treatment to be rendered
        if single_pos {
            let start_width = options
                .pressure_curve
                .apply(options.stroke_width, self.start.pressure);
            cx.fill(
                kurbo::Circle::new(self.start.pos.to_kurbo_point(), start_width * 0.5),
                &Into::<piet::Color>::into(color),
            );
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
            crate::Shape::Polyline(polyline) => polyline.composed_bounds(options),
            crate::Shape::Polygon(polygon) => polygon.composed_bounds(options),
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
            crate::Shape::Polyline(polyline) => polyline.draw_composed(cx, options),
            crate::Shape::Polygon(polygon) => polygon.draw_composed(cx, options),
        }
    }
}

/// Appends a circular arc between two points on a circle.
///
/// The arc is centered at `center` and connects `start` to `end`.
/// Between the two possible sweeps, the arc whose midpoint direction best aligns with `direction` is selected.
fn append_arc_between_points(
    path: &mut kurbo::BezPath,
    center: kurbo::Point,
    start: kurbo::Point,
    end: kurbo::Point,
    direction: kurbo::Vec2,
) {
    let start_vector = start - center;
    let end_vector = end - center;

    let radius = start_vector.length();
    let start_angle = start_vector.angle();
    let end_angle = end_vector.angle();

    // shortest signed angular sweep from start to end, constrained to [-PI, PI]
    let short_sweep = (end_angle - start_angle + std::f64::consts::PI)
        .rem_euclid(std::f64::consts::TAU)
        - std::f64::consts::PI;

    let mid_angle = start_angle + short_sweep * 0.5;
    let mid_direction = kurbo::Vec2::new(mid_angle.cos(), mid_angle.sin());

    // determine if the short or long arc is in the direction of the provided direction vector
    let sweep = if mid_direction.dot(direction) >= 0.0 {
        short_sweep
    } else {
        short_sweep - std::f64::consts::TAU * short_sweep.signum()
    };

    let arc = kurbo::Arc::new(center, (radius, radius), start_angle, sweep, 0.0);
    path.extend(arc.append_iter(0.1));
}

#[derive(Clone, Copy)]
struct Offsets {
    positive_start: kurbo::Point,
    positive_end: kurbo::Point,
    negative_start: kurbo::Point,
    negative_end: kurbo::Point,
}

/// Computes the pair of tangent offset edges for a variable-width line segment.
///
/// The segment endpoints are treated as circles whose radii are interpolated from the overall stroke width.
/// This computes the two lines that are tangent to both circles, resulting in smooth joins between differing widths.
///
/// Returns the positive- and negative-side offset coordinates for both segment endpoints.
///
/// In degenerate cases where no valid external tangent exists (e.g. when one endpoint
/// radius dominates the segment length), this falls back to simple perpendicular offsets.
fn line_offsets(
    line: &Line,
    i: usize,
    n_lines: usize,
    start_width: f64,
    end_width: f64,
) -> Offsets {
    let t_start = i as f64 / n_lines as f64;
    let t_end = (i + 1) as f64 / n_lines as f64;

    let start_radius = 0.5 * (start_width + (end_width - start_width) * t_start);
    let end_radius = 0.5 * (start_width + (end_width - start_width) * t_end);

    let segment = line.end - line.start;
    let segment_length = segment.magnitude();

    let segment_direction = segment / segment_length;
    let segment_normal = segment_direction.orth_unit();

    // we want an offset line that is tangent to both endpoint circles.

    // the tangent direction is composed of:
    // - an axial component along the segment (`segment_direction`)
    // - a lateral component perpendicular to the segment (`segment_normal`)

    // projecting that tangent direction onto the segment axis yields the axial component:
    // dot(normal, direction) = (start_radius - end_radius) / length
    let axial = (start_radius - end_radius) / segment_length;

    let (positive_direction, negative_direction) = if axial.abs() < 1.0 {
        // case with solutions, the two solutions are symmetric around the segment
        // since the direction must remain unit length, we can calculate the lateral component:
        // axial^2 + lateral^2 = 1
        let lateral = (1.0 - axial * axial).sqrt();

        (
            segment_direction * axial + segment_normal * lateral,
            segment_direction * axial - segment_normal * lateral,
        )
    } else {
        // degenerate case with no solution, the circles are too close or one contains the other
        // fall back to perpendicular offsets
        (segment_normal, -segment_normal)
    };

    Offsets {
        positive_start: (line.start + positive_direction * start_radius).to_kurbo_point(),
        positive_end: (line.end + positive_direction * end_radius).to_kurbo_point(),
        negative_start: (line.start + negative_direction * start_radius).to_kurbo_point(),
        negative_end: (line.end + negative_direction * end_radius).to_kurbo_point(),
    }
}

/// Composes lines with variable width. Must be drawn with only a fill.
fn compose_lines_variable_width(
    lines: &[Line],
    start_width: f64,
    end_width: f64,
    _options: &SmoothOptions,
) -> kurbo::BezPath {
    // The lines variable is ghosted here, to make sure we can only use the filtered
    let lines: Vec<_> = lines
        .iter()
        .filter(|line| (line.end - line.start).magnitude() > 0.0)
        .collect();

    let n_lines = lines.len();
    if n_lines == 0 {
        return kurbo::BezPath::new();
    }

    let mut offsets = Vec::with_capacity(n_lines);
    for (i, line) in lines.iter().enumerate() {
        offsets.push(line_offsets(line, i, n_lines, start_width, end_width));
    }

    let first_line = lines[0];
    let last_line = lines[n_lines - 1];
    let first_offsets = offsets[0];
    let last_offsets = offsets[n_lines - 1];

    let mut bez_path = kurbo::BezPath::new();

    // Start cap
    if start_width > 0.0 && first_offsets.positive_start != first_offsets.negative_start {
        let cap_direction = (first_line.start - first_line.end).normalize();

        bez_path.move_to(first_offsets.negative_start);
        append_arc_between_points(
            &mut bez_path,
            first_line.start.to_kurbo_point(),
            first_offsets.negative_start,
            first_offsets.positive_start,
            cap_direction.to_kurbo_vec(),
        );
    } else {
        bez_path.move_to(first_offsets.positive_start);
    }

    // Positive offset path
    for o in &offsets {
        bez_path.line_to(o.positive_start);
        bez_path.line_to(o.positive_end);
    }

    // End cap
    if end_width > 0.0 && last_offsets.positive_end != last_offsets.negative_end {
        let cap_direction = (last_line.end - last_line.start).normalize();

        append_arc_between_points(
            &mut bez_path,
            last_line.end.to_kurbo_point(),
            last_offsets.positive_end,
            last_offsets.negative_end,
            cap_direction.to_kurbo_vec(),
        );
    } else {
        bez_path.line_to(last_offsets.negative_end);
    }

    // Negative offset path (needs to be reversed)
    for o in offsets.iter().rev() {
        bez_path.line_to(o.negative_end);
        bez_path.line_to(o.negative_start);
    }

    bez_path.close_path();
    bez_path
}
