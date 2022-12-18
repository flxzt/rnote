use std::time::Instant;

use p2d::bounding_volume::{BoundingVolume, AABB};

use crate::helpers::AABBHelpers;
use crate::penevents::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::QuadraticBezier;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style};

use super::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use super::{ConstraintRatio, Constraints, ShapeBuilderBehaviour};

#[derive(Debug, Clone)]
/// The quadbez builder state
pub enum QuadBezBuilderState {
    /// setting the start point of the new quadbez
    Start(na::Vector2<f64>),
    /// setting the control point of the new quadbez
    Cp {
        /// start
        start: na::Vector2<f64>,
        /// control point
        cp: na::Vector2<f64>,
    },
    /// setting the end point of the new quadbez
    End {
        /// start
        start: na::Vector2<f64>,
        /// control point
        cp: na::Vector2<f64>,
        /// end
        end: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
/// quadratic bezier builder
pub struct QuadBezBuilder {
    /// the state
    pub state: QuadBezBuilderState,
}

impl ShapeBuilderCreator for QuadBezBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            state: QuadBezBuilderState::Start(element.pos),
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
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        // we always want to allow horizontal and vertical constraints while building a quadbez
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        match (&mut self.state, event) {
            (QuadBezBuilderState::Start(start), PenEvent::Down { element, .. }) => {
                *start = element.pos;

                self.state = QuadBezBuilderState::Cp {
                    start: *start,
                    cp: element.pos,
                };
            }
            (QuadBezBuilderState::Start(_), ..) => {}
            (QuadBezBuilderState::Cp { start, cp }, PenEvent::Down { element, .. }) => {
                *cp = constraints.constrain(element.pos - *start) + *start;
            }
            (QuadBezBuilderState::Cp { start, cp }, PenEvent::Up { element, .. }) => {
                self.state = QuadBezBuilderState::End {
                    start: *start,
                    cp: *cp,
                    end: element.pos,
                };
            }
            (QuadBezBuilderState::Cp { .. }, ..) => {}
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

    fn bounds(&self, style: &Style, zoom: f64) -> Option<AABB> {
        let stroke_width = style.stroke_width();

        match &self.state {
            crate::builders::quadbezbuilder::QuadBezBuilderState::Start(start) => {
                Some(AABB::from_half_extents(
                    na::Point2::from(*start),
                    na::Vector2::repeat(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom),
                ))
            }
            crate::builders::quadbezbuilder::QuadBezBuilderState::Cp { start, cp } => Some(
                AABB::new_positive(na::Point2::from(*start), na::Point2::from(*cp))
                    .loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom),
            ),
            crate::builders::quadbezbuilder::QuadBezBuilderState::End { start, cp, end } => {
                let stroke_width = style.stroke_width();

                let mut aabb = AABB::new_positive(na::Point2::from(*start), na::Point2::from(*end));
                aabb.take_point(na::Point2::from(*cp));

                Some(aabb.loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom))
            }
        }
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        match &self.state {
            QuadBezBuilderState::Start(start) => {
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *start, zoom);
            }
            QuadBezBuilderState::Cp { start, cp } => {
                drawhelpers::draw_vec_indicator(cx, PenState::Down, *start, *cp, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *cp, zoom);
            }
            QuadBezBuilderState::End { start, cp, end } => {
                let quadbez = QuadraticBezier {
                    start: *start,
                    cp: *cp,
                    end: *end,
                };
                quadbez.draw_composed(cx, style);

                drawhelpers::draw_vec_indicator(cx, PenState::Down, *start, *cp, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *cp, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *end, zoom);
            }
        }
    }
}
