use crate::shapes::ShapeBehaviour;
use crate::PenEvent;

/// Types that are shape builders.
/// They receive pen events, and return builded shapes. They usually are drawn while building the shape, and are state machines. 
pub trait ShapeBuilderBehaviour {
    /// The type for shapes that are returned when they were built successfully
    type BuildedShape: ShapeBehaviour;

    /// handles a pen event. Returns None if no shapes can be built in the current state. Returns Some() when a /multiple shapes was/were successfully built.
    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>>;
}
