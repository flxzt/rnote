use std::time::Instant;

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::PenStyle;
use crate::engine::{EngineView, EngineViewMut};
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{DrawOnDocBehaviour, WidgetFlags};

use p2d::bounding_volume::Aabb;
use piet::RenderContext;
use rnote_compose::builders::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use rnote_compose::builders::GridBuilder;
use rnote_compose::builders::{
    CoordSystem2DBuilder, CoordSystem3DBuilder, EllipseBuilder, FociEllipseBuilder, LineBuilder,
    QuadrantCoordSystem2DBuilder, RectangleBuilder, ShapeBuilderBehaviour,
};
use rnote_compose::builders::{CubBezBuilder, QuadBezBuilder, ShapeBuilderType};
use rnote_compose::penevents::{PenEvent, ShortcutKey};

#[derive(Debug)]
enum ShaperState {
    Idle,
    BuildShape {
        builder: Box<dyn ShapeBuilderBehaviour>,
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
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (ShaperState::Idle, PenEvent::Down { element, .. }) => {
                engine_view.pens_config.shaper_config.new_style_seeds();

                match engine_view.pens_config.shaper_config.builder_type {
                    ShapeBuilderType::Line => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(LineBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::Rectangle => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(RectangleBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::Grid => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(GridBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::CoordSystem2D => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CoordSystem2DBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::CoordSystem3D => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CoordSystem3DBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::QuadrantCoordSystem2D => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(QuadrantCoordSystem2DBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::Ellipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(EllipseBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::FociEllipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(FociEllipseBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::QuadBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(QuadBezBuilder::start(element, now)),
                        }
                    }
                    ShapeBuilderType::CubBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CubBezBuilder::start(element, now)),
                        }
                    }
                }

                widget_flags.redraw = true;

                PenProgress::InProgress
            }
            (ShaperState::Idle, _) => PenProgress::Idle,
            (ShaperState::BuildShape { .. }, PenEvent::Cancel) => {
                self.state = ShaperState::Idle;

                widget_flags.redraw = true;
                PenProgress::Finished
            }
            (ShaperState::BuildShape { builder }, event) => {
                // Use Ctrl to temporarily enable/disable constraints when the switch is off/on
                let mut constraints = engine_view.pens_config.shaper_config.constraints.clone();
                constraints.enabled = match event {
                    PenEvent::Down {
                        ref shortcut_keys, ..
                    }
                    | PenEvent::Up {
                        ref shortcut_keys, ..
                    }
                    | PenEvent::Proximity {
                        ref shortcut_keys, ..
                    }
                    | PenEvent::KeyPressed {
                        ref shortcut_keys, ..
                    } => constraints.enabled ^ shortcut_keys.contains(&ShortcutKey::KeyboardCtrl),
                    PenEvent::Text { .. } | PenEvent::Cancel => false,
                };

                match builder.handle_event(event, now, constraints) {
                    ShapeBuilderProgress::InProgress => {
                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    ShapeBuilderProgress::EmitContinue(shapes) => {
                        let mut drawstyle = engine_view
                            .pens_config
                            .shaper_config
                            .gen_style_for_current_options();

                        if !shapes.is_empty() {
                            // Only record if new shapes actually were emitted
                            widget_flags.merge(engine_view.store.record(Instant::now()));
                        }

                        for shape in shapes {
                            let key = engine_view.store.insert_stroke(
                                Stroke::ShapeStroke(ShapeStroke::new(shape, drawstyle.clone())),
                                None,
                            );

                            drawstyle.advance_seed();

                            engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        widget_flags.redraw = true;
                        widget_flags.indicate_changed_store = true;

                        PenProgress::InProgress
                    }
                    ShapeBuilderProgress::Finished(shapes) => {
                        let mut drawstyle = engine_view
                            .pens_config
                            .shaper_config
                            .gen_style_for_current_options();

                        if !shapes.is_empty() {
                            // Only record if new shapes actually were emitted
                            widget_flags.merge(engine_view.store.record(Instant::now()));
                        }

                        if !shapes.is_empty() {
                            engine_view
                                .doc
                                .resize_autoexpand(engine_view.store, engine_view.camera);

                            widget_flags.resize = true;
                            widget_flags.indicate_changed_store = true;
                        }

                        for shape in shapes {
                            let key = engine_view.store.insert_stroke(
                                Stroke::ShapeStroke(ShapeStroke::new(shape, drawstyle.clone())),
                                None,
                            );

                            drawstyle.advance_seed();

                            engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            );
                        }

                        self.state = ShaperState::Idle;

                        widget_flags.redraw = true;

                        PenProgress::Finished
                    }
                }
            }
        };

        (pen_progress, widget_flags)
    }
}

impl DrawOnDocBehaviour for Shaper {
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
