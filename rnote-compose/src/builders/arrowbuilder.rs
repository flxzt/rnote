// Imports
use super::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use super::ShapeBuilderBehaviour;
use crate::constraints::ConstraintRatio;
use crate::penevents::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Arrow;
use crate::style::{indicators, Composer};
use crate::Constraints;
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use std::time::Instant;

/// Arrow builder.
#[derive(Debug, Clone)]
pub struct ArrowBuilder {
    /// Start position.
    start: na::Vector2<f64>,
    /// Position of the tip.
    tip: na::Vector2<f64>,
}

impl ShapeBuilderCreator for ArrowBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            start: element.pos,
            tip: element.pos,
        }
    }
}

impl ShapeBuilderBehaviour for ArrowBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        mut constraints: Constraints,
    ) -> ShapeBuilderProgress {
        // we always want to allow horizontal and vertical constraints while building an arrow
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        match event {
            PenEvent::Down { element, .. } => {
                self.tip = constraints.constrain(element.pos - self.start) + self.start;
            }
            PenEvent::Up { .. } => {
                return ShapeBuilderProgress::Finished(vec![Shape::Arrow(self.state_as_arrow())]);
            }
            _ => {}
        }

        ShapeBuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        Some(
            self.state_as_arrow()
                .composed_bounds(style)
                .loosened(indicators::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();
        let arrow = self.state_as_arrow();
        arrow.draw_composed(cx, style);

        indicators::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        indicators::draw_pos_indicator(cx, PenState::Down, self.tip, zoom);
        cx.restore().unwrap();
    }
}

impl ArrowBuilder {
    /// Returns a configured arrow by the current state of the builder.
    pub fn state_as_arrow(&self) -> Arrow {
        Arrow::new(self.start, self.tip)
    }
}
