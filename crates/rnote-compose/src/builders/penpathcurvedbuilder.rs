// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::eventresult::EventPropagation;
use crate::penpath::{Element, Segment};
use crate::shapes::CubicBezier;
use crate::style::Composer;
use crate::PenEvent;
use crate::{Constraints, EventResult};
use crate::{PenPath, Style};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use std::time::Instant;

#[derive(Debug, Clone)]
pub(crate) enum PenPathCurvedBuilderState {
    Start,
    During,
}

#[derive(Debug, Clone)]
/// Pen path curved builder.
pub struct PenPathCurvedBuilder {
    state: PenPathCurvedBuilderState,
    /// Buffered elements, which are filled up by new pen events and used to build path segments.
    buffer: Vec<Element>,
    /// the index of the current first unprocessed buffer element.
    i: usize,
}

impl BuilderCreator for PenPathCurvedBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        Self {
            state: PenPathCurvedBuilderState::Start,
            buffer: vec![element],
            i: 0,
        }
    }
}

impl Buildable for PenPathCurvedBuilder {
    type Emit = Segment;

    fn handle_event(
        &mut self,
        event: PenEvent,
        _now: Instant,
        _constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>> {
        let progress = match (&mut self.state, event) {
            (PenPathCurvedBuilderState::Start, PenEvent::Down { element, .. }) => {
                self.buffer.push(element);

                match self.try_build_segments_start() {
                    Some(segments) => {
                        // Here we have enough elements to switch into during state

                        self.state = PenPathCurvedBuilderState::During;
                        BuilderProgress::EmitContinue(segments)
                    }
                    None => BuilderProgress::InProgress,
                }
            }
            (PenPathCurvedBuilderState::During, PenEvent::Down { element, .. }) => {
                self.buffer.push(element);

                match self.try_build_segments_during() {
                    Some(shapes) => BuilderProgress::EmitContinue(shapes),
                    None => BuilderProgress::InProgress,
                }
            }
            (_, PenEvent::Up { element, .. }) => {
                self.buffer.push(element);

                BuilderProgress::Finished(self.try_build_segments_end())
            }
            (_, PenEvent::Proximity { .. })
            | (_, PenEvent::KeyPressed { .. })
            | (_, PenEvent::Text { .. }) => BuilderProgress::InProgress,
            (_, PenEvent::Cancel) => {
                self.reset();

                BuilderProgress::Finished(vec![])
            }
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<Aabb> {
        let stroke_width = style.stroke_width();

        if self.buffer.len().saturating_sub(1) < self.i {
            return None;
        }

        Some(
            self.buffer[self.i..]
                .iter()
                .fold(Aabb::new_invalid(), |mut acc, x| {
                    acc.take_point(x.pos.into());
                    acc.loosened(stroke_width / zoom)
                }),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, _zoom: f64) {
        if self.buffer.len().saturating_sub(1) < self.i {
            return;
        }

        cx.save().unwrap();

        let pen_path = match &self.state {
            PenPathCurvedBuilderState::Start => {
                PenPath::try_from_elements(self.buffer[self.i..].iter().copied())
            }
            // Skipping the first buffer element as that is the not drained by the segment builder
            // and is the prev element in the "During" state
            PenPathCurvedBuilderState::During => {
                PenPath::try_from_elements(self.buffer[self.i..].iter().skip(1).copied())
            }
        };

        if let Some(pen_path) = pen_path {
            pen_path.draw_composed(cx, style);
        }

        cx.restore().unwrap();
    }
}

impl PenPathCurvedBuilder {
    fn try_build_segments_start(&mut self) -> Option<Vec<Segment>> {
        if self.buffer.len().saturating_sub(1) > self.i {
            let segment = Segment::LineTo {
                end: self.buffer[self.i],
            };

            Some(vec![segment])
        } else {
            None
        }
    }

    fn try_build_segments_during(&mut self) -> Option<Vec<Segment>> {
        if self.buffer.len().saturating_sub(1) < self.i + 3 {
            return None;
        }

        let mut segments = vec![];

        while self.buffer.len().saturating_sub(1) >= self.i + 3 {
            if let Some(cubbez) = CubicBezier::new_w_catmull_rom(
                self.buffer[self.i].pos,
                self.buffer[self.i + 1].pos,
                self.buffer[self.i + 2].pos,
                self.buffer[self.i + 3].pos,
            ) {
                let segment = Segment::CubBezTo {
                    cp1: cubbez.cp1,
                    cp2: cubbez.cp2,
                    end: Element {
                        pos: cubbez.end,
                        ..self.buffer[self.i + 2]
                    },
                };

                self.i += 1;

                segments.push(segment);
            } else {
                let segment = Segment::LineTo {
                    end: self.buffer[self.i + 2],
                };

                self.i += 1;

                segments.push(segment);
            }
        }

        Some(segments)
    }

    fn try_build_segments_end(&mut self) -> Vec<Segment> {
        let buffer_last_pos = self.buffer.len().saturating_sub(1);
        let mut segments: Vec<Segment> = vec![];

        while let Some(mut new_segments) = if buffer_last_pos > self.i + 2 {
            if let Some(cubbez) = CubicBezier::new_w_catmull_rom(
                self.buffer[self.i].pos,
                self.buffer[self.i + 1].pos,
                self.buffer[self.i + 2].pos,
                self.buffer[self.i + 3].pos,
            ) {
                let segment = Segment::CubBezTo {
                    cp1: cubbez.cp1,
                    cp2: cubbez.cp2,
                    end: Element {
                        pos: cubbez.end,
                        ..self.buffer[self.i + 2]
                    },
                };

                self.i += 1;

                Some(vec![segment])
            } else {
                let segment = Segment::LineTo {
                    end: self.buffer[self.i + 2],
                };

                self.i += 1;

                Some(vec![segment])
            }
        } else if buffer_last_pos > self.i + 1 {
            let segment = Segment::LineTo {
                end: self.buffer[self.i + 1],
            };

            self.i += 2;

            Some(vec![segment])
        } else if buffer_last_pos > self.i {
            let segment = Segment::LineTo {
                end: self.buffer[self.i],
            };

            self.i += 1;

            Some(vec![segment])
        } else {
            None
        } {
            segments.append(&mut new_segments);
        }

        self.reset();

        segments
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.state = PenPathCurvedBuilderState::Start;
    }
}
