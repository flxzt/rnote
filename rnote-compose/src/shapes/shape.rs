use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::{Ellipse, Line, Rectangle, ShapeBehaviour};
use crate::transform::TransformBehaviour;

// Container type to store shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shape")]
/// A Shape type, holding the actual shape inside it
pub enum Shape {
    #[serde(rename = "line")]
    /// A line shape
    Line(Line),
    #[serde(rename = "rectangle")]
    /// A rectangle shape
    Rectangle(Rectangle),
    #[serde(rename = "ellipse")]
    /// An ellipse shape
    Ellipse(Ellipse),
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
        }
    }
}

impl ShapeBehaviour for Shape {
    fn bounds(&self) -> AABB {
        match self {
            Self::Line(line) => line.bounds(),
            Self::Rectangle(rectangle) => rectangle.bounds(),
            Self::Ellipse(ellipse) => ellipse.bounds(),
        }
    }
}
