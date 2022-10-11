use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Instant;

use crate::penhelpers::PenEvent;
use crate::penpath::{Element, Segment};
use crate::style::Composer;
use crate::{PenPath, Shape, Style};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::{Constraints, ShapeBuilderBehaviour};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum SimplePenPathBuilderState {
    Start,
    During,
}

#[derive(Debug, Clone)]
/// The pen path builder
pub struct SimplePenPathBuilder {
    pub(crate) state: SimplePenPathBuilderState,
    /// Buffered elements, which are filled up by new pen events and used to try to build path segments
    pub buffer: VecDeque<Element>,
}

impl ShapeBuilderCreator for SimplePenPathBuilder {
    fn start(element: Element,_now: Instant) -> Self {
        let mut buffer = VecDeque::new();
        buffer.push_back(element);

        Self {
            state: SimplePenPathBuilderState::Start,
            buffer,
        }
    }
}

impl ShapeBuilderBehaviour for SimplePenPathBuilder {
    fn handle_event(&mut self, event: PenEvent,_now: Instant, _constraints: Constraints) -> BuilderProgress {
        /*         log::debug!(
            "event: {:?}; buffer.len(): {}, state: {:?}",
            event,
            self.buffer.len(),
            self.state
        ); */

        match (&mut self.state, event) {
            (SimplePenPathBuilderState::Start, PenEvent::Down { element, .. }) => {
                self.buffer.push_back(element);

                match self.try_build_segments() {
                    Some(shapes) => BuilderProgress::EmitContinue(shapes),
                    None => BuilderProgress::InProgress,
                }
            }
            (SimplePenPathBuilderState::During, PenEvent::Down { element, .. }) => {
                self.buffer.push_back(element);

                match self.try_build_segments() {
                    Some(shapes) => BuilderProgress::EmitContinue(shapes),
                    None => BuilderProgress::InProgress,
                }
            }
            (_, PenEvent::Up { element, .. }) => {
                self.buffer.push_back(element);

                let segment = self.try_build_segments().unwrap_or_else(|| vec![]);

                self.reset();

                BuilderProgress::Finished(segment)
            }
            (_, PenEvent::Proximity { .. })
            | (_, PenEvent::KeyPressed { .. })
            | (_, PenEvent::Text { .. }) => BuilderProgress::InProgress,
            (_, PenEvent::Cancel) => {
                self.reset();

                BuilderProgress::Finished(vec![])
            }
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<AABB> {
        let stroke_width = style.stroke_width();

        if self.buffer.is_empty() {
            return None;
        }

        Some(self.buffer.iter().fold(AABB::new_invalid(), |mut acc, x| {
            acc.take_point(na::Point2::from(x.pos));
            acc.loosened(stroke_width / zoom)
        }))
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, _zoom: f64) {
        cx.save().unwrap();
        let penpath = match &self.state {
            SimplePenPathBuilderState::Start => self
                .buffer
                .iter()
                .zip(self.buffer.iter().skip(1))
                .map(|(start, end)| Segment::Line {
                    start: *start,
                    end: *end,
                })
                .collect::<PenPath>(),
            // Skipping the first buffer element as that is the not drained by the segment builder and is the prev element in the "During" state
            SimplePenPathBuilderState::During => self
                .buffer
                .iter()
                .skip(1)
                .zip(self.buffer.iter().skip(2))
                .map(|(start, end)| Segment::Line {
                    start: *start,
                    end: *end,
                })
                .collect::<PenPath>(),
        };

        penpath.draw_composed(cx, style);
        cx.restore().unwrap();
    }
}

impl SimplePenPathBuilder {
    fn try_build_segments(&mut self) -> Option<Vec<Shape>> {
        if self.buffer.len() < 2 {
            return None;
        }
        let mut segments = vec![];

        while self.buffer.len() > 2 {
            self.state = SimplePenPathBuilderState::During;

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
        self.state = SimplePenPathBuilderState::Start;
    }
}
