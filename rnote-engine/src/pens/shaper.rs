use std::time::Instant;

use super::penbehaviour::{PenBehaviour, PenProgress};
use crate::engine::{EngineView, EngineViewMut};
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{DrawOnDocBehaviour, WidgetFlags};

use p2d::bounding_volume::AABB;
use piet::RenderContext;
use rand::{Rng, SeedableRng};
use rnote_compose::builders::shapebuilderbehaviour::{ShapeBuilderCreator, ShapeBuilderProgress};
use rnote_compose::builders::GridBuilder;
use rnote_compose::builders::{
    ConstraintRatio, Constraints, CubBezBuilder, QuadBezBuilder, ShapeBuilderType,
};
use rnote_compose::builders::{
    CoordSystem2DBuilder, CoordSystem3DBuilder, EllipseBuilder, FociEllipseBuilder, LineBuilder,
    QuadrantCoordSystem2DBuilder, RectangleBuilder, ShapeBuilderBehaviour,
};
use rnote_compose::penhelpers::{PenEvent, ShortcutKey};
use rnote_compose::style::rough::RoughOptions;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::Style;
use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "shaper_style")]
pub enum ShaperStyle {
    #[serde(rename = "smooth")]
    Smooth = 0,
    #[serde(rename = "rough")]
    Rough,
}

impl Default for ShaperStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

impl TryFrom<u32> for ShaperStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("ShaperStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(Debug)]
enum ShaperState {
    Idle,
    BuildShape {
        builder: Box<dyn ShapeBuilderBehaviour>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "shaper")]
pub struct Shaper {
    #[serde(rename = "builder_type")]
    pub builder_type: ShapeBuilderType,
    #[serde(rename = "style")]
    pub style: ShaperStyle,
    #[serde(rename = "smooth_options")]
    pub smooth_options: SmoothOptions,
    #[serde(rename = "rough_options")]
    pub rough_options: RoughOptions,
    #[serde(rename = "constraints")]
    pub constraints: Constraints,
    #[serde(skip)]
    state: ShaperState,
}

impl Clone for Shaper {
    fn clone(&self) -> Self {
        Self {
            builder_type: self.builder_type,
            style: self.style,
            smooth_options: self.smooth_options.clone(),
            rough_options: self.rough_options.clone(),
            constraints: self.constraints.clone(),
            state: ShaperState::Idle,
        }
    }
}

impl Default for Shaper {
    fn default() -> Self {
        let mut constraints = Constraints::default();
        constraints.ratios.insert(ConstraintRatio::OneToOne);
        constraints.ratios.insert(ConstraintRatio::Horizontal);
        constraints.ratios.insert(ConstraintRatio::Vertical);

        Self {
            builder_type: ShapeBuilderType::default(),
            style: ShaperStyle::default(),
            smooth_options: SmoothOptions::default(),
            rough_options: RoughOptions::default(),
            constraints,
            state: ShaperState::Idle,
        }
    }
}

impl PenBehaviour for Shaper {
    fn handle_event(
        &mut self,
        event: PenEvent,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (ShaperState::Idle, PenEvent::Down { element, .. }) => {
                self.new_style_seeds();

                match self.builder_type {
                    ShapeBuilderType::Line => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(LineBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::Rectangle => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(RectangleBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::Grid => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(GridBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::CoordSystem2D => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CoordSystem2DBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::CoordSystem3D => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CoordSystem3DBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::QuadrantCoordSystem2D => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(QuadrantCoordSystem2DBuilder::start(
                                element,
                                Instant::now(),
                            )),
                        }
                    }
                    ShapeBuilderType::Ellipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(EllipseBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::FociEllipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(FociEllipseBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::QuadBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(QuadBezBuilder::start(element, Instant::now())),
                        }
                    }
                    ShapeBuilderType::CubBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CubBezBuilder::start(element, Instant::now())),
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
                let mut constraints = self.constraints.clone();
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

                match builder.handle_event(event, Instant::now(), constraints) {
                    ShapeBuilderProgress::InProgress => {
                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    ShapeBuilderProgress::EmitContinue(shapes) => {
                        let mut drawstyle = self.gen_style_for_current_options();

                        if !shapes.is_empty() {
                            // Only record if new shapes actually were emitted
                            widget_flags.merge_with_other(engine_view.store.record());
                        }

                        for shape in shapes {
                            let key = engine_view.store.insert_stroke(
                                Stroke::ShapeStroke(ShapeStroke::new(shape, drawstyle.clone())),
                                None,
                            );

                            drawstyle.advance_seed();

                            if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            ) {
                                log::error!("regenerate_rendering_for_stroke() failed after inserting new line, Err: {e:?}");
                            }
                        }

                        widget_flags.redraw = true;
                        widget_flags.indicate_changed_store = true;

                        PenProgress::InProgress
                    }
                    ShapeBuilderProgress::Finished(shapes) => {
                        let mut drawstyle = self.gen_style_for_current_options();

                        if !shapes.is_empty() {
                            // Only record if new shapes actually were emitted
                            widget_flags.merge_with_other(engine_view.store.record());
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

                            if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            ) {
                                log::error!("regenerate_rendering_for_stroke() failed after inserting new shape, Err: {e:?}");
                            }
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
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<AABB> {
        let style = self.gen_style_for_current_options();

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
        let style = self.gen_style_for_current_options();

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

impl Shaper {
    pub const STROKE_WIDTH_MIN: f64 = 0.1;
    pub const STROKE_WIDTH_MAX: f64 = 500.0;

    fn new_style_seeds(&mut self) {
        // A new seed for new shapes
        let seed = Some(rand_pcg::Pcg64::from_entropy().gen());
        self.rough_options.seed = seed;
    }

    fn gen_style_for_current_options(&self) -> Style {
        match &self.style {
            ShaperStyle::Smooth => {
                let options = self.smooth_options.clone();

                Style::Smooth(options)
            }
            ShaperStyle::Rough => {
                let options = self.rough_options.clone();

                Style::Rough(options)
            }
        }
    }
}
