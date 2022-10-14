use ink_stroke_modeler_rs::{
    ModelerInput, ModelerInputEventType, StrokeModeler, StrokeModelerParams,
};
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
pub(crate) enum ModeledPenPathBuilderState {
    Start,
    During,
}

/// The pen path builder
pub struct ModeledPenPathBuilder {
    pub(crate) state: ModeledPenPathBuilderState,
    /// Buffered elements, which are filled up by new pen events and used to try to build path segments
    pub buffer: VecDeque<Element>,
    start_time: Instant,
    last_fed_element: Element,
    stroke_modeler: StrokeModeler,
}

impl std::fmt::Debug for ModeledPenPathBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModeledPenPathBuilder")
            .field("state", &self.state)
            .field("buffer", &self.buffer)
            .field("start_time", &self.start_time)
            .field("last_fed_element", &self.last_fed_element)
            .field("stroke_modeler", &".. no debug impl ..")
            .finish()
    }
}

impl ShapeBuilderCreator for ModeledPenPathBuilder {
    fn start(element: Element, now: Instant) -> Self {
        let mut buffer = VecDeque::new();

        let modeler_params = StrokeModelerParams {
            // Increase the time that can pass between inputs (happens when resting on a position while drawing)
            sampling_max_outputs_per_call: 100,
            ..Default::default()
        };

        let mut stroke_modeler = StrokeModeler::new(modeler_params);

        buffer.extend(
            stroke_modeler
                .update(ModelerInput::new(
                    ModelerInputEventType::kDown,
                    (element.pos[0] as f32, element.pos[1] as f32),
                    0.0,
                    element.pressure as f32,
                    0.0,
                    0.0,
                ))
                .into_iter()
                .map(|r| {
                    let pos = r.get_pos();
                    let pressure = r.get_pressure();

                    Element::new(na::vector![pos.0 as f64, pos.1 as f64], pressure as f64)
                }),
        );

        Self {
            state: ModeledPenPathBuilderState::Start,
            buffer,
            start_time: now,
            last_fed_element: element,
            stroke_modeler,
        }
    }
}

impl ShapeBuilderBehaviour for ModeledPenPathBuilder {
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
            (ModeledPenPathBuilderState::Start, PenEvent::Down { element, .. }) => {
                // kDown is already fed when instanciating the builder
                self.update_modeler_w_element(element, ModelerInputEventType::kMove);

                match self.try_build_segments() {
                    Some(shapes) => BuilderProgress::EmitContinue(shapes),
                    None => BuilderProgress::InProgress,
                }
            }
            (ModeledPenPathBuilderState::During, PenEvent::Down { element, .. }) => {
                self.update_modeler_w_element(element, ModelerInputEventType::kMove);

                match self.try_build_segments() {
                    Some(shapes) => BuilderProgress::EmitContinue(shapes),
                    None => BuilderProgress::InProgress,
                }
            }
            (_, PenEvent::Up { element, .. }) => {
                self.update_modeler_w_element(element, ModelerInputEventType::kUp);

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
            ModeledPenPathBuilderState::Start => self
                .buffer
                .iter()
                .zip(self.buffer.iter().skip(1))
                .map(|(start, end)| Segment::Line {
                    start: *start,
                    end: *end,
                })
                .collect::<PenPath>(),
            ModeledPenPathBuilderState::During => {
                let prediction = self
                    .stroke_modeler
                    .predict()
                    .into_iter()
                    .map(|r| {
                        let pos = r.get_pos();
                        let pressure = r.get_pressure();

                        Element::new(na::vector![pos.0 as f64, pos.1 as f64], pressure as f64)
                    })
                    .collect::<Vec<Element>>();

                prediction
                    .iter()
                    .zip(prediction.iter().skip(1))
                    .map(|(start, end)| Segment::Line {
                        start: *start,
                        end: *end,
                    })
                    .collect::<PenPath>()
            }
        };

        /*
               // Change prediction stroke color for debugging
               let mut style = style.clone();
               match style {
                   Style::Smooth(ref mut smooth_options) => {
                       smooth_options.stroke_color = Some(crate::Color::RED)
                   }
                   _ => {}
               }
        */

        penpath.draw_composed(cx, style);
        cx.restore().unwrap();
    }
}

impl ModeledPenPathBuilder {
    fn try_build_segments(&mut self) -> Option<Vec<Shape>> {
        if self.buffer.len() < 2 {
            return None;
        }
        let mut segments = vec![];

        while self.buffer.len() > 2 {
            self.state = ModeledPenPathBuilderState::During;

            segments.push(Shape::Segment(Segment::Line {
                start: self.buffer[0],
                end: self.buffer[1],
            }));

            self.buffer.pop_front();
        }

        Some(segments)
    }

    fn update_modeler_w_element(&mut self, element: Element, event_type: ModelerInputEventType) {
        if self.last_fed_element.pos == element.pos {
            // Can't feed modeler with duplicate elements, will result in `INVALID_ARGUMENT` errors
            return;
        } else {
            self.last_fed_element = element;
        }

        let modeler_input = ModelerInput::new(
            event_type,
            (element.pos[0] as f32, element.pos[1] as f32),
            self.start_time.elapsed().as_secs_f64(),
            element.pressure as f32,
            0.0,
            0.0,
        );

        //log::debug!("{modeler_input}");

        self.buffer.extend(
            self.stroke_modeler
                .update(modeler_input)
                .into_iter()
                .map(|r| {
                    let pos = r.get_pos();
                    let pressure = r.get_pressure();

                    Element::new(na::vector![pos.0 as f64, pos.1 as f64], pressure as f64)
                }),
        );
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.state = ModeledPenPathBuilderState::Start;
    }
}
