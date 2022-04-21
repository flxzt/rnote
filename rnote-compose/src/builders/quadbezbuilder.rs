use crate::penhelpers::PenEvent;
use crate::penpath::Element;
use crate::shapes::QuadraticBezier;
use crate::Shape;

use super::ShapeBuilderBehaviour;

#[derive(Debug, Clone)]
/// The state
pub enum QuadBezBuilderState {
    /// start
    Start(na::Vector2<f64>),
    /// control point
    Cp {
        /// start
        start: na::Vector2<f64>,
        /// control point
        cp: na::Vector2<f64>,
    },
    /// end
    End {
        /// start
        start: na::Vector2<f64>,
        /// control point
        cp: na::Vector2<f64>,
        /// end
        end: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
/// building quadratic bezier
pub struct QuadBezBuilder {
    /// the state
    pub state: QuadBezBuilderState,
}

impl ShapeBuilderBehaviour for QuadBezBuilder {
    type BuildedShape = Shape;

    fn start(element: Element) -> Self {
        Self {
            state: QuadBezBuilderState::Start(element.pos),
        }
    }

    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>> {
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        match (&mut self.state, event) {
            (QuadBezBuilderState::Start(start), PenEvent::Down { element, .. }) => {
                *start = element.pos;

                self.state = QuadBezBuilderState::Cp {
                    start: *start,
                    cp: element.pos,
                };
            }
            (QuadBezBuilderState::Start(start), PenEvent::Up { element, .. }) => {
                // should not be reachable, but just in case we transition here too
                self.state = QuadBezBuilderState::Cp {
                    start: *start,
                    cp: element.pos,
                };
            }
            (QuadBezBuilderState::Start(_), ..) => {}
            (QuadBezBuilderState::Cp { cp, .. }, PenEvent::Down { element, .. }) => {
                *cp = element.pos;
            }
            (QuadBezBuilderState::Cp { start, cp }, PenEvent::Up { element, .. }) => {
                self.state = QuadBezBuilderState::End {
                    start: *start,
                    cp: *cp,
                    end: element.pos,
                };
            }
            (QuadBezBuilderState::Cp { .. }, ..) => {}
            (QuadBezBuilderState::End { end, .. }, PenEvent::Down { element, .. }) => {
                *end = element.pos;
            }
            (QuadBezBuilderState::End { start, cp, end }, PenEvent::Up { .. }) => {
                return Some(vec![Shape::QuadraticBezier(QuadraticBezier {
                    start: *start,
                    cp: *cp,
                    end: *end,
                })]);
            }
            (QuadBezBuilderState::End { .. }, ..) => {}
        }
        None
    }
}
