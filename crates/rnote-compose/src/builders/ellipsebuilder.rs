// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::eventresult::EventPropagation;
use crate::penevent::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Ellipse;
use crate::style::{Composer, indicators};
use crate::{Constraints, EventResult};
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::glamx::DAffine2;
use p2d::math::Vector2;
use piet::RenderContext;
use std::time::Instant;

/// Ellipse builder.
#[derive(Debug, Clone)]
pub struct EllipseBuilder {
    /// Start position.
    start: Vector2,
    /// Current position.
    current: Vector2,
}

impl BuilderCreator for EllipseBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
        }
    }
}

impl Buildable for EllipseBuilder {
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
                BuilderProgress::Finished(vec![Shape::Ellipse(self.state_as_ellipse())])
            }
            _ => BuilderProgress::InProgress,
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
        }
    }

    fn bounds(&self, style: &crate::Style, zoom: f64) -> Option<Aabb> {
        Some(
            self.state_as_ellipse()
                .composed_bounds(style)
                .loosened(indicators::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();
        let ellipse = self.state_as_ellipse();
        ellipse.draw_composed(cx, style);

        indicators::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        indicators::draw_pos_indicator(cx, PenState::Down, self.current, zoom);
        cx.restore().unwrap();
    }
}

impl EllipseBuilder {
    /// The current state as an ellipse.
    pub fn state_as_ellipse(&self) -> Ellipse {
        let affine = DAffine2::from_translation(self.start);
        let radii = (self.current - self.start).abs();

        Ellipse { radii, affine }
    }
}
