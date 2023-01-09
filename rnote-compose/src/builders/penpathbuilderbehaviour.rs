use std::time::Instant;

use p2d::bounding_volume::Aabb;

use crate::penevents::PenEvent;
use crate::penpath::{Element, Segment};
use crate::Style;

use super::Constraints;

#[derive(Debug, Clone)]
/// the builder progress
pub enum PenPathBuilderProgress {
    /// in progress
    InProgress,
    /// emits new path segments, but continue
    EmitContinue(Vec<Segment>),
    /// done building
    Finished(Vec<Segment>),
}

/// Creates a pen path builder
pub trait PenPathBuilderCreator {
    /// Start the builder
    fn start(element: Element, now: Instant) -> Self;
}

/// Types that are pen path builders.
/// They receive pen events, and return path segments. They usually are drawn while building the shape, and are state machines.
pub trait PenPathBuilderBehaviour: std::fmt::Debug {
    /// handles a pen event.
    /// Returns the builder progress.
    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        constraints: Constraints,
    ) -> PenPathBuilderProgress;

    /// the bounds
    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb>;

    /// draw with a style
    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64);
}
