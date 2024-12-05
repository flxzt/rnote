// Imports
use super::ToolsState;
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use p2d::bounding_volume::BoundingVolume;
use piet::RenderContext;
use rnote_compose::builders::buildable::Buildable;
use rnote_compose::builders::buildable::BuilderCreator;
use rnote_compose::builders::buildable::BuilderProgress;
use rnote_compose::builders::PenPathBuilderType;
use rnote_compose::builders::PenPathCurvedBuilder;
use rnote_compose::builders::PenPathModeledBuilder;
use rnote_compose::builders::PenPathSimpleBuilder;
use rnote_compose::color;
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::ext::AabbExt;
use rnote_compose::penevent::{PenEvent, PenProgress};
use rnote_compose::penpath::Element;
use rnote_compose::penpath::Segment;
use rnote_compose::shapes::Shapeable;
use rnote_compose::Constraints;
use rnote_compose::PenPath;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
pub struct LaserTool {
    state: ToolsState,
    path_builder: Option<Box<dyn Buildable<Emit = Segment>>>,
    pen_paths: Vec<PenPath>,
    fade_start_time: Option<Instant>,
    opacity: u8,
}

impl Default for LaserTool {
    fn default() -> Self {
        Self {
            state: ToolsState::default(),
            path_builder: None,
            pen_paths: Vec::new(),
            fade_start_time: None,
            opacity: u8::MAX,
        }
    }
}

impl LaserTool {
    const FULL_FADE_DURATION: Duration = Duration::from_secs(1);

    const OUTER_STROKE_WIDTH: f64 = 6.0;
    const INNER_STROKE_WIDTH: f64 = 1.0;

    const INNER_STROKE_COLOR: piet::Color = color::GNOME_BRIGHTS[1];
    const OUTER_STROKE_COLOR: piet::Color = color::GNOME_REDS[1];

    const STYLE: piet::StrokeStyle = piet::StrokeStyle::new()
        .line_join(piet::LineJoin::Round)
        .line_cap(piet::LineCap::Round);

    pub fn add_new_stroke(&mut self, element: Element) {
        self.pen_paths.push(PenPath::new(element));
        self.stop_fade();
    }

    pub fn extend_last_stroke(&mut self, progress: BuilderProgress<Segment>) {
        if let Some(last_stroke) = self.pen_paths.last_mut() {
            match progress {
                BuilderProgress::InProgress => {}
                BuilderProgress::EmitContinue(segments) | BuilderProgress::Finished(segments) => {
                    last_stroke.extend(segments);
                }
            };
        }
    }

    pub fn start_fade(&mut self, now: Instant) {
        self.fade_start_time = Some(now);
    }

    pub fn stop_fade(&mut self) {
        self.fade_start_time = None;
        self.opacity = u8::MAX;
    }

    /// Returns `Some(bool)` if the fade is in progress, otherwise `None`.
    pub fn has_fully_faded(&self) -> Option<bool> {
        self.fade_start_time
            .map(|time| time.elapsed() >= LaserTool::FULL_FADE_DURATION)
    }

    pub fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let widget_flags = WidgetFlags::default();

        let event_result = match (&mut self.state, &event) {
            (ToolsState::Idle, PenEvent::Down { element, .. }) => {
                self.add_new_stroke(*element);

                self.path_builder = Some(match engine_view.pens_config.brush_config.builder_type {
                    PenPathBuilderType::Simple => {
                        Box::new(PenPathSimpleBuilder::start(*element, now))
                    }
                    PenPathBuilderType::Curved => {
                        Box::new(PenPathCurvedBuilder::start(*element, now))
                    }
                    PenPathBuilderType::Modeled => {
                        Box::new(PenPathModeledBuilder::start(*element, now))
                    }
                });

                self.state = ToolsState::Active;

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (ToolsState::Idle, _) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (ToolsState::Active, PenEvent::Down { .. }) => {
                if let Some(builder) = &mut self.path_builder {
                    let builder_result = builder.handle_event(event, now, Constraints::default());

                    self.extend_last_stroke(builder_result.progress);
                }

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (ToolsState::Active, PenEvent::Up { .. }) => {
                let mut progress = PenProgress::Finished;

                if let Some(builder) = &mut self.path_builder {
                    let builder_result = builder.handle_event(event, now, Constraints::default());

                    self.extend_last_stroke(builder_result.progress);
                    self.start_fade(now);

                    engine_view.animation.claim_frame();
                    progress = PenProgress::InProgress;
                }

                self.reset(false);

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress,
                }
            }
            (ToolsState::Active, PenEvent::Proximity { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            (ToolsState::Active, PenEvent::KeyPressed { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
            (ToolsState::Active, PenEvent::Cancel) => {
                self.reset(true);

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (ToolsState::Active, PenEvent::Text { .. }) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::InProgress,
            },
        };

        (event_result, widget_flags)
    }

    pub fn handle_animation_frame(&mut self, engine_view: &mut EngineViewMut, optimize_epd: bool) {
        let Some(faded) = self.has_fully_faded() else {
            return;
        };

        if faded {
            self.reset(true);
        } else {
            if !optimize_epd {
                let transparency = self
                    .fade_start_time
                    .unwrap() // Never fails because `has_fully_faded` has not returned `None`.
                    .elapsed()
                    .div_duration_f64(Self::FULL_FADE_DURATION)
                    .clamp(0.0, 1.0);

                self.opacity = ((1.0 - transparency) * 255.0) as u8;
            }

            engine_view.animation.claim_frame();
        }
    }

    fn reset(&mut self, clear: bool) {
        self.state = ToolsState::Idle;
        self.path_builder = None;

        if clear {
            self.pen_paths.clear();
            self.stop_fade();
        }
    }
}

impl DrawableOnDoc for LaserTool {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        if self.has_fully_faded().is_some_and(|faded| faded) {
            return None;
        }

        let strokes = self.pen_paths.iter();

        strokes
            .map(|path| path.bounds())
            .reduce(|acc, path| acc.merged(&path))
            .map(|bounds| {
                bounds.extend_by(na::Vector2::repeat(
                    Self::OUTER_STROKE_WIDTH / engine_view.camera.total_zoom(),
                ))
            })
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let total_zoom = engine_view.camera.total_zoom();

        for pen_path in &self.pen_paths {
            let bez_path = pen_path.to_kurbo_flattened(0.5);

            cx.stroke_styled(
                &bez_path,
                &Self::OUTER_STROKE_COLOR.with_a8(self.opacity),
                Self::OUTER_STROKE_WIDTH / total_zoom,
                &LaserTool::STYLE,
            );

            cx.stroke_styled(
                &bez_path,
                &Self::INNER_STROKE_COLOR.with_a8(self.opacity),
                Self::INNER_STROKE_WIDTH / total_zoom,
                &LaserTool::STYLE,
            );
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
