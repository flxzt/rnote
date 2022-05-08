use p2d::bounding_volume::{BoundingVolume, AABB};

use crate::helpers::AABBHelpers;
use crate::penhelpers::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::QuadraticBezier;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::ShapeBuilderBehaviour;

#[derive(Debug, Clone)]
/// The state
pub enum QuadBezBuilderState {
    /// start
    Start(na::Vector2<f64>),
    /// control point
    Cp {
        /// start
        start: na::Vector2<f64>,
        /// control point
        cp: na::Vector2<f64>,
    },
    /// end
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
/// building quadratic bezier
pub struct QuadBezBuilder {
    /// the state
    pub state: QuadBezBuilderState,
}

impl ShapeBuilderCreator for QuadBezBuilder {
    fn start(element: Element) -> Self {
        Self {
            state: QuadBezBuilderState::Start(element.pos),
        }
    }
}

impl ShapeBuilderBehaviour for QuadBezBuilder {
    fn handle_event(&mut self, event: PenEvent) -> BuilderProgress {
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        match (&mut self.state, event) {
            (QuadBezBuilderState::Start(start), PenEvent::Down { element, .. }) => {
                *start = element.pos;

                self.state = QuadBezBuilderState::Cp {
                    start: *start,
                    cp: element.pos,
                };
            }
            (QuadBezBuilderState::Start(start), PenEvent::Up { element, .. }) => {
                // should not be reachable, but just in case we transition here too
                self.state = QuadBezBuilderState::Cp {
                    start: *start,
                    cp: element.pos,
                };
            }
            (QuadBezBuilderState::Start(_), ..) => {}
            (QuadBezBuilderState::Cp { cp, .. }, PenEvent::Down { element, .. }) => {
                *cp = element.pos;
            }
            (QuadBezBuilderState::Cp { start, cp }, PenEvent::Up { element, .. }) => {
                self.state = QuadBezBuilderState::End {
                    start: *start,
                    cp: *cp,
                    end: element.pos,
                };
            }
            (QuadBezBuilderState::Cp { .. }, ..) => {}
            (QuadBezBuilderState::End { end, .. }, PenEvent::Down { element, .. }) => {
                *end = element.pos;
            }
            (QuadBezBuilderState::End { start, cp, end }, PenEvent::Up { .. }) => {
                return BuilderProgress::Finished(vec![Shape::QuadraticBezier(QuadraticBezier {
                    start: *start,
                    cp: *cp,
                    end: *end,
                })]);
            }
            (QuadBezBuilderState::End { .. }, ..) => {}
        }

        BuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> AABB {
        let stroke_width = style.stroke_width();

        match &self.state {
            crate::builders::quadbezbuilder::QuadBezBuilderState::Start(start) => {
                AABB::from_half_extents(
                    na::Point2::from(*start),
                    na::Vector2::repeat(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom),
                )
            }
            crate::builders::quadbezbuilder::QuadBezBuilderState::Cp { start, cp } => {
                AABB::new_positive(na::Point2::from(*start), na::Point2::from(*cp))
                    .loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom)
            }
            crate::builders::quadbezbuilder::QuadBezBuilderState::End { start, cp, end } => {
                let stroke_width = style.stroke_width();

                let mut aabb = AABB::new_positive(na::Point2::from(*start), na::Point2::from(*end));
                aabb.take_point(na::Point2::from(*cp));
                aabb.loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom)
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
