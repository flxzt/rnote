// Modules
/// The module for the rough style.
pub mod roughoptions;

// Re-exports
pub use roughoptions::RoughOptions;

// Imports
use super::Composer;
use crate::Color;
use crate::ext::Vector2Ext;
use crate::shapes::{
    Arrow, CubicBezier, Ellipse, Line, Parabola, Polygon, Polyline, QuadraticBezier, Rectangle,
    Shapeable,
};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use roughr::Point2D;

fn generate_roughr_options(options: &RoughOptions) -> roughr::core::Options {
    let mut roughr_options = roughr::core::OptionsBuilder::default();

    roughr_options
        .stroke_width(options.stroke_width as f32)
        .hachure_angle(options.hachure_angle.to_degrees() as f32)
        .fill_style(options.fill_style.into());

    if let Some(seed) = options.seed {
        roughr_options.seed(seed);
    }

    if let Some(stroke_color) = options.stroke_color {
        roughr_options.stroke(stroke_color.into());
    }

    if let Some(fill_color) = options.fill_color {
        if fill_color != Color::TRANSPARENT {
            roughr_options
                .fill(fill_color.into())
                .fill_style(options.fill_style.into());
        }
    }

    roughr_options.build().unwrap()
}

// Composer implementations

impl Composer<RoughOptions> for Line {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();

        let drawable = rough_piet::KurboGenerator::new(generate_roughr_options(options)).line(
            self.start[0],
            self.start[1],
            self.end[0],
            self.end[1],
        );

        drawable.draw(cx);

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for Arrow {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::Aabb {
        self.internal_compute_bounds(Some(options.stroke_width))
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();

        let arrow_stem = rough_piet::KurboGenerator::new(generate_roughr_options(options)).line(
            self.start[0],
            self.start[1],
            self.tip[0],
            self.tip[1],
        );

        let tip = {
            let lline = {
                let lline = self
                    .compute_lline(Some(options.stroke_width))
                    .to_kurbo_point();
                Point2D::new(lline.x, lline.y)
            };
            let rline = {
                let rline = self.compute_rline(Some(options.stroke_width));
                Point2D::new(rline.x, rline.y)
            };
            let tip = {
                let tip = self.tip.to_kurbo_point();
                Point2D::new(tip.x, tip.y)
            };

            rough_piet::KurboGenerator::new(generate_roughr_options(options))
                .linear_path(&[lline, tip, rline], false)
        };

        arrow_stem.draw(cx);
        tip.draw(cx);

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for Rectangle {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();

        let top_left = -self.cuboid.half_extents;
        let size = self.cuboid.half_extents * 2.0;

        let drawable = rough_piet::KurboGenerator::new(generate_roughr_options(options)).rectangle(
            top_left[0],
            top_left[1],
            size[0],
            size[1],
        );

        cx.transform(self.transform.to_kurbo());
        drawable.draw(cx);

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for Ellipse {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();

        let size = self.radii * 2.0;

        let drawable = rough_piet::KurboGenerator::new(generate_roughr_options(options))
            .ellipse(0.0, 0.0, size[0], size[1]);

        cx.transform(self.transform.to_kurbo());
        drawable.draw(cx);

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for QuadraticBezier {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();

        let drawable = rough_piet::KurboGenerator::new(generate_roughr_options(options))
            .bezier_quadratic(
                roughr::Point2D::new(self.start[0] as f32, self.start[1] as f32),
                roughr::Point2D::new(self.cp[0] as f32, self.cp[1] as f32),
                roughr::Point2D::new(self.end[0] as f32, self.end[1] as f32),
            );

        drawable.draw(cx);

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for CubicBezier {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();

        let drawable = rough_piet::KurboGenerator::new(generate_roughr_options(options))
            .bezier_cubic(
                roughr::Point2D::new(self.start[0] as f32, self.start[1] as f32),
                roughr::Point2D::new(self.cp1[0] as f32, self.cp1[1] as f32),
                roughr::Point2D::new(self.cp2[0] as f32, self.cp2[1] as f32),
                roughr::Point2D::new(self.end[0] as f32, self.end[1] as f32),
            );

        drawable.draw(cx);

        cx.restore().unwrap();
    }
}

impl Composer<RoughOptions> for Polyline {
    fn composed_bounds(&self, options: &RoughOptions) -> Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        let points: Vec<roughr::Point2D<_, _>> = std::iter::once(roughr::Point2D::new(
            self.start[0] as f32,
            self.start[1] as f32,
        ))
        .chain(
            self.path
                .iter()
                .map(|p| roughr::Point2D::new(p[0] as f32, p[1] as f32)),
        )
        .collect();

        let drawable = rough_piet::KurboGenerator::new(generate_roughr_options(options))
            .linear_path(&points, false);

        drawable.draw(cx);
    }
}

impl Composer<RoughOptions> for Polygon {
    fn composed_bounds(&self, options: &RoughOptions) -> Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        let points: Vec<roughr::Point2D<_, _>> = std::iter::once(roughr::Point2D::new(
            self.start[0] as f32,
            self.start[1] as f32,
        ))
        .chain(
            self.path
                .iter()
                .map(|p| roughr::Point2D::new(p[0] as f32, p[1] as f32)),
        )
        .collect();

        let drawable =
            rough_piet::KurboGenerator::new(generate_roughr_options(options)).polygon(&points);

        drawable.draw(cx);
    }
}

impl Composer<RoughOptions> for crate::Shape {
    fn composed_bounds(&self, options: &RoughOptions) -> Aabb {
        match self {
            crate::Shape::Arrow(arrow) => arrow.composed_bounds(options),
            crate::Shape::Line(line) => line.composed_bounds(options),
            crate::Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            crate::Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.composed_bounds(options),
            crate::Shape::CubicBezier(cubbez) => cubbez.composed_bounds(options),
            crate::Shape::Polyline(polyline) => polyline.composed_bounds(options),
            crate::Shape::Polygon(polygon) => polygon.composed_bounds(options),
            crate::Shape::Parabola(parabola) => parabola.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        match self {
            crate::Shape::Arrow(arrow) => arrow.draw_composed(cx, options),
            crate::Shape::Line(line) => line.draw_composed(cx, options),
            crate::Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            crate::Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
            crate::Shape::QuadraticBezier(quadbez) => quadbez.draw_composed(cx, options),
            crate::Shape::CubicBezier(cubbez) => cubbez.draw_composed(cx, options),
            crate::Shape::Polyline(polyline) => polyline.draw_composed(cx, options),
            crate::Shape::Polygon(polygon) => polygon.draw_composed(cx, options),
            crate::Shape::Parabola(parabola) => parabola.draw_composed(cx, options),
        }
    }
}

impl Composer<RoughOptions> for Parabola {
    fn composed_bounds(&self, options: &RoughOptions) -> p2d::bounding_volume::Aabb {
        self.bounds()
            .loosened(options.stroke_width * 0.5 + RoughOptions::ROUGH_BOUNDS_MARGIN)
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &RoughOptions) {
        cx.save().unwrap();

        let tl = self.transform.affine
            * na::point![-self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let tr = self.transform.affine
            * na::point![self.cuboid.half_extents[0], -self.cuboid.half_extents[1]];
        let bm = self.transform.affine * na::point![0.0, 3.0 * self.cuboid.half_extents[1]];

        let drawable = rough_piet::KurboGenerator::new(generate_roughr_options(options))
            .bezier_quadratic(
                roughr::Point2D::new(tl[0] as f32, tl[1] as f32),
                roughr::Point2D::new(bm[0] as f32, bm[1] as f32),
                roughr::Point2D::new(tr[0] as f32, tr[1] as f32),
            );

        drawable.draw(cx);

        cx.restore().unwrap();
    }
}
