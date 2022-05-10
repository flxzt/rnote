use p2d::bounding_volume::{BoundingVolume, AABB};
use p2d::shape::Cuboid;
use piet::RenderContext;

use crate::penhelpers::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Rectangle;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style, Transform};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::{Constraint, ShapeBuilderBehaviour};

/// rect builder
#[derive(Debug, Clone)]
pub struct RectangleBuilder {
    /// the start position
    pub start: na::Vector2<f64>,
    /// the current position
    pub current: na::Vector2<f64>,

    constraint: Constraint,
}

impl ShapeBuilderCreator for RectangleBuilder {
    fn start(element: Element) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
            constraint: Constraint::default(),
        }
    }
}

impl ShapeBuilderBehaviour for RectangleBuilder {
    fn handle_event(&mut self, event: PenEvent, constraint: Constraint) -> BuilderProgress {
        self.constraint = constraint;

        match event {
            PenEvent::Down { element, .. } => {
                self.current = element.pos;
            }
            PenEvent::Up { .. } => {
                return BuilderProgress::Finished(vec![Shape::Rectangle(self.state_as_rect())]);
            }
            PenEvent::Proximity { .. } => {}
            PenEvent::Cancel => {}
        }

        BuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> AABB {
        self.state_as_rect()
            .composed_bounds(style)
            .loosened(drawhelpers::POS_INDICATOR_RADIUS / zoom)
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();
        let rect = self.state_as_rect();
        rect.draw_composed(cx, style);

        drawhelpers::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        drawhelpers::draw_pos_indicator(cx, PenState::Down, self.current, zoom);
        cx.restore().unwrap();
    }
}

impl RectangleBuilder {
    /// The current state as rectangle
    pub fn state_as_rect(&self) -> Rectangle {
        let relative_extents = self.constraint.constrain(self.current - self.start);
        let current = relative_extents + self.start;

        let center = (self.start + current) / 2.0;
        let transform = Transform::new_w_isometry(na::Isometry2::new(center, 0.0));
        let half_extents = relative_extents / 2.0;
        let cuboid = Cuboid::new(half_extents);

        Rectangle { cuboid, transform }
    }
}
