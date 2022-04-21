use crate::penhelpers::PenEvent;
use crate::penpath::Element;
use crate::shapes::CubicBezier;
use crate::Shape;

use super::ShapeBuilderBehaviour;

#[derive(Debug, Clone)]
/// The state
pub enum CubBezBuilderState {
    /// start
    Start(na::Vector2<f64>),
    /// first control point
    Cp1 {
        /// start
        start: na::Vector2<f64>,
        /// first control point
        cp1: na::Vector2<f64>,
    },
    /// second control point
    Cp2 {
        /// start
        start: na::Vector2<f64>,
        /// first control point
        cp1: na::Vector2<f64>,
        /// second control point
        cp2: na::Vector2<f64>,
    },
    /// end
    End {
        /// start
        start: na::Vector2<f64>,
        /// first control point
        cp1: na::Vector2<f64>,
        /// second control point
        cp2: na::Vector2<f64>,
        /// end
        end: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
/// building cubic bezier
pub struct CubBezBuilder {
    /// the state
    pub state: CubBezBuilderState,
}

impl ShapeBuilderBehaviour for CubBezBuilder {
    type BuildedShape = Shape;

    fn start(element: Element) -> Self {
        Self {
            state: CubBezBuilderState::Start(element.pos),
        }
    }

    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>> {
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        match (&mut self.state, event) {
            (CubBezBuilderState::Start(start), PenEvent::Down { element, .. }) => {
                *start = element.pos;

                self.state = CubBezBuilderState::Cp1 {
                    start: *start,
                    cp1: element.pos,
                };
            }
            (CubBezBuilderState::Start(start), PenEvent::Up { element, .. }) => {
                // should not be reachable, but just in case we transition here too
                self.state = CubBezBuilderState::Cp1 {
                    start: *start,
                    cp1: element.pos,
                };
            }
            (CubBezBuilderState::Start(_), ..) => {}
            (CubBezBuilderState::Cp1 { cp1, .. }, PenEvent::Down { element, .. }) => {
                *cp1 = element.pos;
            }
            (CubBezBuilderState::Cp1 { start, cp1 }, PenEvent::Up { element, .. }) => {
                self.state = CubBezBuilderState::Cp2 {
                    start: *start,
                    cp1: *cp1,
                    cp2: element.pos,
                };
            }
            (CubBezBuilderState::Cp1 { .. }, ..) => {}
            (CubBezBuilderState::Cp2 { cp2, .. }, PenEvent::Down { element, .. }) => {
                *cp2 = element.pos;
            }
            (CubBezBuilderState::Cp2 { start, cp1, cp2 }, PenEvent::Up { element, .. }) => {
                self.state = CubBezBuilderState::End {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: element.pos,
                };
            }
            (CubBezBuilderState::Cp2 { .. }, ..) => {}
            (CubBezBuilderState::End { end, .. }, PenEvent::Down { element, .. }) => {
                *end = element.pos;
            }
            (
                CubBezBuilderState::End {
                    start,
                    cp1,
                    cp2,
                    end,
                },
                PenEvent::Up { .. },
            ) => {
                return Some(vec![Shape::CubicBezier(CubicBezier {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: *end,
                })]);
            }
            (CubBezBuilderState::End { .. }, ..) => {}
        }
        None
    }
}
