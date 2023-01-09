use std::time::Instant;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;

use crate::helpers::AabbHelpers;
use crate::penevents::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Ellipse;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style};

use super::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use super::{ConstraintRatio, Constraints, ShapeBuilderBehaviour};

#[derive(Debug, Clone)]
/// The foci ellipse builder state
pub enum FociEllipseBuilderState {
    /// first
    First(na::Vector2<f64>),
    /// foci
    Foci([na::Vector2<f64>; 2]),
    /// foci and point
    FociAndPoint {
        /// The foci
        foci: [na::Vector2<f64>; 2],
        /// the point
        point: na::Vector2<f64>,
    },
}

#[derive(Debug, Clone)]
/// building ellipse with foci and point
pub struct FociEllipseBuilder {
    /// the state
    pub state: FociEllipseBuilderState,
}

impl ShapeBuilderCreator for FociEllipseBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            state: FociEllipseBuilderState::First(element.pos),
        }
    }
}

impl ShapeBuilderBehaviour for FociEllipseBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        mut constraints: Constraints,
    ) -> ShapeBuilderProgress {
        //log::debug!("state: {:?}, event: {:?}", &self.state, &event);

        match (&mut self.state, event) {
            (FociEllipseBuilderState::First(first), PenEvent::Down { element, .. }) => {
                *first = element.pos;
            }
            (FociEllipseBuilderState::First(first), PenEvent::Up { element, .. }) => {
                self.state = FociEllipseBuilderState::Foci([*first, element.pos])
            }
            (FociEllipseBuilderState::First(_), _) => {}
            (FociEllipseBuilderState::Foci(foci), PenEvent::Down { element, .. }) => {
                // we want to allow horizontal and vertical constraints while setting the second foci
                constraints.ratios.insert(ConstraintRatio::Horizontal);
                constraints.ratios.insert(ConstraintRatio::Vertical);

                foci[1] = constraints.constrain(element.pos - foci[0]) + foci[0];
            }
            (FociEllipseBuilderState::Foci(foci), PenEvent::Up { element, .. }) => {
                self.state = FociEllipseBuilderState::FociAndPoint {
                    foci: *foci,
                    point: element.pos,
                };
            }
            (FociEllipseBuilderState::Foci(_), _) => {}
            (
                FociEllipseBuilderState::FociAndPoint { foci: _, point },
                PenEvent::Down { element, .. },
            ) => {
                *point = element.pos;
            }
            (FociEllipseBuilderState::FociAndPoint { foci, point }, PenEvent::Up { .. }) => {
                let shape = Ellipse::from_foci_and_point(*foci, *point);

                return ShapeBuilderProgress::Finished(vec![Shape::Ellipse(shape)]);
            }
            (FociEllipseBuilderState::FociAndPoint { .. }, _) => {}
        }

        ShapeBuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        let stroke_width = style.stroke_width();

        match &self.state {
            FociEllipseBuilderState::First(point) => Some(Aabb::from_half_extents(
                na::Point2::from(*point),
                na::Vector2::repeat(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom),
            )),
            FociEllipseBuilderState::Foci(foci) => Some(
                Aabb::new_positive(na::Point2::from(foci[0]), na::Point2::from(foci[1]))
                    .loosened(stroke_width.max(drawhelpers::POS_INDICATOR_RADIUS) / zoom),
            ),
            FociEllipseBuilderState::FociAndPoint { foci, point } => {
                let ellipse = Ellipse::from_foci_and_point(*foci, *point);

                Some(
                    ellipse
                        .composed_bounds(style)
                        .loosened(drawhelpers::POS_INDICATOR_RADIUS / zoom),
                )
            }
        }
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();
        match &self.state {
            FociEllipseBuilderState::First(point) => {
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *point, zoom);
            }
            FociEllipseBuilderState::Foci(foci) => {
                drawhelpers::draw_pos_indicator(cx, PenState::Up, foci[0], zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, foci[1], zoom);
            }
            FociEllipseBuilderState::FociAndPoint { foci, point } => {
                let ellipse = Ellipse::from_foci_and_point(*foci, *point);

                ellipse.draw_composed(cx, style);

                drawhelpers::draw_vec_indicator(cx, PenState::Down, foci[0], *point, zoom);
                drawhelpers::draw_vec_indicator(cx, PenState::Down, foci[1], *point, zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, foci[0], zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Up, foci[1], zoom);
                drawhelpers::draw_pos_indicator(cx, PenState::Down, *point, zoom);
            }
        }
        cx.restore().unwrap();
    }
}
