// Imports
use super::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use super::ShapeBuilderBehaviour;
use crate::constraints::ConstraintRatio;
use crate::helpers::AabbHelpers;
use crate::penevents::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::QuadraticBezier;
use crate::style::{indicators, Composer};
use crate::Constraints;
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use std::time::Instant;

#[derive(Debug, Clone)]
enum QuadBezBuilderState {
    Cp {
        start: na::Vector2<f64>,
        cp: na::Vector2<f64>,
    },
    CpFinished {
        start: na::Vector2<f64>,
        cp: na::Vector2<f64>,
    },
    End {
        start: na::Vector2<f64>,
        cp: na::Vector2<f64>,
        end: na::Vector2<f64>,
    },
}

/// Quadratic bezier builder.
#[derive(Debug, Clone)]
pub struct QuadBezBuilder {
    state: QuadBezBuilderState,
}

impl ShapeBuilderCreator for QuadBezBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            state: QuadBezBuilderState::Cp {
                start: element.pos,
                cp: element.pos,
            },
        }
    }
}

impl ShapeBuilderBehaviour for QuadBezBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        mut constraints: Constraints,
    ) -> ShapeBuilderProgress {
        // we always want to allow horizontal and vertical constraints while building a quadbez
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        match (&mut self.state, event) {
            (QuadBezBuilderState::Cp { start, cp }, PenEvent::Down { element, .. }) => {
                *cp = constraints.constrain(element.pos - *start) + *start;
            }
            (QuadBezBuilderState::Cp { start, .. }, PenEvent::Up { element, .. }) => {
                self.state = QuadBezBuilderState::CpFinished {
                    start: *start,
                    cp: constraints.constrain(element.pos - *start) + *start,
                };
            }
            (QuadBezBuilderState::Cp { .. }, ..) => {}
            (QuadBezBuilderState::CpFinished { start, cp }, PenEvent::Down { element, .. }) => {
                self.state = QuadBezBuilderState::End {
                    start: *start,
                    cp: *cp,
                    end: constraints.constrain(element.pos - *cp) + *cp,
                };
            }
            (QuadBezBuilderState::CpFinished { .. }, ..) => {}
            (QuadBezBuilderState::End { end, cp, .. }, PenEvent::Down { element, .. }) => {
                *end = constraints.constrain(element.pos - *cp) + *cp;
            }
            (QuadBezBuilderState::End { start, cp, end }, PenEvent::Up { .. }) => {
                return ShapeBuilderProgress::Finished(vec![Shape::QuadraticBezier(
                    QuadraticBezier {
                        start: *start,
                        cp: *cp,
                        end: *end,
                    },
                )]);
            }
            (QuadBezBuilderState::End { .. }, ..) => {}
        }

        ShapeBuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        let stroke_width = style.stroke_width();

        match &self.state {
            QuadBezBuilderState::Cp { start, cp }
            | QuadBezBuilderState::CpFinished { start, cp } => Some(
                Aabb::new_positive((*start).into(), (*cp).into())
                    .loosened(stroke_width.max(indicators::POS_INDICATOR_RADIUS) / zoom),
            ),
            QuadBezBuilderState::End { start, cp, end } => {
                let stroke_width = style.stroke_width();

                let mut aabb = Aabb::new_positive((*start).into(), (*end).into());
                aabb.take_point((*cp).into());

                Some(aabb.loosened(stroke_width.max(indicators::POS_INDICATOR_RADIUS) / zoom))
            }
        }
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        match &self.state {
            QuadBezBuilderState::Cp { start, cp }
            | QuadBezBuilderState::CpFinished { start, cp } => {
                indicators::draw_vec_indicator(cx, PenState::Down, *start, *cp, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                indicators::draw_pos_indicator(cx, PenState::Down, *cp, zoom);
            }
            QuadBezBuilderState::End { start, cp, end } => {
                let quadbez = QuadraticBezier {
                    start: *start,
                    cp: *cp,
                    end: *end,
                };
                quadbez.draw_composed(cx, style);

                indicators::draw_vec_indicator(cx, PenState::Down, *start, *cp, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *cp, zoom);
                indicators::draw_pos_indicator(cx, PenState::Down, *end, zoom);
            }
        }
    }
}
