// Imports
use super::buildable::{Buildable, BuilderCreator, BuilderProgress};
use crate::eventresult::EventPropagation;
use crate::penpath::{Element, Segment};
use crate::style::Composer;
use crate::PenEvent;
use crate::{Constraints, EventResult};
use crate::{PenPath, Style};
use ink_stroke_modeler_rs::{ModelerInput, ModelerInputEventType, ModelerParams, StrokeModeler};
use once_cell::sync::Lazy;
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use std::time::{Duration, Instant};

/// Pen path modeled builder.
pub struct PenPathModeledBuilder {
    /// Buffered elements, which are filled up by new pen events and used to build path segments.
    buffer: Vec<Element>,
    prediction_start: Element,
    /// Holding the current prediction. Is recalculated after the modeler is updated with a new element.
    prediction_buffer: Vec<Element>,
    start_time: Instant,
    last_element: Element,
    last_element_time: Instant,
    stroke_modeler: StrokeModeler,
}

impl std::fmt::Debug for PenPathModeledBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModeledPenPathBuilder")
            .field("buffer", &self.buffer)
            .field("prediction_buffer", &self.prediction_buffer)
            .field("start_time", &self.start_time)
            .field("last_element", &self.last_element)
            .field("last_element_time", &self.last_element_time)
            .field("stroke_modeler", &"- no debug impl -")
            .finish()
    }
}

impl BuilderCreator for PenPathModeledBuilder {
    fn start(element: Element, now: Instant) -> Self {
        let mut builder = Self {
            buffer: vec![],
            prediction_start: element,
            prediction_buffer: vec![],
            start_time: now,
            last_element: element,
            last_element_time: now,
            stroke_modeler: StrokeModeler::default(),
        };

        builder.restart(element, now);

        builder
    }
}

impl Buildable for PenPathModeledBuilder {
    type Emit = Segment;

    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        _constraints: Constraints,
    ) -> EventResult<BuilderProgress<Self::Emit>> {
        let progress = match event {
            PenEvent::Down { element, .. } => {
                // kDown is already fed into the modeler when the builder was instantiated (with start())
                self.update_modeler_w_element(element, ModelerInputEventType::kMove, now);

                match self.try_build_segments() {
                    Some(segments) => BuilderProgress::EmitContinue(segments),
                    None => BuilderProgress::InProgress,
                }
            }
            PenEvent::Up { element, .. } => {
                self.update_modeler_w_element(element, ModelerInputEventType::kUp, now);

                let segments = self.build_segments_end();

                BuilderProgress::Finished(segments)
            }
            PenEvent::Proximity { .. } | PenEvent::KeyPressed { .. } | PenEvent::Text { .. } => {
                BuilderProgress::InProgress
            }
            PenEvent::Cancel => BuilderProgress::Finished(vec![]),
        };

        EventResult {
            handled: true,
            propagate: EventPropagation::Stop,
            progress,
        }
    }

    fn bounds(&self, style: &Style, _zoom: f64) -> Option<Aabb> {
        PenPath::try_from_elements(
            self.buffer
                .iter()
                .chain(std::iter::once(&self.prediction_start))
                .chain(self.prediction_buffer.iter())
                .copied(),
        )
        .map(|pp| pp.composed_bounds(style))
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, _zoom: f64) {
        cx.save().unwrap();

        let pen_path = PenPath::try_from_elements(
            self.buffer
                .iter()
                .chain(std::iter::once(&self.prediction_start))
                .chain(self.prediction_buffer.iter())
                .copied(),
        );

        if let Some(pen_path) = pen_path {
            pen_path.draw_composed(cx, style);
        }

        cx.restore().unwrap();
    }
}

static MODELER_PARAMS: Lazy<ModelerParams> = Lazy::new(|| ModelerParams {
    sampling_min_output_rate: 120.0,
    sampling_end_of_stroke_stopping_distance: 0.01,
    sampling_end_of_stroke_max_iterations: 20,
    sampling_max_outputs_per_call: 200,
    stylus_state_modeler_max_input_samples: 20,
    ..ModelerParams::suggested()
});

impl PenPathModeledBuilder {
    fn try_build_segments(&mut self) -> Option<Vec<Segment>> {
        if self.buffer.is_empty() {
            return None;
        }

        Some(
            self.buffer
                .drain(..)
                .map(|el| Segment::LineTo { end: el })
                .collect(),
        )
    }

    fn build_segments_end(&mut self) -> Vec<Segment> {
        self.buffer
            .drain(..)
            .map(|el| Segment::LineTo { end: el })
            .collect()
    }

    fn update_modeler_w_element(
        &mut self,
        element: Element,
        event_type: ModelerInputEventType,
        now: Instant,
    ) {
        if self.last_element == element
            || now.duration_since(self.last_element_time) <= Duration::ZERO
        {
            // Can't feed modeler with duplicate elements or with same or reverse time,
            // would result in `INVALID_ARGUMENT` errors
            return;
        }
        self.last_element = element;

        let n_steps = (now.duration_since(self.last_element_time).as_secs_f64()
            * MODELER_PARAMS.sampling_min_output_rate)
            .ceil() as usize;

        if n_steps > MODELER_PARAMS.sampling_max_outputs_per_call {
            // If the no of outputs the modeler would need to produce exceeds the configured maximum
            // (because the time delta between the last elements is too large), it needs to be restarted.
            tracing::debug!(
                "PenpathModeledBuilder: updating modeler with element failed,
n_steps exceeds configured max outputs per call."
            );

            self.restart(element, now);
        }
        self.last_element_time = now;

        let modeler_input = ModelerInput::new(
            event_type,
            (element.pos[0] as f32, element.pos[1] as f32),
            now.duration_since(self.start_time).as_secs_f64(),
            element.pressure as f32,
        );

        match self.stroke_modeler.update(modeler_input) {
            Ok(results) => self.buffer.extend(results.into_iter().map(|r| {
                let pos = r.pos();
                let pressure = r.pressure();
                Element::new(na::vector![pos.0 as f64, pos.1 as f64], pressure as f64)
            })),
            Err(e) => tracing::error!("Updating stroke modeler with element failed, Err: {e:?}"),
        }

        // The prediction start is the last buffer element (which will get drained)
        if let Some(last) = self.buffer.last() {
            self.prediction_start = *last;
        }

        // When the stroke is finished it is invalid to predict, and the existing prediction should be cleared.
        if event_type == ModelerInputEventType::kUp {
            self.prediction_buffer.clear();
        } else {
            self.prediction_buffer = match self.stroke_modeler.predict() {
                Ok(results) => results
                    .into_iter()
                    .map(|r| {
                        let pos = r.pos();
                        let pressure = r.pressure();
                        Element::new(na::vector![pos.0 as f64, pos.1 as f64], pressure as f64)
                    })
                    .collect::<Vec<Element>>(),
                Err(e) => {
                    tracing::error!("Stroke modeler predict failed, Err: {e:?}");
                    Vec::new()
                }
            }
        }
    }

    fn restart(&mut self, element: Element, now: Instant) {
        self.buffer.clear();
        self.prediction_buffer.clear();
        self.start_time = now;
        self.last_element_time = now;
        self.last_element = element;
        if let Err(e) = self.stroke_modeler.reset_w_params(*MODELER_PARAMS) {
            tracing::error!("Resetting stroke modeler failed while restarting, Err: {e:?}");
            return;
        }

        match self.stroke_modeler.update(ModelerInput::new(
            ModelerInputEventType::kDown,
            (element.pos[0] as f32, element.pos[1] as f32),
            0.0,
            element.pressure as f32,
        )) {
            Ok(results) => {
                self.buffer.extend(results.into_iter().map(|r| {
                    let pos = r.pos();
                    let pressure = r.pressure();
                    Element::new(na::vector![pos.0 as f64, pos.1 as f64], pressure as f64)
                }));
            }
            Err(e) => {
                tracing::error!("Updating stroke modeler failed while restarting, Err: {e:?}")
            }
        }
    }
}
