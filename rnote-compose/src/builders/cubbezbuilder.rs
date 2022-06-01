use p2d::bounding_volume::{BoundingVolume, AABB};

use crate::helpers::AABBHelpers;
use crate::penhelpers::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::CubicBezier;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::{ConstraintRatio, Constraints, ShapeBuilderBehaviour};

#[derive(Debug, Clone)]
/// The state
pub enum CubBezBuilderState {
    /// start
    Start(na::Vector2<f64>),
    /// first control point
    Cp1 {
        /// start
        start: na::Vector2<f64>,
        /// first control point
        cp1: na::Vector2<f64>,
    },
    /// second control point
    Cp2 {
        /// start
        start: na::Vector2<f64>,
        /// first control point
        cp1: na::Vector2<f64>,
        /// second control point
        cp2: na::Vector2<f64>,
    },
    /// end
    End {
        /// start
        start: na::Vector2<f64>,
        /// first control point
        cp1: na::Vector2<f64>,
        /// second control point
        cp2: na::Vector2<f64>,
        /// end
        end: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
/// building cubic bezier
pub struct CubBezBuilder {
    /// the state
    pub state: CubBezBuilderState,
}

impl ShapeBuilderCreator for CubBezBuilder {
    fn start(element: Element) -> Self {
        Self {
            state: CubBezBuilderState::Start(element.pos),
        }
    }
}

impl ShapeBuilderBehaviour for CubBezBuilder {
    fn handle_event(&mut self, event: PenEvent, constraints: Constraints) -> BuilderProgress {
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        match (&mut self.state, event) {
            (CubBezBuilderState::Start(start), PenEvent::Down { element, .. }) => {
                *start = element.pos;

                self.state = CubBezBuilderState::Cp1 {
                    start: *start,
                    cp1: element.pos,
                };
            }
            (CubBezBuilderState::Start(start), PenEvent::Up { element, .. }) => {
                // should not be reachable, but just in case we transition here too
                self.state = CubBezBuilderState::Cp1 {
                    start: *start,
                    cp1: element.pos,
                };
            }
            (CubBezBuilderState::Start(_), ..) => {}
            (CubBezBuilderState::Cp1 { start, cp1, .. }, PenEvent::Down { element, .. }) => {
                *cp1 = constraints.constrain(element.pos - *start) + *start;
            }
            (CubBezBuilderState::Cp1 { start, cp1 }, PenEvent::Up { element, .. }) => {
                self.state = CubBezBuilderState::Cp2 {
                    start: *start,
                    cp1: *cp1,
                    cp2: element.pos,
                };
            }
            (CubBezBuilderState::Cp1 { .. }, ..) => {}
            (CubBezBuilderState::Cp2 { cp1, cp2, .. }, PenEvent::Down { element, .. }) => {
                *cp2 = constraints.constrain(element.pos - *cp1) + *cp1;
            }
            (CubBezBuilderState::Cp2 { start, cp1, cp2 }, PenEvent::Up { element, .. }) => {
                self.state = CubBezBuilderState::End {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: element.pos,
                };
            }
            (CubBezBuilderState::Cp2 { .. }, ..) => {}
            (CubBezBuilderState::End { cp2, end, .. }, PenEvent::Down { element, .. }) => {
                *end = constraints.constrain(element.pos - *cp2) + *cp2;
            }
            (
                CubBezBuilderState::End {
                    start,
                    cp1,
                    cp2,
                    end,
                },
                PenEvent::Up { .. },
            ) => {
                return BuilderProgress::Finished(vec![Shape::CubicBezier(CubicBezier {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: *end,
                })]);
            }
            (CubBezBuilderState::End { .. }, ..) => {}
        }

        BuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64, _constraints: Constraints) -> AABB {
        let stroke_width = style.stroke_width();

        match &self.state {
            CubBezBuilderState::Start(start) => AABB::from_half_extents(
                na::Point2::from(*start),
                na::Vector2::repeat(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom),
            ),
            CubBezBuilderState::Cp1 { start, cp1 } => {
                AABB::new_positive(na::Point2::from(*start), na::Point2::from(*cp1))
                    .loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom)
            }
            CubBezBuilderState::Cp2 { start, cp1, cp2 } => {
                let mut aabb = AABB::new_positive(na::Point2::from(*start), na::Point2::from(*cp2));
                aabb.take_point(na::Point2::from(*cp1));
                aabb.loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom)
            }
            CubBezBuilderState::End {
                start,
                cp1,
                cp2,
                end,
            } => {
                let mut aabb = AABB::new_positive(na::Point2::from(*start), na::Point2::from(*end));
                aabb.take_point(na::Point2::from(*cp1));
                aabb.take_point(na::Point2::from(*cp2));
                aabb.loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom)
            }
        }
    }

    fn draw_styled(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        style: &Style,
        zoom: f64,
        _constraints: Constraints,
    ) {
        match &self.state {
            CubBezBuilderState::Start(start) => {
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *start, zoom);
            }
            CubBezBuilderState::Cp1 { start, cp1 } => {
                drawhelpers::draw_vec_indicator(cx, PenState::Down, *start, *cp1, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *cp1, zoom);
            }
            CubBezBuilderState::Cp2 { start, cp1, cp2 } => {
                let cubbez = CubicBezier {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: *cp2,
                };
                cubbez.draw_composed(cx, style);

                drawhelpers::draw_vec_indicator(cx, PenState::Down, *start, *cp1, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *cp1, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *cp2, zoom);
            }
            CubBezBuilderState::End {
                start,
                cp1,
                cp2,
                end,
            } => {
                let cubbez = CubicBezier {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: *end,
                };
                cubbez.draw_composed(cx, style);

                drawhelpers::draw_vec_indicator(cx, PenState::Down, *start, *cp1, zoom);
                drawhelpers::draw_vec_indicator(cx, PenState::Down, *cp2, *end, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *cp1, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, *cp2, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *end, zoom);
            }
        }
    }
}
