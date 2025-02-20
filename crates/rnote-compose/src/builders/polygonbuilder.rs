// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::constraints::ConstraintRatio;
use crate::eventresult::EventPropagation;
use crate::penevent::{KeyboardKey, PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Polygon;
use crate::style::{Composer, indicators};
use crate::{Constraints, EventResult};
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use std::time::Instant;

/// Polygon builder.
#[derive(Debug, Clone)]
pub struct PolygonBuilder {
    /// Start position.
    start: na::Vector2<f64>,
    /// Position of the next/current path segment.
    current: na::Vector2<f64>,
    /// Path.
    path: Vec<na::Vector2<f64>>,
    /// Pen state.
    pen_state: PenState,
    /// Pen position.
    pen_pos: na::Vector2<f64>,
    /// Finish the polygon on the next `PenEvent::Up`.
    finish: bool,
}

impl BuilderCreator for PolygonBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
            path: Vec::new(),
            pen_state: PenState::Down,
            pen_pos: element.pos,
            finish: false,
        }
    }
}

impl Buildable for PolygonBuilder {
    type Emit = Shape;

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        mut constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>> {
        // we always want to allow horizontal and vertical constraints while building a polygon
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        let progress = match event {
            PenEvent::Down { element, .. } => {
                if (self.pen_state == PenState::Up || self.pen_state == PenState::Proximity)
                    && self.pos_in_finish(element.pos)
                {
                    self.finish = true;
                }
                self.pen_state = PenState::Down;
                self.pen_pos = element.pos;
                let last_pos = self.path.last().copied().unwrap_or(self.start);
                self.current = constraints.constrain(element.pos - last_pos) + last_pos;
                BuilderProgress::InProgress
            }
            PenEvent::Up { element, .. } => {
                if self.finish {
                    BuilderProgress::Finished(vec![Shape::Polygon(self.state_as_polygon())])
                } else {
                    if self.pen_state == PenState::Down {
                        self.path.push(self.current);
                    }
                    self.pen_state = PenState::Up;
                    self.pen_pos = element.pos;
                    BuilderProgress::InProgress
                }
            }
            PenEvent::Proximity { element, .. } => {
                self.pen_state = PenState::Proximity;
                self.pen_pos = element.pos;
                BuilderProgress::InProgress
            }
            PenEvent::KeyPressed { keyboard_key, .. } => match keyboard_key {
                KeyboardKey::Escape | KeyboardKey::CarriageReturn | KeyboardKey::Linefeed => {
                    BuilderProgress::Finished(vec![Shape::Polygon(self.state_as_polygon())])
                }
                _ => BuilderProgress::InProgress,
            },
            PenEvent::Text { .. } => BuilderProgress::InProgress,
            PenEvent::Cancel => {
                self.pen_state = PenState::Up;
                self.finish = false;
                BuilderProgress::Finished(vec![])
            }
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        let mut polygon = self.state_as_polygon();
        if !self.finish {
            polygon.path.push(self.current);
        }
        Some(
            polygon
                .composed_bounds(style)
                .loosened(indicators::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();

        let mut polygon = self.state_as_polygon();
        if !self.finish {
            polygon.path.push(self.current);
        }

        polygon.draw_composed(cx, style);
        indicators::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        if !self.finish {
            if self.pos_in_finish(self.pen_pos)
                && (self.pen_state == PenState::Up || self.pen_state == PenState::Proximity)
            {
                indicators::draw_finish_indicator(cx, self.pen_state, self.current, zoom);
            } else {
                indicators::draw_pos_indicator(cx, self.pen_state, self.current, zoom);
            }
        }

        cx.restore().unwrap();
    }
}

impl PolygonBuilder {
    const FINISH_THRESHOLD_DIST: f64 = 8.0;

    /// The current state as a polygon.
    pub fn state_as_polygon(&self) -> Polygon {
        Polygon {
            start: self.start,
            path: self.path.clone(),
        }
    }

    fn pos_in_finish(&self, pos: na::Vector2<f64>) -> bool {
        (pos - self.path.last().copied().unwrap_or(self.start)).magnitude()
            < Self::FINISH_THRESHOLD_DIST
    }
}
