use crate::penpath::Element;
use crate::shapes::Line;
use crate::{PenEvent, Shape};

use super::ShapeBuilderBehaviour;

/// line builder
#[derive(Debug, Clone)]
pub struct LineBuilder {
    /// the start position
    pub start: na::Vector2<f64>,
    /// the current position
    pub current: na::Vector2<f64>,
}

impl ShapeBuilderBehaviour for LineBuilder {
    type BuildedShape = Shape;

    fn start(element: Element) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
        }
    }

    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>> {
        match event {
            crate::PenEvent::Down { element, .. } => {
                self.current = element.pos;
            }
            crate::PenEvent::Up { .. } => {
                return Some(vec![Shape::Line(Line {
                    start: self.start,
                    end: self.current,
                })]);
            }
            crate::PenEvent::Proximity { .. } => {}
            crate::PenEvent::Cancel => {}
        }

        None
    }
}

impl LineBuilder {
    /// The current state as line
    pub fn state_as_line(&self) -> Line {
        Line {
            start: self.start,
            end: self.current,
        }
    }
}
