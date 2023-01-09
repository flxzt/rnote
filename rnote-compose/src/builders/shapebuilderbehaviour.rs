use std::time::Instant;

use p2d::bounding_volume::Aabb;

use crate::penevents::PenEvent;
use crate::penpath::Element;
use crate::{Shape, Style};

use super::Constraints;

#[derive(Debug, Clone)]
/// the builder progress
pub enum ShapeBuilderProgress {
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
    fn start(element: Element, now: Instant) -> Self;
}

/// Types that are shape builders.
/// They receive pen events, and return built shapes. They usually are drawn while building the shape, and are state machines.
pub trait ShapeBuilderBehaviour: std::fmt::Debug {
    /// handles a pen event.
    /// Returns the builder progress.
    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        constraints: Constraints,
    ) -> ShapeBuilderProgress;

    /// the bounds
    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb>;

    /// draw with a style
    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64);
}
