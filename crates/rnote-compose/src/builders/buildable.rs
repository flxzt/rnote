// Imports
use crate::penpath::Element;
use crate::PenEvent;
use crate::Style;
use crate::{Constraints, EventResult};
use p2d::bounding_volume::Aabb;
use std::time::Instant;

#[derive(Debug, Clone)]
/// Builder progress.
pub enum BuilderProgress<T> {
    /// In progress.
    InProgress,
    /// Emit but continue.
    EmitContinue(Vec<T>),
    /// Done building.
    Finished(Vec<T>),
}

/// Creator of a builder.
pub trait BuilderCreator {
    /// Start the builder.
    fn start(element: Element, now: Instant) -> Self;
}

/// Types that are builders.
///
/// They receive pen events, and return the associated `Emit` type.
/// They usually are drawn while building and are finite state machines.
pub trait Buildable: std::fmt::Debug {
    /// The type that is emitted by the builder.
    type Emit: std::fmt::Debug;

    /// Handle a pen event.
    ///
    /// Returns the builder progress.
    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>>;

    /// Bounds.
    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb>;

    /// Draw with a style.
    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64);
}
