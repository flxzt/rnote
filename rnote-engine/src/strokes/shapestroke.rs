use super::StrokeBehaviour;
use crate::DrawBehaviour;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::shapes::{ Shape};
use rnote_compose::style::Composer;
use rnote_compose::transform::TransformBehaviour;
use rnote_compose::Style;

use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shapestroke")]
pub struct ShapeStroke {
    #[serde(rename = "shape")]
    pub shape: Shape,
    #[serde(rename = "style")]
    pub style: Style,
}

impl StrokeBehaviour for ShapeStroke {}

impl DrawBehaviour for ShapeStroke {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        match self.shape {
            Shape::Line(ref line) => match &self.style {
                Style::Smooth(options) => {
                    line.draw_composed(cx, options);
                }
                Style::Rough(options) => {
                    line.draw_composed(cx, options);
                }
                Style::Textured(_) => {}
            },
            Shape::Rectangle(ref rectangle) => match &self.style {
                Style::Smooth(options) => {
                    rectangle.draw_composed(cx, options);
                }
                Style::Rough(options) => {
                    rectangle.draw_composed(cx, options);
                }
                Style::Textured(_) => {}
            },
            Shape::Ellipse(ref ellipse) => match &self.style {
                Style::Smooth(options) => {
                    ellipse.draw_composed(cx, options);
                }
                Style::Rough(options) => {
                    ellipse.draw_composed(cx, options);
                }
                Style::Textured(_) => {}
            },
        };

        Ok(())
    }
}

impl ShapeBehaviour for ShapeStroke {
    fn bounds(&self) -> AABB {
        match &self.style {
            Style::Smooth(options) => self.shape.composed_bounds(options),
            Style::Rough(options) => self.shape.composed_bounds(options),
            Style::Textured(_) => self.shape.bounds(),
        }
    }
}

impl TransformBehaviour for ShapeStroke {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.shape.translate(offset);
    }
    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.shape.rotate(angle, center);
    }
    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        self.shape.scale(scale);
    }
}

impl ShapeStroke {
    pub fn new(shape: Shape, style: Style) -> Self {
        Self {
            shape,
            style,
        }
    }
}
