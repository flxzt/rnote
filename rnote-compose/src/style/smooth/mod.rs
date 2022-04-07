mod smoothoptions;

// Re-exports
pub use smoothoptions::SmoothOptions;

use crate::builders::penpathbuilder::PenPathBuilderState;
use crate::builders::PenPathBuilder;
use crate::helpers::{Affine2Helpers, Vector2Helpers};
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

use super::Composer;

// Composes a Bezier path with variable width from the line. Must be drawn with only a fill
fn compose_line_variable_width(
    line: &Line,
    width_start: f64,
    width_end: f64,
    _options: &SmoothOptions,
) -> kurbo::BezPath {
    let start_offset_dist = width_start / 2.0;
    let end_offset_dist = width_end / 2.0;

    let direction_unit_norm = (line.end - line.start).orth_unit();
    let end_arc_rotation = na::Vector2::y().angle_ahead(&(line.end - line.start));

    let mut bez_path = kurbo::BezPath::new();

    bez_path.extend(
        kurbo::Arc {
            center: line.start.to_kurbo_point(),
            radii: kurbo::Vec2::new(start_offset_dist, start_offset_dist),
            start_angle: 0.0,
            sweep_angle: std::f64::consts::PI,
            x_rotation: end_arc_rotation + std::f64::consts::PI,
        }
        .into_path(0.1)
        .into_iter(),
    );

    bez_path.extend(
        [
            kurbo::PathEl::MoveTo(
                (line.start + direction_unit_norm * start_offset_dist).to_kurbo_point(),
            ),
            kurbo::PathEl::LineTo(
                (line.start - direction_unit_norm * start_offset_dist).to_kurbo_point(),
            ),
            kurbo::PathEl::LineTo(
                (line.end - direction_unit_norm * end_offset_dist).to_kurbo_point(),
            ),
            kurbo::PathEl::LineTo(
                (line.end + direction_unit_norm * end_offset_dist).to_kurbo_point(),
            ),
            kurbo::PathEl::ClosePath,
        ]
        .into_iter(),
    );

    bez_path.extend(
        kurbo::Arc {
            center: line.end.to_kurbo_point(),
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

impl Composer<SmoothOptions> for Segment {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let shape = {
            match self {
                Segment::Dot { element } => {
                    let radii = if options.segment_constant_width {
                        na::Vector2::from_element(options.width / 2.0)
                    } else {
                        na::Vector2::from_element(element.pressure * (options.width / 2.0))
                    };
                    kurbo::Ellipse::new(element.pos.to_kurbo_point(), radii.to_kurbo_vec(), 0.0)
                        .into_path(0.1)
                }
                Segment::Line { start, end } => {
                    let (line_width_start, line_width_end) = if options.segment_constant_width {
                        (options.width, options.width)
                    } else {
                        (start.pressure * options.width, end.pressure * options.width)
                    };
                    let line = Line {
                        start: start.pos,
                        end: end.pos,
                    };

                    compose_line_variable_width(&line, line_width_start, line_width_end, &options)
                }
                Segment::QuadBez { start, cp, end } => {
                    let (width_start, width_end) = if options.segment_constant_width {
                        (options.width, options.width)
                    } else {
                        (start.pressure * options.width, end.pressure * options.width)
                    };
                    let n_splits = 5;

                    let quadbez = QuadraticBezier {
                        start: start.pos,
                        cp: *cp,
                        end: end.pos,
                    };

                    let lines = quadbez.approx_with_lines(n_splits);
                    let n_lines = lines.len() as i32;

                    lines
                        .iter()
                        .enumerate()
                        .map(|(i, line)| {
                            // splitted line start / end widths are a linear interpolation between the start and end width / n splits.
                            let line_start_width = width_start
                                + (width_end - width_start)
                                    * (f64::from(i as i32) / f64::from(n_lines));
                            let line_end_width = width_start
                                + (width_end - width_start)
                                    * (f64::from(i as i32 + 1) / f64::from(n_lines));

                            compose_line_variable_width(
                                line,
                                line_start_width,
                                line_end_width,
                                &options,
                            )
                        })
                        .flatten()
                        .collect::<kurbo::BezPath>()
                }
                Segment::CubBez {
                    start,
                    cp1,
                    cp2,
                    end,
                } => {
                    let (width_start, width_end) = if options.segment_constant_width {
                        (options.width, options.width)
                    } else {
                        (start.pressure * options.width, end.pressure * options.width)
                    };
                    let n_splits = 5;

                    let cubbez = CubicBezier {
                        start: start.pos,
                        cp1: *cp1,
                        cp2: *cp2,
                        end: end.pos,
                    };
                    let lines = cubbez.approx_with_lines(n_splits);
                    let n_lines = lines.len() as i32;

                    lines
                        .iter()
                        .enumerate()
                        .map(|(i, line)| {
                            // splitted line start / end widths are a linear interpolation between the start and end width / n splits.
                            let line_start_width = width_start
                                + (width_end - width_start)
                                    * (f64::from(i as i32) / f64::from(n_lines));
                            let line_end_width = width_start
                                + (width_end - width_start)
                                    * (f64::from(i as i32 + 1) / f64::from(n_lines));

                            compose_line_variable_width(
                                line,
                                line_start_width,
                                line_end_width,
                                &options,
                            )
                        })
                        .flatten()
                        .collect::<kurbo::BezPath>()
                }
            }
        };

        if let Some(fill_color) = options.stroke_color {
            // Outlines for debugging
            //let stroke_brush = cx.solid_brush(piet::Color::RED);
            //cx.stroke(segment.clone(), &stroke_brush, 0.4);
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(shape, &fill_brush);
        }
    }
}

impl Composer<SmoothOptions> for PenPath {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.iter()
            .map(|segment| segment.composed_bounds(options))
            .fold(AABB::new_invalid(), |acc, x| acc.merged(&x))
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        for segment in self.iter() {
            segment.draw_composed(cx, options);
        }
    }
}

impl Composer<SmoothOptions> for Line {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let line = kurbo::Line::new(self.start.to_kurbo_point(), self.end.to_kurbo_point());

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(line, &stroke_brush, options.width);
        }
    }
}

impl Composer<SmoothOptions> for Ellipse {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        cx.transform(self.transform.affine.to_kurbo());

        let ellipse = kurbo::Ellipse::new(
            kurbo::Point { x: 0.0, y: 0.0 },
            self.radii.to_kurbo_vec(),
            0.0,
        );

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(ellipse.clone(), &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(ellipse, &stroke_brush, options.width);
        }
    }
}

impl Composer<SmoothOptions> for Rectangle {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let shape = {
            let tl = self.transform.affine
                * na::point![-self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
            let tr = self.transform.affine
                * na::point![self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
            let bl = self.transform.affine
                * na::point![-self.cuboid.half_extents[0], self.cuboid.half_extents[1]];
            let br = self.transform.affine
                * na::point![self.cuboid.half_extents[0], self.cuboid.half_extents[1]];

            kurbo::BezPath::from_vec(vec![
                kurbo::PathEl::MoveTo(tl.coords.to_kurbo_point()),
                kurbo::PathEl::LineTo(tr.coords.to_kurbo_point()),
                kurbo::PathEl::LineTo(br.coords.to_kurbo_point()),
                kurbo::PathEl::LineTo(bl.coords.to_kurbo_point()),
                kurbo::PathEl::ClosePath,
            ])
        };

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(shape.clone(), &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(shape, &stroke_brush, options.width);
        }
    }
}

impl Composer<SmoothOptions> for crate::Shape {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        match self {
            crate::Shape::Line(line) => line.composed_bounds(options),
            crate::Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            crate::Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        match self {
            crate::Shape::Line(line) => line.draw_composed(cx, options),
            crate::Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            crate::Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
        }
    }
}

impl Composer<SmoothOptions> for PenPathBuilder {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.buffer.iter().fold(AABB::new_invalid(), |mut acc, x| {
            acc.take_point(na::Point2::from(x.pos));
            acc.loosened(options.width)
        })
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let penpath = match &self.state {
            PenPathBuilderState::Start => self
                .buffer
                .iter()
                .zip(self.buffer.iter().skip(1))
                .map(|(start, end)| Segment::Line {
                    start: *start,
                    end: *end,
                })
                .collect::<PenPath>(),
            // Skipping the first buffer element as that is the not drained by the segment builder and is the prev element in the "During" state
            PenPathBuilderState::During => self
                .buffer
                .iter()
                .skip(1)
                .zip(self.buffer.iter().skip(2))
                .map(|(start, end)| Segment::Line {
                    start: *start,
                    end: *end,
                })
                .collect::<PenPath>(),
        };

        penpath.draw_composed(cx, options);
    }
}
