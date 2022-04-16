mod smoothoptions;

// Re-exports
pub use smoothoptions::SmoothOptions;

use crate::builders::fociellipsebuilder::FociEllipseBuilderState;
use crate::builders::linebuilder::LineBuilder;
use crate::builders::penpathbuilder::PenPathBuilderState;
use crate::builders::{EllipseBuilder, FociEllipseBuilder, PenPathBuilder, RectangleBuilder};
use crate::helpers::{AABBHelpers, Vector2Helpers};
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

use super::{drawhelpers, Composer};
use crate::penhelpers::PenState;

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
        self.bounds().loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let shape = {
            match self {
                Segment::Dot { element } => {
                    let radii = if options.segment_constant_width {
                        na::Vector2::from_element(options.stroke_width / 2.0)
                    } else {
                        na::Vector2::from_element(element.pressure * (options.stroke_width / 2.0))
                    };
                    kurbo::Ellipse::new(element.pos.to_kurbo_point(), radii.to_kurbo_vec(), 0.0)
                        .into_path(0.1)
                }
                Segment::Line { start, end } => {
                    let (line_width_start, line_width_end) = if options.segment_constant_width {
                        (options.stroke_width, options.stroke_width)
                    } else {
                        (
                            start.pressure * options.stroke_width,
                            end.pressure * options.stroke_width,
                        )
                    };
                    let line = Line {
                        start: start.pos,
                        end: end.pos,
                    };

                    compose_line_variable_width(&line, line_width_start, line_width_end, &options)
                }
                Segment::QuadBez { start, cp, end } => {
                    let (width_start, width_end) = if options.segment_constant_width {
                        (options.stroke_width, options.stroke_width)
                    } else {
                        (
                            start.pressure * options.stroke_width,
                            end.pressure * options.stroke_width,
                        )
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
                        (options.stroke_width, options.stroke_width)
                    } else {
                        (
                            start.pressure * options.stroke_width,
                            end.pressure * options.stroke_width,
                        )
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
        self.bounds().loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let line = self.to_kurbo();

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(line, &stroke_brush, options.stroke_width);
        }
    }
}

impl Composer<SmoothOptions> for Rectangle {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let shape = self.to_kurbo();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(shape.clone(), &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(shape, &stroke_brush, options.stroke_width);
        }
    }
}

impl Composer<SmoothOptions> for Ellipse {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.bounds().loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let ellipse = self.to_kurbo();

        if let Some(fill_color) = options.fill_color {
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(ellipse.clone(), &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());
            cx.stroke(ellipse, &stroke_brush, options.stroke_width);
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
            acc.loosened(options.stroke_width)
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

impl Composer<SmoothOptions> for LineBuilder {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.state_as_line().composed_bounds(options)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let line = self.state_as_line();
        line.draw_composed(cx, options);
    }
}

impl Composer<SmoothOptions> for RectangleBuilder {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.state_as_rect().composed_bounds(options)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let rect = self.state_as_rect();
        rect.draw_composed(cx, options);
    }
}

impl Composer<SmoothOptions> for EllipseBuilder {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        self.state_as_ellipse().composed_bounds(options)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        let ellipse = self.state_as_ellipse();
        ellipse.draw_composed(cx, options);
    }
}

impl Composer<SmoothOptions> for FociEllipseBuilder {
    fn composed_bounds(&self, options: &SmoothOptions) -> AABB {
        match &self.state {
            FociEllipseBuilderState::First(point) => AABB::from_half_extents(
                na::Point2::from(*point),
                na::Vector2::repeat(options.stroke_width),
            ),
            FociEllipseBuilderState::Foci(foci) => {
                AABB::new_positive(na::Point2::from(foci[0]), na::Point2::from(foci[1]))
                    .loosened(options.stroke_width)
            }
            FociEllipseBuilderState::FociAndPoint { foci, point } => {
                let ellipse = Ellipse::from_foci_and_point(*foci, *point);
                ellipse.composed_bounds(options)
            }
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &SmoothOptions) {
        match &self.state {
            FociEllipseBuilderState::First(point) => {
                let circle = kurbo::Circle::new(point.to_kurbo_point(), 2.0);
                cx.stroke(circle, &piet::Color::MAROON, 1.0);
            }
            FociEllipseBuilderState::Foci(foci) => {
                drawhelpers::draw_pos_indicator(cx, PenState::Up, foci[0], 1.0);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, foci[1], 1.0);
            }
            FociEllipseBuilderState::FociAndPoint { foci, point } => {
                let ellipse = Ellipse::from_foci_and_point(*foci, *point);

                ellipse.draw_composed(cx, options);

                drawhelpers::draw_pos_indicator(cx, PenState::Up, foci[0], 1.0);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, foci[1], 1.0);
                drawhelpers::draw_vec_indicator(cx, PenState::Down, foci[0], *point, 1.0);
                drawhelpers::draw_vec_indicator(cx, PenState::Down, foci[1], *point, 1.0);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *point, 1.0);
            }
        }
    }
}
