use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;

use crate::penhelpers::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Ellipse;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style, Transform};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::ShapeBuilderBehaviour;

/// ellipse builder
#[derive(Debug, Clone)]
pub struct EllipseBuilder {
    /// the start position
    pub start: na::Vector2<f64>,
    /// the current position
    pub current: na::Vector2<f64>,
}

impl ShapeBuilderCreator for EllipseBuilder {
    fn start(element: Element) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
        }
    }
}

impl ShapeBuilderBehaviour for EllipseBuilder {
    fn handle_event(&mut self, event: PenEvent) -> BuilderProgress {
        match event {
            PenEvent::Down { element, .. } => {
                self.current = element.pos;
            }
            PenEvent::Up { .. } => {
                return BuilderProgress::Finished(vec![Shape::Ellipse(self.state_as_ellipse())]);
            }
            _ => {}
        }

        BuilderProgress::InProgress
    }

    fn bounds(&self, style: &crate::Style, zoom: f64) -> Option<AABB> {
        Some(
            self.state_as_ellipse()
                .composed_bounds(style)
                .loosened(drawhelpers::POS_INDICATOR_RADIUS / zoom),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();
        let ellipse = self.state_as_ellipse();
        ellipse.draw_composed(cx, style);

        drawhelpers::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        drawhelpers::draw_pos_indicator(cx, PenState::Down, self.current, zoom);
        cx.restore().unwrap();
    }
}

impl EllipseBuilder {
    /// The current state as ellipse
    pub fn state_as_ellipse(&self) -> Ellipse {
        let transform = Transform::new_w_isometry(na::Isometry2::new(self.start, 0.0));
        let radii = (self.current - self.start).abs();

        Ellipse { radii, transform }
    }
}
