// Imports
use super::PenBehaviour;
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{DrawableOnDoc, WidgetFlags};
use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::builders::buildable::{Buildable, BuilderCreator, BuilderProgress};
use rnote_compose::builders::ParabolaBuilder;
use rnote_compose::builders::{ArrowBuilder, GridBuilder, PolygonBuilder, PolylineBuilder};
use rnote_compose::builders::{
    CoordSystem2DBuilder, CoordSystem3DBuilder, CubBezBuilder, EllipseBuilder, FociEllipseBuilder,
    LineBuilder, QuadBezBuilder, QuadrantCoordSystem2DBuilder, RectangleBuilder, ShapeBuilderType,
};
use rnote_compose::eventresult::{EventPropagation, EventResult};
use rnote_compose::penevent::{KeyboardKey, ModifierKey, PenEvent, PenProgress};
use rnote_compose::penpath::Element;
use rnote_compose::Shape;
use std::time::Instant;

#[derive(Debug)]
enum ShaperState {
    Idle,
    BuildShape {
        builder: Box<dyn Buildable<Emit = Shape>>,
    },
}

#[derive(Debug)]
pub struct Shaper {
    state: ShaperState,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            state: ShaperState::Idle,
        }
    }
}

impl PenBehaviour for Shaper {
    fn init(&mut self, _engine_view: &EngineView) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn deinit(&mut self) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn style(&self) -> PenStyle {
        PenStyle::Shaper
    }

    fn update_state(&mut self, _engine_view: &mut EngineViewMut) -> WidgetFlags {
        WidgetFlags::default()
    }

    fn handle_event(
        &mut self,
        event: PenEvent,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (EventResult<PenProgress>, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let event_result = match (&mut self.state, event) {
            (ShaperState::Idle, PenEvent::Down { element, .. }) => {
                engine_view.pens_config.shaper_config.new_style_seeds();

                self.state = ShaperState::BuildShape {
                    builder: new_builder(
                        engine_view.pens_config.shaper_config.builder_type,
                        element,
                        now,
                    ),
                };

                EventResult {
                    handled: true,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::InProgress,
                }
            }
            (ShaperState::Idle, _) => EventResult {
                handled: false,
                propagate: EventPropagation::Proceed,
                progress: PenProgress::Idle,
            },
            (ShaperState::BuildShape { .. }, PenEvent::Cancel) => {
                self.state = ShaperState::Idle;

                EventResult {
                    handled: false,
                    propagate: EventPropagation::Stop,
                    progress: PenProgress::Finished,
                }
            }
            (ShaperState::BuildShape { builder }, event) => {
                // Use Ctrl to temporarily enable/disable constraints when the switch is off/on
                let mut constraints = engine_view.pens_config.shaper_config.constraints.clone();
                constraints.enabled = match event {
                    PenEvent::Down {
                        ref modifier_keys, ..
                    }
                    | PenEvent::Up {
                        ref modifier_keys, ..
                    }
                    | PenEvent::Proximity {
                        ref modifier_keys, ..
                    }
                    | PenEvent::KeyPressed {
                        ref modifier_keys, ..
                    } => constraints.enabled ^ modifier_keys.contains(&ModifierKey::KeyboardCtrl),
                    PenEvent::Text { .. } | PenEvent::Cancel => false,
                };
                let builder_result = builder.handle_event(event.clone(), now, constraints);
                let handled = builder_result.handled;
                let propagate = builder_result.propagate;

                let mut progress = match builder_result.progress {
                    BuilderProgress::InProgress => PenProgress::InProgress,
                    BuilderProgress::EmitContinue(shapes) => {
                        let mut style = engine_view
                            .pens_config
                            .shaper_config
                            .gen_style_for_current_options();
                        let shapes_emitted = !shapes.is_empty();

                        for shape in shapes {
                            let key = engine_view.store.insert_stroke(
                                Stroke::ShapeStroke(ShapeStroke::new(shape, style.clone())),
                                None,
                            );
                            style.advance_seed();
                            engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        if shapes_emitted {
                            widget_flags |= engine_view.store.record(Instant::now());
                            widget_flags.store_modified = true;
                        }
                        PenProgress::InProgress
                    }
                    BuilderProgress::Finished(shapes) => {
                        let mut style = engine_view
                            .pens_config
                            .shaper_config
                            .gen_style_for_current_options();

                        let shapes_emitted = !shapes.is_empty();
                        for shape in shapes {
                            let key = engine_view.store.insert_stroke(
                                Stroke::ShapeStroke(ShapeStroke::new(shape, style.clone())),
                                None,
                            );
                            style.advance_seed();
                            engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        self.state = ShaperState::Idle;

                        if shapes_emitted {
                            widget_flags |= engine_view
                                .document
                                .resize_autoexpand(engine_view.store, engine_view.camera)
                                | engine_view.store.record(Instant::now());
                            widget_flags.store_modified = true;
                        }
                        PenProgress::Finished
                    }
                };

                // When esc is pressed, reset state
                if let PenEvent::KeyPressed {
                    keyboard_key,
                    modifier_keys,
                } = event
                {
                    if keyboard_key == KeyboardKey::Escape && modifier_keys.is_empty() {
                        self.state = ShaperState::Idle;
                        progress = PenProgress::Finished;
                    }
                }

                EventResult {
                    handled,
                    propagate,
                    progress,
                }
            }
        };

        (event_result, widget_flags)
    }
}

impl DrawableOnDoc for Shaper {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        let style = engine_view
            .pens_config
            .shaper_config
            .gen_style_for_current_options();

        match &self.state {
            ShaperState::Idle => None,
            ShaperState::BuildShape { builder } => {
                builder.bounds(&style, engine_view.camera.total_zoom())
            }
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let style = engine_view
            .pens_config
            .shaper_config
            .gen_style_for_current_options();

        match &self.state {
            ShaperState::Idle => {}
            ShaperState::BuildShape { builder } => {
                builder.draw_styled(cx, &style, engine_view.camera.total_zoom())
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

fn new_builder(
    builder_type: ShapeBuilderType,
    element: Element,
    now: Instant,
) -> Box<dyn Buildable<Emit = Shape>> {
    match builder_type {
        ShapeBuilderType::Arrow => Box::new(ArrowBuilder::start(element, now)),
        ShapeBuilderType::Line => Box::new(LineBuilder::start(element, now)),
        ShapeBuilderType::Rectangle => Box::new(RectangleBuilder::start(element, now)),
        ShapeBuilderType::Grid => Box::new(GridBuilder::start(element, now)),
        ShapeBuilderType::CoordSystem2D => Box::new(CoordSystem2DBuilder::start(element, now)),
        ShapeBuilderType::CoordSystem3D => Box::new(CoordSystem3DBuilder::start(element, now)),
        ShapeBuilderType::QuadrantCoordSystem2D => {
            Box::new(QuadrantCoordSystem2DBuilder::start(element, now))
        }
        ShapeBuilderType::Ellipse => Box::new(EllipseBuilder::start(element, now)),
        ShapeBuilderType::FociEllipse => Box::new(FociEllipseBuilder::start(element, now)),
        ShapeBuilderType::QuadBez => Box::new(QuadBezBuilder::start(element, now)),
        ShapeBuilderType::CubBez => Box::new(CubBezBuilder::start(element, now)),
        ShapeBuilderType::Polyline => Box::new(PolylineBuilder::start(element, now)),
        ShapeBuilderType::Polygon => Box::new(PolygonBuilder::start(element, now)),
        ShapeBuilderType::Parabola => Box::new(ParabolaBuilder::start(element, now)),
    }
}
