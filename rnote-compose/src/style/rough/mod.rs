mod roughgenerator;
/// The module for the rough style.
pub mod roughoptions;

use p2d::bounding_volume::{BoundingVolume, AABB};
// Re-exports
pub use roughoptions::RoughOptions;

use super::Composer;
use crate::helpers::{Affine2Helpers, Vector2Helpers};
use crate::penpath::Segment;
use crate::shapes::Line;
use crate::shapes::Rectangle;
use crate::shapes::{CubicBezier, ShapeBehaviour};
use crate::shapes::{Ellipse, QuadraticBezier};
use crate::PenPath;

/// This is a (incomplete) port of the [Rough.js](https://roughjs.com/) javascript library to Rust.
/// Rough.js is a small (<9kB gzipped) graphics library that lets you draw in a sketchy, hand-drawn-like, style.

/// Generating a fill polygon
fn fill_polygon(coords: Vec<na::Vector2<f64>>, options: &RoughOptions) -> kurbo::BezPath {
    let mut rng = crate::utils::new_rng_default_pcg64(options.seed);

    roughgenerator::fill_polygon(coords, options, &mut rng)
}

// Composer implementations

impl Composer<RoughOptions> for Line {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::AABB {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();
        let mut rng = crate::utils::new_rng_default_pcg64(options.seed);

        let bez_path = if !options.disable_multistroke {
            roughgenerator::doubleline(self.start, self.end, options, &mut rng)
        } else {
            roughgenerator::line(self.start, self.end, true, false, options, &mut rng)
        };

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());

            cx.stroke(bez_path, &stroke_brush, options.stroke_width)
        }

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for Rectangle {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::AABB {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();
        let mut rng = crate::utils::new_rng_default_pcg64(options.seed);

        cx.transform(self.transform.affine.to_kurbo());

        let mut rect_path = kurbo::BezPath::new();

        let top_left = -self.cuboid.half_extents;
        let bottom_right = self.cuboid.half_extents;

        if !options.disable_multistroke {
            rect_path.extend(
                roughgenerator::doubleline(
                    top_left,
                    na::vector![bottom_right[0], top_left[1]],
                    options,
                    &mut rng,
                )
                .into_iter(),
            );
            rect_path.extend(roughgenerator::doubleline(
                na::vector![bottom_right[0], top_left[1]],
                bottom_right,
                options,
                &mut rng,
            ));
            rect_path.extend(
                roughgenerator::doubleline(
                    bottom_right,
                    na::vector![top_left[0], bottom_right[1]],
                    options,
                    &mut rng,
                )
                .into_iter(),
            );
            rect_path.extend(
                roughgenerator::doubleline(
                    na::vector![top_left[0], bottom_right[1]],
                    top_left,
                    options,
                    &mut rng,
                )
                .into_iter(),
            );
        } else {
            rect_path.extend(
                roughgenerator::line(
                    top_left,
                    na::vector![bottom_right[0], top_left[1]],
                    true,
                    false,
                    options,
                    &mut rng,
                )
                .into_iter(),
            );
            rect_path.extend(
                roughgenerator::line(
                    na::vector![bottom_right[0], top_left[1]],
                    bottom_right,
                    true,
                    false,
                    options,
                    &mut rng,
                )
                .into_iter(),
            );
            rect_path.extend(roughgenerator::line(
                bottom_right,
                na::vector![top_left[0], bottom_right[1]],
                true,
                false,
                options,
                &mut rng,
            ));
            rect_path.extend(
                roughgenerator::line(
                    na::vector![top_left[0], bottom_right[1]],
                    top_left,
                    true,
                    false,
                    options,
                    &mut rng,
                )
                .into_iter(),
            );
        }

        if let Some(fill_color) = options.fill_color {
            let fill_points = vec![
                na::vector![top_left[0], top_left[1]],
                na::vector![bottom_right[0], top_left[1]],
                na::vector![bottom_right[0], bottom_right[1]],
                na::vector![top_left[0], bottom_right[1]],
            ];
            let fill_polygon = fill_polygon(fill_points, options);

            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(fill_polygon, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());

            cx.stroke(rect_path, &stroke_brush, options.stroke_width)
        }

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for Ellipse {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::AABB {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();
        let mut rng = crate::utils::new_rng_default_pcg64(options.seed);

        cx.transform(self.transform.affine.to_kurbo());

        let ellipse_result = roughgenerator::ellipse(
            na::vector![0.0, 0.0],
            self.radii[0],
            self.radii[1],
            options,
            &mut rng,
        );

        if let Some(fill_color) = options.fill_color {
            let fill_polygon = fill_polygon(ellipse_result.estimated_points, options);

            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(fill_polygon, &fill_brush);
        }

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());

            cx.stroke(ellipse_result.bez_path, &stroke_brush, options.stroke_width)
        }

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for QuadraticBezier {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::AABB {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();
        let mut rng = crate::utils::new_rng_default_pcg64(options.seed);

        let bez_path =
            roughgenerator::quadratic_bezier(self.start, self.cp, self.end, options, &mut rng);

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());

            cx.stroke(bez_path, &stroke_brush, options.stroke_width)
        }

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for CubicBezier {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::AABB {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();
        let mut rng = crate::utils::new_rng_default_pcg64(options.seed);

        let bez_path = roughgenerator::cubic_bezier(
            self.start, self.cp1, self.cp2, self.end, options, &mut rng,
        );

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());

            cx.stroke(bez_path, &stroke_brush, options.stroke_width)
        }

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for Segment {
    fn composed_bounds(&self, options: &RoughOptions) -> AABB {
        self.bounds().loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();
        let mut options = options.clone();
        options.disable_multistroke = true;
        options.preserve_vertices = true;

        if let Some(stroke_color) = options.stroke_color {
            let stroke_brush = cx.solid_brush(stroke_color.into());

            match self {
                Segment::Dot { element } => {
                    let radii = na::Vector2::from_element(options.stroke_width / 2.0);
                    let dot_ellipse = kurbo::Ellipse::new(
                        element.pos.to_kurbo_point(),
                        radii.to_kurbo_vec(),
                        0.0,
                    );

                    cx.fill(dot_ellipse, &stroke_brush);
                }
                Segment::Line { start, end } => {
                    let line = Line {
                        start: start.pos,
                        end: end.pos,
                    };

                    line.draw_composed(cx, &options)
                }
                Segment::QuadBez { start, cp, end } => {
                    let n_splits = 5;

                    let quadbez = QuadraticBezier {
                        start: start.pos,
                        cp: *cp,
                        end: end.pos,
                    };

                    let lines = quadbez.approx_with_lines(n_splits);

                    lines
                        .iter()
                        .for_each(|line| line.draw_composed(cx, &options));
                }
                Segment::CubBez {
                    start,
                    cp1,
                    cp2,
                    end,
                } => {
                    let n_splits = 5;

                    let cubbez = CubicBezier {
                        start: start.pos,
                        cp1: *cp1,
                        cp2: *cp2,
                        end: end.pos,
                    };
                    let lines = cubbez.approx_with_lines(n_splits);

                    lines
                        .iter()
                        .for_each(|line| line.draw_composed(cx, &options));
                }
            }
        }
        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for PenPath {
    fn composed_bounds(&self, options: &RoughOptions) -> AABB {
        self.iter()
            .map(|segment| segment.composed_bounds(options))
            .fold(AABB::new_invalid(), |acc, x| acc.merged(&x))
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();
        for segment in self.iter() {
            segment.draw_composed(cx, options);
        }
        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for crate::Shape {
    fn composed_bounds(&self, options: &RoughOptions) -> AABB {
        match self {
            crate::Shape::Line(line) => line.composed_bounds(options),
            crate::Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            crate::Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.composed_bounds(options),
            crate::Shape::CubicBezier(cubbez) => cubbez.composed_bounds(options),
            crate::Shape::Segment(segment) => segment.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
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
