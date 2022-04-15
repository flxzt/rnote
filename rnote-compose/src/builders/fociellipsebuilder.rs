use crate::penpath::Element;
use crate::shapes::Ellipse;
use crate::{PenEvent, Shape};

use super::ShapeBuilderBehaviour;

#[derive(Debug, Clone)]
/// The state
pub enum FociEllipseBuilderState {
    /// first
    First(na::Vector2<f64>),
    /// foci
    Foci([na::Vector2<f64>; 2]),
    /// foci and point
    FociAndPoint {
        /// The foci
        foci: [na::Vector2<f64>; 2],
        /// the point
        point: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
/// building ellipse with foci and point
pub struct FociEllipseBuilder {
    /// the state
    pub state: FociEllipseBuilderState,
}

impl ShapeBuilderBehaviour for FociEllipseBuilder {
    type BuildedShape = Shape;

    fn start(element: Element) -> Self {
        Self {
            state: FociEllipseBuilderState::First(element.pos),
        }
    }

    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>> {
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        match (&mut self.state, event) {
            (FociEllipseBuilderState::First(first), PenEvent::Down { element, .. }) => {
                *first = element.pos;
            }
            (FociEllipseBuilderState::First(first), PenEvent::Up { element, .. }) => {
                self.state = FociEllipseBuilderState::Foci([*first, element.pos])
            }
            (FociEllipseBuilderState::First(_), _) => {}
            (FociEllipseBuilderState::Foci(foci), PenEvent::Down { element, .. }) => {
                foci[1] = element.pos;
            }
            (FociEllipseBuilderState::Foci(foci), PenEvent::Up { element, .. }) => {
                self.state = FociEllipseBuilderState::FociAndPoint {
                    foci: *foci,
                    point: element.pos,
                };
            }
            (FociEllipseBuilderState::Foci(_), _) => {}
            (
                FociEllipseBuilderState::FociAndPoint { foci: _, point },
                PenEvent::Down { element, .. },
            ) => {
                *point = element.pos;
            }
            (FociEllipseBuilderState::FociAndPoint { foci, point }, PenEvent::Up { .. }) => {
                let shape = Ellipse::from_foci_and_point(*foci, *point);

                return Some(vec![Shape::Ellipse(shape)]);
            }
            (FociEllipseBuilderState::FociAndPoint { .. }, _) => {}
        }
        None
    }
}
