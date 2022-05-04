mod textureddotsdistribution;
mod texturedoptions;

use kurbo::Shape;
use p2d::bounding_volume::{BoundingVolume, AABB};
// Re-exports
pub use textureddotsdistribution::TexturedDotsDistribution;
pub use texturedoptions::TexturedOptions;

use crate::helpers::Vector2Helpers;
use crate::penpath::Segment;
use crate::shapes::{Line, ShapeBehaviour};
use crate::PenPath;

use rand_distr::{Distribution, Uniform};

use super::Composer;

impl Composer<TexturedOptions> for Line {
    fn composed_bounds(&self, options: &TexturedOptions) -> AABB {
        self.bounds().loosened(options.stroke_width * 0.5)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &TexturedOptions) {
        cx.save().unwrap();
        let bez_path = {
            // Return early if line has no length, else Uniform::new() will panic for range with low >= high
            if (self.end - self.start).magnitude() <= 0.0 {
                return;
            };

            let mut rng = crate::utils::new_rng_default_pcg64(options.seed);

            let line_vec = self.end - self.start;
            let line_rect = self.line_w_width_to_rect(options.stroke_width);

            let area = 4.0 * line_rect.cuboid.half_extents[0] * line_rect.cuboid.half_extents[1];

            // Ranges for randomization
            let range_x = -line_rect.cuboid.half_extents[0]..line_rect.cuboid.half_extents[0];
            let range_y = -line_rect.cuboid.half_extents[1]..line_rect.cuboid.half_extents[1];
            let range_dots_rot = -std::f64::consts::FRAC_PI_8..std::f64::consts::FRAC_PI_8;
            let range_dots_rx = options.radii[0] * 0.8..options.radii[0] * 1.25;
            let range_dots_ry = options.radii[1] * 0.8..options.radii[1] * 1.25;

            let distr_x = Uniform::from(range_x);
            let distr_dots_rot = Uniform::from(range_dots_rot);
            let distr_dots_rx = Uniform::from(range_dots_rx);
            let distr_dots_ry = Uniform::from(range_dots_ry);

            let n_dots = (area * 0.1 * options.density).round() as i32;

            let mut bez_path = kurbo::BezPath::new();

            for _ in 0..n_dots {
                let x_pos = distr_x.sample(&mut rng);
                let y_pos = options
                    .distribution
                    .sample_for_range_symmetrical_clipped(&mut rng, range_y.clone());

                let pos = line_rect.transform.affine * na::point![x_pos, y_pos];

                let rotation_angle = na::Rotation2::rotation_between(&na::Vector2::x(), &line_vec)
                    .angle()
                    + distr_dots_rot.sample(&mut rng);
                let radii = na::vector![
                    distr_dots_rx.sample(&mut rng),
                    distr_dots_ry.sample(&mut rng)
                ];

                let ellipse = kurbo::Ellipse::new(
                    kurbo::Point {
                        x: pos[0],
                        y: pos[1],
                    },
                    radii.to_kurbo_vec(),
                    rotation_angle,
                );

                bez_path.extend(ellipse.to_path(0.1));
            }

            bez_path
        };

        if let Some(fill_color) = options.stroke_color {
            // Outlines for debugging
            //let stroke_brush = cx.solid_brush(piet::Color::RED);
            //cx.stroke(segment.clone(), &stroke_brush, 0.4);
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(bez_path, &fill_brush);
        }
        cx.restore().unwrap();
    }
}

impl Composer<TexturedOptions> for Segment {
    fn composed_bounds(&self, options: &TexturedOptions) -> AABB {
        let max_pressure = if options.segment_constant_width {
            1.0
        } else {
            self.start().pressure.max(self.end().pressure)
        };

        self.bounds()
            .loosened(options.stroke_width * 0.5 * max_pressure)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &TexturedOptions) {
        cx.save().unwrap();
        match self {
            Self::Dot { .. } => {
                // Dont draw dots for textured segments. They stand out
            }
            Self::Line { start, end } => {
                let line = Line {
                    start: start.pos,
                    end: end.pos,
                };

                let options = if options.segment_constant_width {
                    options.clone()
                } else {
                    let mut options = options.clone();
                    // Line width with the mean of the start and end pressure
                    options.stroke_width =
                        options.stroke_width * ((start.pressure + end.pressure) * 0.5);
                    options
                };

                line.draw_composed(cx, &options);
            }
            Self::QuadBez { start, cp: _, end } => {
                let line = Line {
                    start: start.pos,
                    end: end.pos,
                };
                let options = if options.segment_constant_width {
                    options.clone()
                } else {
                    let mut options = options.clone();
                    // Line width with the mean of the start and end pressure
                    options.stroke_width =
                        options.stroke_width * ((start.pressure + end.pressure) * 0.5);
                    options
                };

                line.draw_composed(cx, &options);
            }
            Self::CubBez {
                start,
                cp1: _,
                cp2: _,
                end,
            } => {
                let line = Line {
                    start: start.pos,
                    end: end.pos,
                };
                let options = if options.segment_constant_width {
                    options.clone()
                } else {
                    let mut options = options.clone();
                    // Line width with the mean of the start and end pressure
                    options.stroke_width =
                        options.stroke_width * ((start.pressure + end.pressure) * 0.5);
                    options
                };

                line.draw_composed(cx, &options);
            }
        }
        cx.restore().unwrap();
    }
}

impl Composer<TexturedOptions> for PenPath {
    fn composed_bounds(&self, options: &TexturedOptions) -> AABB {
        self.iter()
            .map(|segment| segment.composed_bounds(options))
            .fold(AABB::new_invalid(), |acc, x| acc.merged(&x))
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &TexturedOptions) {
        cx.save().unwrap();
        let mut options = options.clone();

        for segment in self.iter() {
            options.seed = options.seed.map(|seed| crate::utils::seed_advance(seed));
            segment.draw_composed(cx, &options);
        }
        cx.restore().unwrap();
    }
}
