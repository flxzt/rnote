use p2d::shape::Cuboid;

use crate::penpath::Element;
use crate::shapes::Rectangle;
use crate::{PenEvent, Shape, Transform};

use super::ShapeBuilderBehaviour;

/// rect builder
#[derive(Debug, Clone)]
pub struct RectangleBuilder {
    /// the start position
    pub start: na::Vector2<f64>,
    /// the current position
    pub current: na::Vector2<f64>,
}

impl ShapeBuilderBehaviour for RectangleBuilder {
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
                return Some(vec![Shape::Rectangle(self.state_as_rect())]);
            }
            PenEvent::Proximity { .. } => {}
            PenEvent::Cancel => {}
        }

        None
    }
}

impl RectangleBuilder {
    /// The current state as rectangle
    pub fn state_as_rect(&self) -> Rectangle {
        let center = (self.start + self.current) / 2.0;
        let transform = Transform::new_w_isometry(na::Isometry2::new(center, 0.0));
        let half_extents = (self.current - self.start) / 2.0;
        let cuboid = Cuboid::new(half_extents);

        Rectangle { cuboid, transform }
    }
}
