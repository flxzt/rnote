use std::time::Instant;

use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use serde::{Deserialize, Serialize};

use crate::penhelpers::PenEvent;
use crate::penpath::{Element, Segment};
use crate::shapes::CubicBezier;
use crate::style::Composer;
use crate::{PenPath, Shape, Style};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::{Constraints, ShapeBuilderBehaviour};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum PenPathCurvedBuilderState {
    Start,
    During,
}

#[derive(Debug, Clone)]
/// The pen path builder
pub struct PenPathCurvedBuilder {
    pub(crate) state: PenPathCurvedBuilderState,
    /// Buffered elements, which are filled up by new pen events and used to try to build path segments
    pub buffer: Vec<Element>,
    /// the index of the current first unprocessed buffer element
    i: usize,
}

impl ShapeBuilderCreator for PenPathCurvedBuilder {
    fn start(element: Element, _now: Instant) -> Self {
        let mut buffer = Vec::new();
        buffer.push(element);

        Self {
            state: PenPathCurvedBuilderState::Start,
            buffer,
            i: 0,
        }
    }
}

impl ShapeBuilderBehaviour for PenPathCurvedBuilder {
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

        match (&mut self.state, event) {
            (PenPathCurvedBuilderState::Start, PenEvent::Down { element, .. }) => {
                self.buffer.push(element);

                match self.try_build_segments_start() {
                    Some(shapes) => BuilderProgress::EmitContinue(shapes),
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
        }
    }

    fn bounds(&self, style: &Style, zoom: f64) -> Option<AABB> {
        let stroke_width = style.stroke_width();

        if self.buffer.len().saturating_sub(1) < self.i {
            return None;
        }

        Some(
            self.buffer[self.i..]
                .iter()
                .fold(AABB::new_invalid(), |mut acc, x| {
                    acc.take_point(na::Point2::from(x.pos));
                    acc.loosened(stroke_width / zoom)
                }),
        )
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, _zoom: f64) {
        if self.buffer.len().saturating_sub(1) < self.i {
            return;
        }

        cx.save().unwrap();

        let penpath = match &self.state {
            PenPathCurvedBuilderState::Start => self.buffer[self.i..]
                .iter()
                .zip(self.buffer.iter().skip(1))
                .map(|(start, end)| Segment::Line {
                    start: *start,
                    end: *end,
                })
                .collect::<PenPath>(),
            // Skipping the first buffer element as that is the not drained by the segment builder and is the prev element in the "During" state
            PenPathCurvedBuilderState::During => self
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

impl PenPathCurvedBuilder {
    fn try_build_segments_start(&mut self) -> Option<Vec<Shape>> {
        if self.buffer.len().saturating_sub(1) >= self.i + 2 {
            // Here we have enough elements to switch into during state
            self.state = PenPathCurvedBuilderState::During;

            let segment = Shape::Segment(Segment::Line {
                start: self.buffer[self.i],
                end: self.buffer[self.i + 1],
            });

            Some(vec![segment])
        } else {
            None
        }
    }

    fn try_build_segments_during(&mut self) -> Option<Vec<Shape>> {
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
                let segment = Shape::Segment(Segment::CubBez {
                    start: Element {
                        pos: cubbez.start,
                        ..self.buffer[self.i + 1]
                    },
                    cp1: cubbez.cp1,
                    cp2: cubbez.cp2,
                    end: Element {
                        pos: cubbez.end,
                        ..self.buffer[self.i + 2]
                    },
                });

                self.i += 1;

                segments.push(segment);
            } else {
                let segment = Shape::Segment(Segment::Line {
                    start: self.buffer[self.i + 1],
                    end: self.buffer[self.i + 2],
                });

                self.i += 1;

                segments.push(segment);
            }
        }

        Some(segments)
    }

    fn try_build_segments_end(&mut self) -> Vec<Shape> {
        let buffer_last_pos = self.buffer.len().saturating_sub(1);
        let mut segments: Vec<Shape> = vec![];

        while let Some(mut new_segments) = if buffer_last_pos >= self.i + 3 {
            if let Some(cubbez) = CubicBezier::new_w_catmull_rom(
                self.buffer[self.i].pos,
                self.buffer[self.i + 1].pos,
                self.buffer[self.i + 2].pos,
                self.buffer[self.i + 3].pos,
            ) {
                let segment = Shape::Segment(Segment::CubBez {
                    start: Element {
                        pos: cubbez.start,
                        ..self.buffer[self.i + 1]
                    },
                    cp1: cubbez.cp1,
                    cp2: cubbez.cp2,
                    end: Element {
                        pos: cubbez.end,
                        ..self.buffer[self.i + 2]
                    },
                });

                self.i += 1;

                Some(vec![segment])
            } else {
                let segment = Shape::Segment(Segment::Line {
                    start: self.buffer[self.i + 1],
                    end: self.buffer[self.i + 2],
                });

                self.i += 1;

                Some(vec![segment])
            }
        } else if buffer_last_pos > self.i + 2 {
            let segment = Shape::Segment(Segment::Line {
                start: self.buffer[self.i + 1],
                end: self.buffer[self.i + 2],
            });

            self.i += 2;

            Some(vec![segment])
        } else if buffer_last_pos > self.i + 1 {
            let segment = Shape::Segment(Segment::Line {
                start: self.buffer[self.i],
                end: self.buffer[self.i + 1],
            });

            self.i += 2;

            Some(vec![segment])
        } else if buffer_last_pos > self.i {
            let segment = Shape::Segment(Segment::Dot {
                element: self.buffer[self.i],
            });

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
