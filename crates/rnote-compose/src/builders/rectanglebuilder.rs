// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::eventresult::EventPropagation;
use crate::penevent::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Rectangle;
use crate::style::{Composer, indicators};
use crate::{Constraints, EventResult};
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::glamx::DAffine2;
use p2d::math::Vector2;
use p2d::shape::Cuboid;
use std::time::Instant;
use vello_cpu::RenderContext;

/// Rectangle builder.
#[derive(Debug, Clone)]
pub struct RectangleBuilder {
    /// Start position.
    start: Vector2,
    /// Current position.
    current: Vector2,
}

impl BuilderCreator for RectangleBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
        }
    }
}

impl Buildable for RectangleBuilder {
    type Emit = Shape;

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>> {
        let progress = match event {
            PenEvent::Down { element, .. } => {
                self.current = constraints.constrain(element.pos - self.start) + self.start;
                BuilderProgress::InProgress
            }
            PenEvent::Up { .. } => {
                BuilderProgress::Finished(vec![Shape::Rectangle(self.state_as_rect())])
            }
            _ => BuilderProgress::InProgress,
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        Some(
            self.state_as_rect()
                .composed_bounds(style)
                .loosened(indicators::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw(&self, cx: &mut RenderContext, style: &Style, zoom: f64) {
        let rect = self.state_as_rect();
        rect.draw_composed(cx, style);

        indicators::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        indicators::draw_pos_indicator(cx, PenState::Down, self.current, zoom);
    }
}

impl RectangleBuilder {
    /// The current state as a rectangle.
    pub fn state_as_rect(&self) -> Rectangle {
        let center = (self.start + self.current) * 0.5;
        let affine = DAffine2::from_translation(center);
        let half_extents = (self.current - self.start) * 0.5;
        let cuboid = Cuboid::new(half_extents);

        Rectangle { cuboid, affine }
    }
}
