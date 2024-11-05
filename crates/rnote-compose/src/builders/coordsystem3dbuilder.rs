// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::eventresult::EventPropagation;
use crate::penevent::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Line;
use crate::style::{indicators, Composer};
use crate::{Constraints, EventResult};
use crate::{Shape, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use std::time::Instant;

/// 3D coordinate system builder.
#[derive(Debug, Clone)]
pub struct CoordSystem3DBuilder {
    /// Tip of the z axis.
    tip_z: na::Vector2<f64>,
    /// Tip of the y axis.
    tip_y: na::Vector2<f64>,
}

impl BuilderCreator for CoordSystem3DBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            tip_z: element.pos,
            tip_y: element.pos,
        }
    }
}

impl Buildable for CoordSystem3DBuilder {
    type Emit = Shape;

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>> {
        let progress = match event {
            PenEvent::Down { element, .. } => {
                self.tip_y = constraints.constrain(element.pos - self.tip_z) + self.tip_z;
                BuilderProgress::InProgress
            }
            PenEvent::Up { .. } => BuilderProgress::Finished(
                self.state_as_lines()
                    .iter()
                    .map(|&line| Shape::Line(line))
                    .collect::<Vec<Shape>>(),
            ),
            _ => BuilderProgress::InProgress,
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
            request_animation_frame: false,
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        Some(
            self.state_as_lines()
                .iter()
                .map(|line| line.composed_bounds(style))
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x))
                .loosened(indicators::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();

        for line in self.state_as_lines() {
            line.draw_composed(cx, style);
        }

        indicators::draw_pos_indicator(cx, PenState::Up, self.tip_z, zoom);
        indicators::draw_pos_indicator(cx, PenState::Down, self.tip_y, zoom);
        cx.restore().unwrap();
    }
}

impl CoordSystem3DBuilder {
    /// The current state as six individual lines.
    pub fn state_as_lines(&self) -> Vec<Line> {
        let center = na::vector!(self.tip_z.x, self.tip_y.y);
        let tip_x_offset =
            ((center - self.tip_z).magnitude() + (center - self.tip_y).magnitude()) / 4.0;

        let up_axis = Line {
            start: center,
            end: self.tip_z,
        };

        let down_axis = Line {
            start: center,
            end: na::vector![center.x, 2.0 * self.tip_y.y - self.tip_z.y],
        };

        let right_axis = Line {
            start: center,
            end: self.tip_y,
        };

        let left_axis = Line {
            start: center,
            end: na::vector![2.0 * self.tip_z.x - self.tip_y.x, center.y],
        };

        let forward_axis = Line {
            start: center,
            end: center + na::vector![-tip_x_offset, tip_x_offset],
        };

        let backward_axis = Line {
            start: center,
            end: center + na::vector![tip_x_offset, -tip_x_offset],
        };

        vec![
            up_axis,
            down_axis,
            right_axis,
            left_axis,
            forward_axis,
            backward_axis,
        ]
    }
}
