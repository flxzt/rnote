use crate::penpath::Element;
use crate::shapes::{Ellipse};
use crate::{PenEvent, Shape, Transform};

use super::ShapeBuilderBehaviour;


/// line builder
#[derive(Debug, Clone)]
pub struct EllipseBuilder {
    /// the start position
    pub start: na::Vector2<f64>,
    /// the current position
    pub current: na::Vector2<f64>,
}

impl ShapeBuilderBehaviour for EllipseBuilder {
    type BuildedShape = Shape;

    fn start(element: Element) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
        }
    }

    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>> {
        match event {
            PenEvent::Down { element, .. } => {
                self.current = element.pos;
            }
            PenEvent::Up { .. } => {
                return Some(vec![Shape::Ellipse(self.state_as_ellipse())]);
            }
            PenEvent::Proximity { .. } => {}
            PenEvent::Cancel => {}
        }

        None
    }
}

impl EllipseBuilder {
    /// The current state as rectangle
    pub fn state_as_ellipse(&self) -> Ellipse {
        let transform = Transform::new_w_isometry(na::Isometry2::new(self.start, 0.0));
        let radii = (self.current - self.start).abs();

        Ellipse {
            radii,
            transform
        }
    }
}