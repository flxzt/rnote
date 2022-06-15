use super::penbehaviour::{PenBehaviour, PenProgress};
use crate::engine::{EngineView, EngineViewMut};
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{DrawOnDocBehaviour, WidgetFlags};

use p2d::bounding_volume::AABB;
use piet::RenderContext;
use rand::{Rng, SeedableRng};
use rnote_compose::builders::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use rnote_compose::builders::{Constraints, CubBezBuilder, QuadBezBuilder, ShapeBuilderType};
use rnote_compose::builders::{
    EllipseBuilder, FociEllipseBuilder, LineBuilder, RectangleBuilder, ShapeBuilderBehaviour,
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

impl Default for Shaper {
    fn default() -> Self {
        let mut smooth_options = SmoothOptions::default();
        let mut rough_options = RoughOptions::default();
        smooth_options.stroke_width = Self::STROKE_WIDTH_DEFAULT;
        rough_options.stroke_width = Self::STROKE_WIDTH_DEFAULT;

        Self {
            builder_type: ShapeBuilderType::default(),
            style: ShaperStyle::default(),
            smooth_options,
            rough_options,
            constraints: Constraints::default(),
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
                // A new seed for a new shape
                let seed = Some(rand_pcg::Pcg64::from_entropy().gen());
                self.rough_options.seed = seed;

                match self.builder_type {
                    ShapeBuilderType::Line => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(LineBuilder::start(element)),
                        }
                    }
                    ShapeBuilderType::Rectangle => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(RectangleBuilder::start(element)),
                        }
                    }
                    ShapeBuilderType::Ellipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(EllipseBuilder::start(element)),
                        }
                    }
                    ShapeBuilderType::FociEllipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(FociEllipseBuilder::start(element)),
                        }
                    }
                    ShapeBuilderType::QuadBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(QuadBezBuilder::start(element)),
                        }
                    }
                    ShapeBuilderType::CubBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CubBezBuilder::start(element)),
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
                    PenEvent::Cancel => false,
                };

                match builder.handle_event(event, constraints) {
                    BuilderProgress::InProgress => {
                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    BuilderProgress::EmitContinue(shapes) => {
                        let drawstyle = self.gen_style_for_current_options();

                        if !shapes.is_empty() {
                            // Only record if new shapes actually were emitted
                            widget_flags.merge_with_other(engine_view.store.record());
                        }

                        for shape in shapes {
                            let key = engine_view.store.insert_stroke(Stroke::ShapeStroke(
                                ShapeStroke::new(shape, drawstyle.clone()),
                            ));
                            if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            ) {
                                log::error!("regenerate_rendering_for_stroke() failed after inserting new line, Err {}", e);
                            }
                        }

                        widget_flags.redraw = true;
                        widget_flags.indicate_changed_store = true;

                        PenProgress::InProgress
                    }
                    BuilderProgress::Finished(shapes) => {
                        let drawstyle = self.gen_style_for_current_options();

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
                            let key = engine_view.store.insert_stroke(Stroke::ShapeStroke(
                                ShapeStroke::new(shape, drawstyle.clone()),
                            ));
                            if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                                key,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            ) {
                                log::error!("regenerate_rendering_for_stroke() failed after inserting new shape, Err {}", e);
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
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
        let style = self.gen_style_for_current_options();

        match &self.state {
            ShaperState::Idle => {}
            ShaperState::BuildShape { builder } => {
                builder.draw_styled(cx, &style, engine_view.camera.total_zoom())
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

impl Shaper {
    pub const INPUT_OVERSHOOT: f64 = 30.0;

    pub const STROKE_WIDTH_MIN: f64 = 1.0;
    pub const STROKE_WIDTH_MAX: f64 = 500.0;
    pub const STROKE_WIDTH_DEFAULT: f64 = 2.0;

    pub fn gen_style_for_current_options(&self) -> Style {
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
