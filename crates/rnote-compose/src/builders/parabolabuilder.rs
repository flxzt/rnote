// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::eventresult::EventPropagation;
use crate::penevent::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Parabola;
use crate::style::{indicators, Composer};
use crate::{Constraints, EventResult};
use crate::{Shape, Style, Transform};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::shape::Cuboid;
use piet::RenderContext;
use std::time::Instant;

/// Parabola builder.
#[derive(Debug, Clone)]
pub struct ParabolaBuilder {
    /// Start position.
    start: na::Vector2<f64>,
    /// Current position.
    current: na::Vector2<f64>,
}

impl BuilderCreator for ParabolaBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
        }
    }
}

impl Buildable for ParabolaBuilder {
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
                BuilderProgress::Finished(vec![Shape::Parabola(self.state_as_parabola())])
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
            self.state_as_parabola()
                .composed_bounds(style)
                .loosened(indicators::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();
        let parabola = self.state_as_parabola();
        parabola.draw_composed(cx, style);

        indicators::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        indicators::draw_pos_indicator(cx, PenState::Down, self.current, zoom);
        cx.restore().unwrap();
    }
}

impl ParabolaBuilder {
    /// The current state as a parabola.
    pub fn state_as_parabola(&self) -> Parabola {
        let center = (self.start + self.current) * 0.5;
        let transform = Transform::new_w_isometry(na::Isometry2::new(center, 0.0));
        let half_extents = (self.current - self.start) * 0.5;
        let cuboid = Cuboid::new(half_extents);

        Parabola { cuboid, transform }
    }
}
