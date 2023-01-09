use std::time::Instant;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;

use crate::penevents::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Line;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style};

use super::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use super::{Constraints, ShapeBuilderBehaviour};

/// 2D single quadrant coordinate system builder
#[derive(Debug, Clone)]
pub struct QuadrantCoordSystem2DBuilder {
    /// the tip of the y axis
    pub tip_y: na::Vector2<f64>,
    /// the tip of the x axis
    pub tip_x: na::Vector2<f64>,
}

impl ShapeBuilderCreator for QuadrantCoordSystem2DBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            tip_y: element.pos,
            tip_x: element.pos,
        }
    }
}

impl ShapeBuilderBehaviour for QuadrantCoordSystem2DBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        constraints: Constraints,
    ) -> ShapeBuilderProgress {
        match event {
            PenEvent::Down { element, .. } => {
                self.tip_x = constraints.constrain(element.pos - self.tip_y) + self.tip_y;
            }
            PenEvent::Up { .. } => {
                return ShapeBuilderProgress::Finished(
                    self.state_as_lines()
                        .iter()
                        .map(|&line| Shape::Line(line))
                        .collect::<Vec<Shape>>(),
                );
            }
            _ => {}
        }

        ShapeBuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        Some(
            self.state_as_lines()
                .iter()
                .map(|line| line.composed_bounds(style))
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x))
                .loosened(drawhelpers::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();

        for line in self.state_as_lines() {
            line.draw_composed(cx, style);
        }

        drawhelpers::draw_pos_indicator(cx, PenState::Up, self.tip_y, zoom);
        drawhelpers::draw_pos_indicator(cx, PenState::Down, self.tip_x, zoom);
        cx.restore().unwrap();
    }
}

impl QuadrantCoordSystem2DBuilder {
    /// The current state represented by two lines
    pub fn state_as_lines(&self) -> Vec<Line> {
        let center = na::vector!(self.tip_y.x, self.tip_x.y);

        let up_axis = Line {
            start: center,
            end: self.tip_y,
        };

        let right_axis = Line {
            start: center,
            end: self.tip_x,
        };

        vec![up_axis, right_axis]
    }
}
