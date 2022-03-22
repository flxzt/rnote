use crate::shapes::ShapeBehaviour;
use crate::PenEvent;

pub trait ShapeBuilderBehaviour {
    type BuildedShape: ShapeBehaviour;

    /// Returns None if no shapes can be built in the current state
    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>>;
}
