// Imports
use crate::penevents::PenEvent;
use crate::penpath::Element;
use crate::Constraints;
use crate::{Shape, Style};
use p2d::bounding_volume::Aabb;
use std::time::Instant;

#[derive(Debug, Clone)]
/// Builder progress.
pub enum ShapeBuilderProgress {
    /// In progress.
    InProgress,
    /// Emit shapes, but continue.
    EmitContinue(Vec<Shape>),
    /// Done building.
    Finished(Vec<Shape>),
}

/// Creator for a shape builder.
///
/// This needs to be a separate trait because ShapeBuilderBehaviour is used as trait object,
/// so we can't have a method on it returning `Self`.
pub trait ShapeBuilderCreator {
    /// Start the builder.
    fn start(element: Element, now: Instant) -> Self;
}

/// Types that are shape builders.
///
/// They receive pen events, and return built shapes.
/// They are usually drawn while building the shape, and are finite state machines.
pub trait ShapeBuilderBehaviour: std::fmt::Debug {
    /// Handle a pen event.
    ///
    /// Returns the builder progress.
    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        constraints: Constraints,
    ) -> ShapeBuilderProgress;

    /// Bounds.
    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb>;

    /// Draw with a style.
    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64);
}
