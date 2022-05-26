use p2d::bounding_volume::AABB;

use crate::penhelpers::PenEvent;
use crate::penpath::Element;
use crate::{Shape, Style};

#[derive(Debug, Clone)]
/// the builder progress
pub enum BuilderProgress {
    /// in progress
    InProgress,
    /// emits shapes, but continue
    EmitContinue(Vec<Shape>),
    /// done building
    Finished(Vec<Shape>),
}

/// Creates a shape builder (separate trait because we use the ShapeBuilderBehaviour as trait object, so we can't have a method returning Self there.)
pub trait ShapeBuilderCreator {
    /// Start the builder
    fn start(element: Element) -> Self;
}

/// Types that are shape builders.
/// They receive pen events, and return builded shapes. They usually are drawn while building the shape, and are state machines.
pub trait ShapeBuilderBehaviour: std::fmt::Debug {
    /// handles a pen event. Returns None if no shapes can be built in the current state.
    /// Returns the builder progress which can contain the builded shapes.
    fn handle_event(&mut self, event: PenEvent) -> BuilderProgress;

    /// the bounds
    fn bounds(&self, style: &Style, zoom: f64) -> AABB;

    /// draw with a style
    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64);
}
