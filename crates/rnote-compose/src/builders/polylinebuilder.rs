// Imports
use super::shapebuildable::{ShapeBuilderCreator, ShapeBuilderProgress};
use super::ShapeBuildable;
use crate::constraints::ConstraintRatio;
use crate::penevents::{KeyboardKey, PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Polyline;
use crate::style::{indicators, Composer};
use crate::Constraints;
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use std::time::Instant;

/// Line builder.
#[derive(Debug, Clone)]
pub struct PolylineBuilder {
    /// Start position.
    start: na::Vector2<f64>,
    /// Current position.
    current: na::Vector2<f64>,
    /// Path
    path: Vec<na::Vector2<f64>>,
    /// Pen state
    pen_state: PenState,
    /// Finish the polyline on the next `PenEvent::Up`
    finish: bool,
}

impl ShapeBuilderCreator for PolylineBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
            path: Vec::new(),
            pen_state: PenState::Down,
            finish: false,
        }
    }
}

impl ShapeBuildable for PolylineBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        mut constraints: Constraints,
    ) -> ShapeBuilderProgress {
        // we always want to allow horizontal and vertical constraints while building a polyline
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        match event {
            PenEvent::Down { element, .. } => {
                if self.pen_state == PenState::Up
                    && (element.pos - self.path.last().copied().unwrap_or(self.start)).magnitude()
                        < Self::FINISH_TRESHOLD_DIST
                {
                    self.finish = true;
                }
                self.pen_state = PenState::Down;

                if let Some(last) = self.path.last() {
                    self.current = constraints.constrain(element.pos - *last) + *last;
                } else {
                    self.current = constraints.constrain(element.pos - self.start) + self.start;
                }
            }
            PenEvent::Up { .. } => {
                if self.finish {
                    return ShapeBuilderProgress::Finished(vec![Shape::Polyline(
                        self.state_as_polyline(),
                    )]);
                }

                if self.pen_state == PenState::Down {
                    self.path.push(self.current);
                }
                self.pen_state = PenState::Up;
            }
            PenEvent::Proximity { .. } => {
                self.pen_state = PenState::Proximity;
            }
            PenEvent::KeyPressed { keyboard_key, .. } => {
                if keyboard_key == KeyboardKey::Escape {
                    return ShapeBuilderProgress::Finished(vec![Shape::Polyline(
                        self.state_as_polyline(),
                    )]);
                }
            }
            _ => {
                self.pen_state = PenState::Up;
            }
        }

        ShapeBuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        let mut polyline = self.state_as_polyline();
        if !self.finish {
            polyline.path.push(self.current);
        }
        Some(
            polyline
                .composed_bounds(style)
                .loosened(indicators::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();

        let mut polyline = self.state_as_polyline();
        if !self.finish {
            polyline.path.push(self.current);
        }
        polyline.draw_composed(cx, style);

        indicators::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        if !self.finish {
            indicators::draw_pos_indicator(cx, PenState::Down, self.current, zoom);
        }

        cx.restore().unwrap();
    }
}

impl PolylineBuilder {
    const FINISH_TRESHOLD_DIST: f64 = 5.0;

    /// The current state as a polyline.
    pub fn state_as_polyline(&self) -> Polyline {
        Polyline {
            start: self.start,
            path: self.path.clone(),
        }
    }
}
