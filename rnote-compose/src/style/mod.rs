mod composer;
/// Draw helpers
pub mod drawhelpers;
/// The rough module for rough styles
pub mod rough;
/// The smooth module for smooth styles
pub mod smooth;
/// The textured module for textured styles
pub mod textured;

// Re exports
use self::rough::RoughOptions;
use self::smooth::SmoothOptions;
use self::textured::TexturedOptions;
use anyhow::Context;
pub use composer::Composer;

use crate::shapes::{CubicBezier, Ellipse, Line, QuadraticBezier, Rectangle};
use crate::{PenPath, Shape};
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

    /// The margins for bounds in which the shape fits
    pub fn bounds_margin(&self) -> f64 {
        match self {
            Style::Smooth(options) => options.stroke_width,
            Style::Rough(options) => options.stroke_width + RoughOptions::ROUGH_BOUNDS_MARGIN,
            Style::Textured(options) => options.stroke_width,
        }
    }

    /// Advances the seed for styles that have one
    pub fn advance_seed(&mut self) {
        match self {
            Style::Smooth(_) => {}
            Style::Rough(options) => options.advance_seed(),
            Style::Textured(options) => options.advance_seed(),
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

impl Composer<Style> for PenPath {
    fn composed_bounds(&self, options: &Style) -> p2d::bounding_volume::AABB {
        match options {
            Style::Smooth(options) => self.composed_bounds(options),
            Style::Rough(_) => unimplemented!(),
            Style::Textured(options) => self.composed_bounds(options),
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match options {
            Style::Smooth(options) => self.draw_composed(cx, options),
            Style::Rough(_) => unimplemented!(),
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
        }
    }

    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &Style) {
        match self {
            Shape::Line(line) => line.draw_composed(cx, options),
            Shape::Rectangle(rectangle) => rectangle.draw_composed(cx, options),
            Shape::Ellipse(ellipse) => ellipse.draw_composed(cx, options),
            Shape::QuadraticBezier(quadratic_bezier) => quadratic_bezier.draw_composed(cx, options),
            Shape::CubicBezier(cubic_bezier) => cubic_bezier.draw_composed(cx, options),
        }
    }
}

/// The pressure curve used by some styles
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "pressure_curve")]
pub enum PressureCurve {
    /// Constant
    #[serde(rename = "const")]
    Const = 0,
    /// linear
    #[serde(rename = "linear")]
    Linear,
    /// square root
    #[serde(rename = "sqrt")]
    Sqrt,
    /// cubic root
    #[serde(rename = "cbrt")]
    Cbrt,
    /// quadratic polynomial
    #[serde(rename = "pow2")]
    Pow2,
    /// cubic polynomial
    #[serde(rename = "pow3")]
    Pow3,
}

impl Default for PressureCurve {
    fn default() -> Self {
        Self::Linear
    }
}

impl PressureCurve {
    /// Expects pressure to be between range 0.0 to 1.0
    pub fn apply(&self, width: f64, pressure: f64) -> f64 {
        match self {
            Self::Const => width,
            Self::Linear => width * pressure,
            Self::Sqrt => width * pressure.sqrt(),
            Self::Cbrt => width * pressure.cbrt(),
            Self::Pow2 => width * pressure.powi(2),
            Self::Pow3 => width * pressure.powi(3),
        }
    }
}

impl TryFrom<u32> for PressureCurve {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("PressureCurve try_from::<u32>() for value {value} failed"))
    }
}
