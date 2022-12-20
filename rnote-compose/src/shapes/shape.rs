use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

use super::{CubicBezier, Ellipse, Line, QuadraticBezier, Rectangle, ShapeBehaviour};
use crate::transform::TransformBehaviour;

// Container type to store shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shape")]
/// A Shape type, holding the actual shape inside it
pub enum Shape {
    #[serde(rename = "line")]
    /// A line shape
    Line(Line),
    #[serde(rename = "rect")]
    /// A rectangle shape
    Rectangle(Rectangle),
    #[serde(rename = "ellipse")]
    /// An ellipse shape
    Ellipse(Ellipse),
    #[serde(rename = "quadbez")]
    /// A quadratic bezier curve shape
    QuadraticBezier(QuadraticBezier),
    #[serde(rename = "cubbez")]
    /// A cubic bezier curve shape
    CubicBezier(CubicBezier),
}

impl Default for Shape {
    fn default() -> Self {
        Self::Line(Line::default())
    }
}

impl TransformBehaviour for Shape {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        match self {
            Self::Line(line) => {
                line.translate(offset);
            }
            Self::Rectangle(rectangle) => {
                rectangle.translate(offset);
            }
            Self::Ellipse(ellipse) => {
                ellipse.translate(offset);
            }
            Self::QuadraticBezier(quadbez) => {
                quadbez.translate(offset);
            }
            Self::CubicBezier(cubbez) => {
                cubbez.translate(offset);
            }
        }
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        match self {
            Self::Line(line) => {
                line.rotate(angle, center);
            }
            Self::Rectangle(rectangle) => {
                rectangle.rotate(angle, center);
            }
            Self::Ellipse(ellipse) => {
                ellipse.rotate(angle, center);
            }
            Self::QuadraticBezier(quadbez) => {
                quadbez.rotate(angle, center);
            }
            Self::CubicBezier(cubbez) => {
                cubbez.rotate(angle, center);
            }
        }
    }

    fn scale(&mut self, scale: nalgebra::Vector2<f64>) {
        match self {
            Self::Line(line) => {
                line.scale(scale);
            }
            Self::Rectangle(rectangle) => {
                rectangle.scale(scale);
            }
            Self::Ellipse(ellipse) => {
                ellipse.scale(scale);
            }
            Self::QuadraticBezier(quadbez) => {
                quadbez.scale(scale);
            }
            Self::CubicBezier(cubbez) => {
                cubbez.scale(scale);
            }
        }
    }
}

impl ShapeBehaviour for Shape {
    fn bounds(&self) -> Aabb {
        match self {
            Self::Line(line) => line.bounds(),
            Self::Rectangle(rectangle) => rectangle.bounds(),
            Self::Ellipse(ellipse) => ellipse.bounds(),
            Self::QuadraticBezier(quadbez) => quadbez.bounds(),
            Self::CubicBezier(cubbez) => cubbez.bounds(),
        }
    }
    fn hitboxes(&self) -> Vec<Aabb> {
        match self {
            Self::Line(line) => line.hitboxes(),
            Self::Rectangle(rectangle) => rectangle.hitboxes(),
            Self::Ellipse(ellipse) => ellipse.hitboxes(),
            Self::QuadraticBezier(quadbez) => quadbez.hitboxes(),
            Self::CubicBezier(cubbez) => cubbez.hitboxes(),
        }
    }
}
