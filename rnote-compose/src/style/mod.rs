mod composer;
/// Draw helpers
pub mod drawhelpers;
/// The rough module for rough styles
pub mod rough;
/// The smooth module for smooth styles
pub mod smooth;
/// The textured module for textured styles
pub mod textured;

use crate::penpath::Segment;
use crate::shapes::{CubicBezier, Ellipse, Line, QuadraticBezier, Rectangle};
use crate::{PenPath, Shape};

// Re exports
use self::rough::RoughOptions;
use self::smooth::SmoothOptions;
use self::textured::TexturedOptions;
pub use composer::Composer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A style choice holding the style options inside its variants
#[serde(rename = "style")]
pub enum Style {
    /// A smooth style
    #[serde(rename = "smooth")]
    Smooth(SmoothOptions),
    /// A rough style
    #[serde(rename = "rough")]
    Rough(RoughOptions),
    /// A textured style
    #[serde(rename = "textured")]
    Textured(TexturedOptions),
}

impl Default for Style {
    fn default() -> Self {
        Self::Smooth(SmoothOptions::default())
    }
}

impl Style {
    /// returns the stroke width. available on all styles
    pub fn stroke_width(&self) -> f64 {
        match self {
            Style::Smooth(options) => options.stroke_width,
            Style::Rough(options) => options.stroke_width,
            Style::Textured(options) => options.stroke_width,
        }
    }
}

impl Composer<Style> for Line {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(options) => self.composed_bounds(options),
            Style::Textured(options) => self.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(options) => self.draw_composed(cx, options),
            Style::Textured(options) => self.draw_composed(cx, options),
        }
    }
}

impl Composer<Style> for Rectangle {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(options) => self.composed_bounds(options),
            Style::Textured(_options) => unimplemented!(),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(options) => self.draw_composed(cx, options),
            Style::Textured(_options) => unimplemented!(),
        }
    }
}

impl Composer<Style> for Ellipse {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(options) => self.composed_bounds(options),
            Style::Textured(_options) => unimplemented!(),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(options) => self.draw_composed(cx, options),
            Style::Textured(_options) => unimplemented!(),
        }
    }
}

impl Composer<Style> for QuadraticBezier {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(options) => self.composed_bounds(options),
            Style::Textured(_options) => unimplemented!(),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(options) => self.draw_composed(cx, options),
            Style::Textured(_options) => unimplemented!(),
        }
    }
}

impl Composer<Style> for CubicBezier {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(options) => self.composed_bounds(options),
            Style::Textured(_options) => unimplemented!(),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(options) => self.draw_composed(cx, options),
            Style::Textured(_options) => unimplemented!(),
        }
    }
}

impl Composer<Style> for Segment {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(options) => self.composed_bounds(options),
            Style::Textured(options) => self.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(options) => self.draw_composed(cx, options),
            Style::Textured(options) => self.draw_composed(cx, options),
        }
    }
}

impl Composer<Style> for PenPath {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(options) => self.composed_bounds(options),
            Style::Textured(options) => self.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(options) => self.draw_composed(cx, options),
            Style::Textured(options) => self.draw_composed(cx, options),
        }
    }
}

impl Composer<Style> for Shape {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match self {
            Shape::Line(line) => line.composed_bounds(options),
            Shape::Rectangle(rectangle) => rectangle.composed_bounds(options),
            Shape::Ellipse(ellipse) => ellipse.composed_bounds(options),
            Shape::QuadraticBezier(quadratic_bezier) => quadratic_bezier.composed_bounds(options),
            Shape::CubicBezier(cubic_bezier) => cubic_bezier.composed_bounds(options),
            Shape::Segment(segment) => segment.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match self {
            Shape::Line(line) => line.draw_composed(cx, options),
            Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
            Shape::QuadraticBezier(quadratic_bezier) => quadratic_bezier.draw_composed(cx, options),
            Shape::CubicBezier(cubic_bezier) => cubic_bezier.draw_composed(cx, options),
            Shape::Segment(segment) => segment.draw_composed(cx, options),
        }
    }
}
