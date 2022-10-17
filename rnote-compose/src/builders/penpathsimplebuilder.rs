use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use std::collections::VecDeque;
use std::time::Instant;

use crate::penhelpers::PenEvent;
use crate::penpath::{Element, Segment};
use crate::style::Composer;
use crate::{PenPath, Shape, Style};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::{Constraints, ShapeBuilderBehaviour};

#[derive(Debug, Clone)]
/// The simple pen path builder
pub struct PenPathSimpleBuilder {
    /// Buffered elements, which are filled up by new pen events and used to try to build path segments
    pub buffer: VecDeque<Element>,
}

impl ShapeBuilderCreator for PenPathSimpleBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        let mut buffer = VecDeque::new();
        buffer.push_back(element);

        Self { buffer }
    }
}

impl ShapeBuilderBehaviour for PenPathSimpleBuilder {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        _constraints: Constraints,
    ) -> BuilderProgress {
        /*         log::debug!(
            "event: {:?}; buffer.len(): {}, state: {:?}",
            event,
            self.buffer.len(),
            self.state
        ); */

        match event {
            PenEvent::Down { element, .. } => {
                self.buffer.push_back(element);

                match self.try_build_segments() {
                    Some(shapes) => BuilderProgress::EmitContinue(shapes),
                    None => BuilderProgress::InProgress,
                }
            }
            PenEvent::Up { element, .. } => {
                self.buffer.push_back(element);

                let segment = self.try_build_segments().unwrap_or_else(|| vec![]);

                self.reset();

                BuilderProgress::Finished(segment)
            }
            PenEvent::Proximity { .. } | PenEvent::KeyPressed { .. } | PenEvent::Text { .. } => {
                BuilderProgress::InProgress
            }
            PenEvent::Cancel => {
                self.reset();

                BuilderProgress::Finished(vec![])
            }
        }
    }

    fn bounds(&self, style: &Style, _zoom: f64) -> Option<AABB> {
        let penpath = self
            .buffer
            .iter()
            .zip(self.buffer.iter().skip(1))
            .map(|(start, end)| Segment::Line {
                start: *start,
                end: *end,
            })
            .collect::<PenPath>();

        if penpath.is_empty() {
            return None;
        }

        Some(penpath.iter().fold(AABB::new_invalid(), |acc, x| {
            acc.merged(&x.composed_bounds(style))
        }))
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, _zoom: f64) {
        cx.save().unwrap();
        let penpath = self
            .buffer
            .iter()
            .zip(self.buffer.iter().skip(1))
            .map(|(start, end)| Segment::Line {
                start: *start,
                end: *end,
            })
            .collect::<PenPath>();

        penpath.draw_composed(cx, style);
        cx.restore().unwrap();
    }
}

impl PenPathSimpleBuilder {
    fn try_build_segments(&mut self) -> Option<Vec<Shape>> {
        if self.buffer.len() < 2 {
            return None;
        }
        let mut segments = vec![];

        while self.buffer.len() > 2 {
            segments.push(Shape::Segment(Segment::Line {
                start: self.buffer[0],
                end: self.buffer[1],
            }));

            self.buffer.pop_front();
        }

        Some(segments)
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}
