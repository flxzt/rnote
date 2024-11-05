// Impoorts
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::constraints::ConstraintRatio;
use crate::eventresult::EventPropagation;
use crate::ext::AabbExt;
use crate::penevent::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::CubicBezier;
use crate::style::{indicators, Composer};
use crate::{Constraints, EventResult};
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use std::time::Instant;

#[derive(Debug, Clone)]
enum CubBezBuilderState {
    Cp1 {
        start: na::Vector2<f64>,
        cp1: na::Vector2<f64>,
    },
    Cp1Finished {
        start: na::Vector2<f64>,
        cp1: na::Vector2<f64>,
    },
    Cp2 {
        start: na::Vector2<f64>,
        cp1: na::Vector2<f64>,
        cp2: na::Vector2<f64>,
    },
    Cp2Finished {
        start: na::Vector2<f64>,
        cp1: na::Vector2<f64>,
        cp2: na::Vector2<f64>,
    },
    End {
        start: na::Vector2<f64>,
        cp1: na::Vector2<f64>,
        cp2: na::Vector2<f64>,
        end: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
/// Cubic bezier builder.
pub struct CubBezBuilder {
    state: CubBezBuilderState,
}

impl BuilderCreator for CubBezBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            state: CubBezBuilderState::Cp1 {
                start: element.pos,
                cp1: element.pos,
            },
        }
    }
}

impl Buildable for CubBezBuilder {
    type Emit = Shape;

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        mut constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>> {
        // we always want to allow horizontal and vertical constraints while building a cubbez
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        let progress = match (&mut self.state, event) {
            (CubBezBuilderState::Cp1 { start, cp1, .. }, PenEvent::Down { element, .. }) => {
                *cp1 = constraints.constrain(element.pos - *start) + *start;
                BuilderProgress::InProgress
            }
            (CubBezBuilderState::Cp1 { start, .. }, PenEvent::Up { element, .. }) => {
                self.state = CubBezBuilderState::Cp1Finished {
                    start: *start,
                    cp1: constraints.constrain(element.pos - *start) + *start,
                };
                BuilderProgress::InProgress
            }
            (CubBezBuilderState::Cp1 { .. }, ..) => BuilderProgress::InProgress,
            (CubBezBuilderState::Cp1Finished { start, cp1 }, PenEvent::Down { element, .. }) => {
                self.state = CubBezBuilderState::Cp2 {
                    start: *start,
                    cp1: *cp1,
                    cp2: constraints.constrain(element.pos - *cp1) + *cp1,
                };
                BuilderProgress::InProgress
            }
            (CubBezBuilderState::Cp1Finished { .. }, ..) => BuilderProgress::InProgress,
            (CubBezBuilderState::Cp2 { cp1, cp2, .. }, PenEvent::Down { element, .. }) => {
                *cp2 = constraints.constrain(element.pos - *cp1) + *cp1;
                BuilderProgress::InProgress
            }
            (CubBezBuilderState::Cp2 { start, cp1, .. }, PenEvent::Up { element, .. }) => {
                self.state = CubBezBuilderState::Cp2Finished {
                    start: *start,
                    cp1: *cp1,
                    cp2: constraints.constrain(element.pos - *cp1) + *cp1,
                };
                BuilderProgress::InProgress
            }
            (CubBezBuilderState::Cp2 { .. }, ..) => BuilderProgress::InProgress,
            (
                CubBezBuilderState::Cp2Finished { start, cp1, cp2 },
                PenEvent::Down { element, .. },
            ) => {
                self.state = CubBezBuilderState::End {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: constraints.constrain(element.pos - *cp2) + *cp2,
                };
                BuilderProgress::InProgress
            }
            (CubBezBuilderState::Cp2Finished { .. }, ..) => BuilderProgress::InProgress,
            (CubBezBuilderState::End { cp2, end, .. }, PenEvent::Down { element, .. }) => {
                *end = constraints.constrain(element.pos - *cp2) + *cp2;
                BuilderProgress::InProgress
            }
            (
                CubBezBuilderState::End {
                    start,
                    cp1,
                    cp2,
                    end,
                },
                PenEvent::Up { .. },
            ) => BuilderProgress::Finished(vec![Shape::CubicBezier(CubicBezier {
                start: *start,
                cp1: *cp1,
                cp2: *cp2,
                end: *end,
            })]),
            (CubBezBuilderState::End { .. }, ..) => BuilderProgress::InProgress,
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
            request_animation_frame: false,
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        let stroke_width = style.stroke_width();

        match &self.state {
            CubBezBuilderState::Cp1 { start, cp1 }
            | CubBezBuilderState::Cp1Finished { start, cp1 } => Some(
                Aabb::new_positive((*start).into(), (*cp1).into())
                    .loosened(stroke_width.max(indicators::POS_INDICATOR_RADIUS) / zoom),
            ),
            CubBezBuilderState::Cp2 { start, cp1, cp2 }
            | CubBezBuilderState::Cp2Finished { start, cp1, cp2 } => {
                let mut aabb = Aabb::new_positive((*start).into(), (*cp2).into());
                aabb.take_point((*cp1).into());

                Some(aabb.loosened(stroke_width.max(indicators::POS_INDICATOR_RADIUS) / zoom))
            }
            CubBezBuilderState::End {
                start,
                cp1,
                cp2,
                end,
            } => {
                let mut aabb = Aabb::new_positive((*start).into(), (*end).into());
                aabb.take_point((*cp1).into());
                aabb.take_point((*cp2).into());

                Some(aabb.loosened(stroke_width.max(indicators::POS_INDICATOR_RADIUS) / zoom))
            }
        }
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        match &self.state {
            CubBezBuilderState::Cp1 { start, cp1 }
            | CubBezBuilderState::Cp1Finished { start, cp1 } => {
                indicators::draw_vec_indicator(cx, PenState::Down, *start, *cp1, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                indicators::draw_pos_indicator(cx, PenState::Down, *cp1, zoom);
            }
            CubBezBuilderState::Cp2 { start, cp1, cp2 }
            | CubBezBuilderState::Cp2Finished { start, cp1, cp2 } => {
                let cubbez = CubicBezier {
                    start: *start,
                    cp1: *cp1,
                    cp2: *cp2,
                    end: *cp2,
                };
                cubbez.draw_composed(cx, style);

                indicators::draw_vec_indicator(cx, PenState::Down, *start, *cp1, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *cp1, zoom);
                indicators::draw_pos_indicator(cx, PenState::Down, *cp2, zoom);
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

                indicators::draw_vec_indicator(cx, PenState::Down, *start, *cp1, zoom);
                indicators::draw_vec_indicator(cx, PenState::Down, *cp2, *end, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *start, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *cp1, zoom);
                indicators::draw_pos_indicator(cx, PenState::Up, *cp2, zoom);
                indicators::draw_pos_indicator(cx, PenState::Down, *end, zoom);
            }
        }
    }
}
