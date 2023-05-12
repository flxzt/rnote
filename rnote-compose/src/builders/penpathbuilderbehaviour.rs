// Imports
use crate::penevents::PenEvent;
use crate::penpath::{Element, Segment};
use crate::Constraints;
use crate::Style;
use p2d::bounding_volume::Aabb;
use std::time::Instant;

#[derive(Debug, Clone)]
/// Builder progress.
pub enum PenPathBuilderProgress {
    /// In progress.
    InProgress,
    /// Emit new path segments, but continue.
    EmitContinue(Vec<Segment>),
    /// Done building.
    Finished(Vec<Segment>),
}

/// Creator of a pen path builder.
pub trait PenPathBuilderCreator {
    /// Start the builder.
    fn start(element: Element, now: Instant) -> Self;
}

/// Types that are pen path builders.
///
/// They receive pen events, and return path segments.
/// They usually are drawn while building the shape and are finite state machines.
pub trait PenPathBuilderBehaviour: std::fmt::Debug {
    /// Handle a pen event.
    ///
    /// Returns the builder progress.
    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        constraints: Constraints,
    ) -> PenPathBuilderProgress;

    /// Bounds.
    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb>;

    /// Draw with a style.
    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64);
}
