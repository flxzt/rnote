// Modules
mod textureddotsdistribution;
mod texturedoptions;

// Re-exports
pub use textureddotsdistribution::TexturedDotsDistribution;
pub use texturedoptions::TexturedOptions;

// Imports
use super::Composer;
use crate::ext::Vector2Ext;
use crate::penpath::Segment;
use crate::shapes::{Line, Shapeable};
use crate::PenPath;
use kurbo::Shape;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rand_distr::{Distribution, Uniform};

impl Composer<TexturedOptions> for Line {
    fn composed_bounds(&self, options: &TexturedOptions) -> Aabb {
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

            // Radii scale with the stroke width, with a weight.
            let dots_radii = TexturedOptions::DOTS_RADII_DEFAULT
                * (1.0 + options.stroke_width * TexturedOptions::STROKE_WIDTH_RADII_WEIGHT);

            // Ranges for randomization
            let range_x = -line_rect.cuboid.half_extents[0]..line_rect.cuboid.half_extents[0];
            let range_y = -line_rect.cuboid.half_extents[1]..line_rect.cuboid.half_extents[1];
            let range_dots_rot = -std::f64::consts::FRAC_PI_8..std::f64::consts::FRAC_PI_8;
            let range_dots_rx = dots_radii[0] * 0.8..dots_radii[0] * 1.25;
            let range_dots_ry = dots_radii[1] * 0.8..dots_radii[1] * 1.25;

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
            let fill_brush = cx.solid_brush(fill_color.into());
            cx.fill(bez_path, &fill_brush);
        }
        cx.restore().unwrap();
    }
}

impl Composer<TexturedOptions> for PenPath {
    fn composed_bounds(&self, options: &TexturedOptions) -> Aabb {
        self.bounds().loosened(options.stroke_width)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &TexturedOptions) {
        let mut options = options.clone();
        let mut prev = self.start;

        cx.save().unwrap();

        for seg in self.segments.iter() {
            if seg.end().pos == self.start.pos {
                continue;
            }

            match seg {
                Segment::LineTo { end } => {
                    let line = Line {
                        start: prev.pos,
                        end: end.pos,
                    };

                    let mut options = options.clone();

                    options.stroke_width = options
                        .pressure_curve
                        .apply(options.stroke_width, (prev.pressure + end.pressure) * 0.5);

                    line.draw_composed(cx, &options);
                    prev = *end;
                }
                Segment::QuadBezTo { end, .. } => {
                    let line = Line {
                        start: prev.pos,
                        end: end.pos,
                    };

                    let mut options = options.clone();

                    options.stroke_width = options
                        .pressure_curve
                        .apply(options.stroke_width, (prev.pressure + end.pressure) * 0.5);

                    line.draw_composed(cx, &options);
                    prev = *end;
                }
                Segment::CubBezTo { end, .. } => {
                    let line = Line {
                        start: prev.pos,
                        end: end.pos,
                    };

                    let mut options = options.clone();

                    options.stroke_width = options
                        .pressure_curve
                        .apply(options.stroke_width, (prev.pressure + end.pressure) * 0.5);

                    line.draw_composed(cx, &options);
                    prev = *end;
                }
            }

            options.advance_seed();
        }

        cx.restore().unwrap();
    }
}
