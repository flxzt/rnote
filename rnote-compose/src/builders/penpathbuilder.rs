use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::penpath::{Element, Segment};
use crate::shapes::CubicBezier;
use crate::PenEvent;

use super::ShapeBuilderBehaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum PenPathBuilderState {
    Start,
    During,
}

impl Default for PenPathBuilderState {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "penpathbuilder")]
/// The pen path builder
pub struct PenPathBuilder {
    #[serde(rename = "state")]
    pub(crate) state: PenPathBuilderState,
    #[serde(rename = "buffer")]
    /// Buffered elements, which is filled up by new pen events and used to try to build path segments
    pub buffer: VecDeque<Element>,
}

impl ShapeBuilderBehaviour for PenPathBuilder {
    type BuildedShape = Segment;

    fn handle_event(&mut self, event: PenEvent) -> Option<Vec<Self::BuildedShape>> {
        /*         log::debug!(
            "event: {:?}; buffer.len(): {}, state: {:?}",
            event,
            self.buffer.len(),
            self.state
        ); */

        match (&mut self.state, event) {
            (
                PenPathBuilderState::Start,
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                self.buffer.push_back(element);

                self.try_build_segment_start()
            }
            (
                PenPathBuilderState::During,
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                self.buffer.push_back(element);

                self.try_build_segment_during()
            }
            (
                _,
                PenEvent::Up {
                    element,
                    shortcut_key: _,
                },
            ) => {
                self.buffer.push_back(element);

                self.try_build_segment_end()
            }
            (_, PenEvent::Proximity { .. }) => None,
            (_, PenEvent::Cancel) => {
                self.reset();

                None
            }
        }
    }
}

impl PenPathBuilder {
    fn try_build_segment_start(&mut self) -> Option<Vec<Segment>> {
        let segments = match self.buffer.len() {
            0 => None,
            1 => Some(vec![Segment::Dot {
                element: self.buffer[0],
            }]),
            2 => Some(vec![Segment::Line {
                start: self.buffer[0],
                end: self.buffer[1],
            }]),
            3.. => {
                // Here we have enough elements to switch into during state
                self.state = PenPathBuilderState::During;

                Some(vec![Segment::Line {
                    start: self.buffer[0],
                    end: self.buffer[1],
                }])
            }
            _ => None,
        };

        segments
    }

    fn try_build_segment_during(&mut self) -> Option<Vec<Segment>> {
        let segment = match self.buffer.len() {
            4.. => {
                if let Some(cubbez) = CubicBezier::new_w_catmull_rom(
                    self.buffer[0].pos,
                    self.buffer[1].pos,
                    self.buffer[2].pos,
                    self.buffer[3].pos,
                ) {
                    let segment = Segment::CubBez {
                        start: Element {
                            pos: cubbez.start,
                            ..self.buffer[1]
                        },
                        cp1: cubbez.cp1,
                        cp2: cubbez.cp2,
                        end: Element {
                            pos: cubbez.end,
                            ..self.buffer[2]
                        },
                    };

                    self.buffer.pop_front();

                    Some(vec![segment])
                } else {
                    let segment = Segment::Line {
                        start: self.buffer[1],
                        end: self.buffer[2],
                    };

                    self.buffer.pop_front();

                    Some(vec![segment])
                }
            }
            _ => None,
        };

        segment
    }

    fn try_build_segment_end(&mut self) -> Option<Vec<Segment>> {
        let mut segments: Option<Vec<Segment>> = None;

        while let Some(mut new_segments) = match self.buffer.len() {
            0 => None,
            1 => Some(vec![Segment::Dot {
                element: self.buffer.remove(0).unwrap(),
            }]),
            2 => {
                let elements = self.buffer.drain(0..2).collect::<Vec<Element>>();
                Some(vec![Segment::Line {
                    start: elements[0],
                    end: elements[1],
                }])
            }
            3 => {
                let elements = self.buffer.drain(0..3).collect::<Vec<Element>>();
                Some(vec![Segment::Line {
                    start: elements[1],
                    end: elements[2],
                }])
            }
            4.. => {
                if let Some(cubbez) = CubicBezier::new_w_catmull_rom(
                    self.buffer[0].pos,
                    self.buffer[1].pos,
                    self.buffer[2].pos,
                    self.buffer[3].pos,
                ) {
                    let segment = Segment::CubBez {
                        start: Element {
                            pos: cubbez.start,
                            ..self.buffer[1]
                        },
                        cp1: cubbez.cp1,
                        cp2: cubbez.cp2,
                        end: Element {
                            pos: cubbez.end,
                            ..self.buffer[2]
                        },
                    };

                    // Only remove one element as more segments can be build
                    self.buffer.pop_front();

                    Some(vec![segment])
                } else {
                    let segment = Segment::Line {
                        start: self.buffer[1],
                        end: self.buffer[2],
                    };

                    self.buffer.pop_front();

                    Some(vec![segment])
                }
            }
            _ => None,
        } {
            if let Some(ref mut segments) = segments {
                segments.append(&mut new_segments);
            } else {
                segments = Some(new_segments);
            }
        }

        self.reset();

        segments
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.state = PenPathBuilderState::Start;
    }
}
