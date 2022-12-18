use p2d::bounding_volume::AABB;
use piet::RenderContext;
use std::collections::VecDeque;
use std::time::Instant;

use crate::penevents::PenEvent;
use crate::penpath::{Element, Segment};
use crate::style::Composer;
use crate::{PenPath, Style};

use super::penpathbuilderbehaviour::{
    PenPathBuilderBehaviour, PenPathBuilderCreator, PenPathBuilderProgress,
};
use super::Constraints;

#[derive(Debug, Clone)]
/// The simple pen path builder
pub struct PenPathSimpleBuilder {
    /// Buffered elements, which are filled up by new pen events and used to try to build path segments
    pub buffer: VecDeque<Element>,
}

impl PenPathBuilderCreator for PenPathSimpleBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        let buffer = VecDeque::from_iter([element]);

        Self { buffer }
    }
}

impl PenPathBuilderBehaviour for PenPathSimpleBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        _constraints: Constraints,
    ) -> PenPathBuilderProgress {
        /*         log::debug!(
            "event: {:?}; buffer.len(): {}, state: {:?}",
            event,
            self.buffer.len(),
            self.state
        ); */

        match event {
            PenEvent::Down { element, .. } => {
                self.buffer.push_back(element);

                PenPathBuilderProgress::EmitContinue(self.build_segments())
            }
            PenEvent::Up { element, .. } => {
                self.buffer.push_back(element);

                let segments = self.build_segments();
                self.reset();

                PenPathBuilderProgress::Finished(segments)
            }
            PenEvent::Proximity { .. } | PenEvent::KeyPressed { .. } | PenEvent::Text { .. } => {
                PenPathBuilderProgress::InProgress
            }
            PenEvent::Cancel => {
                self.reset();
                PenPathBuilderProgress::Finished(vec![])
            }
        }
    }

    fn bounds(&self, style: &Style, _zoom: f64) -> Option<AABB> {
        let pen_path = PenPath::try_from_elements(self.buffer.iter().copied())?;

        Some(pen_path.composed_bounds(style))
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, _zoom: f64) {
        cx.save().unwrap();

        if let Some(pen_path) = PenPath::try_from_elements(self.buffer.iter().copied()) {
            pen_path.draw_composed(cx, style);
        }

        cx.restore().unwrap();
    }
}

impl PenPathSimpleBuilder {
    fn build_segments(&mut self) -> Vec<Segment> {
        self.buffer
            .drain(..)
            .map(|el| Segment::LineTo { end: el })
            .collect()
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}
