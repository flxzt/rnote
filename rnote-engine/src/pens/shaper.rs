use super::penbehaviour::{PenBehaviour, PenProgress};
use super::AudioPlayer;
use crate::sheet::Sheet;
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{Camera, DrawOnSheetBehaviour, StrokeStore, SurfaceFlags};

use gtk4::glib;
use p2d::bounding_volume::AABB;
use piet::RenderContext;
use rand::{Rng, SeedableRng};
use rnote_compose::builders::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use rnote_compose::builders::{ConstraintRatio, CubBezBuilder, QuadBezBuilder, ShapeBuilderType};
use rnote_compose::builders::{
    EllipseBuilder, FociEllipseBuilder, LineBuilder, RectangleBuilder, ShapeBuilderBehaviour,
};
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::style::rough::RoughOptions;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::Style;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "shaper_style")]
pub enum ShaperStyle {
    #[serde(rename = "smooth")]
    Smooth,
    #[serde(rename = "rough")]
    Rough,
}

impl Default for ShaperStyle {
    fn default() -> Self {
        Self::Smooth
    }
}

#[derive(Debug)]
enum ShaperState {
    Idle,
    BuildShape {
        builder: Box<dyn ShapeBuilderBehaviour>,
    },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, glib::Enum)]
#[enum_type(name = "ShaperConstraintRatio")]
#[serde(rename = "shaper_constraint_ratio")]
pub enum ShaperConstraintRatio {
    #[enum_value(name = "Disabled", nick = "disabled")]
    #[serde(rename = "disabled")]
    Disabled,
    #[enum_value(name = "1:1", nick = "one_to_one")]
    #[serde(rename = "one_to_one")]
    OneToOne,
    #[enum_value(name = "3:2", nick = "three_to_two")]
    #[serde(rename = "three_to_two")]
    ThreeToTwo,
    #[enum_value(name = "Golden ratio", nick = "golden")]
    #[serde(rename = "golden")]
    Golden,
}

impl From<glib::GString> for ShaperConstraintRatio {
    fn from(nick: glib::GString) -> Self {
        match nick.to_string().as_str() {
            "disabled" => ShaperConstraintRatio::Disabled,
            "one_to_one" => ShaperConstraintRatio::OneToOne,
            "three_to_two" => ShaperConstraintRatio::ThreeToTwo,
            "golden" => ShaperConstraintRatio::Golden,
            _ => unreachable!(),
        }
    }
}

impl From<ShaperConstraintRatio> for ConstraintRatio {
    fn from(r: ShaperConstraintRatio) -> Self {
        match r {
            ShaperConstraintRatio::Disabled => ConstraintRatio::Disabled,
            ShaperConstraintRatio::OneToOne => ConstraintRatio::OneToOne,
            ShaperConstraintRatio::ThreeToTwo => ConstraintRatio::ThreeToTwo,
            ShaperConstraintRatio::Golden => ConstraintRatio::Golden,
        }
    }
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

    #[serde(skip)]
    ratio: ShaperConstraintRatio,
    #[serde(skip)]
    state: ShaperState,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            builder_type: ShapeBuilderType::default(),
            style: ShaperStyle::default(),
            smooth_options: SmoothOptions::default(),
            rough_options: RoughOptions::default(),
            ratio: ShaperConstraintRatio::Disabled,
            state: ShaperState::Idle,
        }
    }
}

impl PenBehaviour for Shaper {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _sheet: &mut Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags) {
        let mut surface_flags = SurfaceFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (ShaperState::Idle, PenEvent::Down { element, .. }) => {
                store.record();

                // A new seed for a new shape
                let seed = Some(rand_pcg::Pcg64::from_entropy().gen());
                self.rough_options.seed = seed;

                match self.builder_type {
                    ShapeBuilderType::Line => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(LineBuilder::start(element, self.ratio.into())),
                        }
                    }
                    ShapeBuilderType::Rectangle => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(RectangleBuilder::start(element, self.ratio.into())),
                        }
                    }
                    ShapeBuilderType::Ellipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(EllipseBuilder::start(element, self.ratio.into())),
                        }
                    }
                    ShapeBuilderType::FociEllipse => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(FociEllipseBuilder::start(
                                element,
                                self.ratio.into(),
                            )),
                        }
                    }
                    ShapeBuilderType::QuadBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(QuadBezBuilder::start(element, self.ratio.into())),
                        }
                    }
                    ShapeBuilderType::CubBez => {
                        self.state = ShaperState::BuildShape {
                            builder: Box::new(CubBezBuilder::start(element, self.ratio.into())),
                        }
                    }
                }

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (ShaperState::Idle, _) => PenProgress::Idle,
            (ShaperState::BuildShape { .. }, PenEvent::Cancel) => {
                self.state = ShaperState::Idle;

                surface_flags.redraw = true;
                PenProgress::Finished
            }
            (ShaperState::BuildShape { builder }, event) => match builder.handle_event(event) {
                BuilderProgress::InProgress => {
                    surface_flags.redraw = true;

                    PenProgress::InProgress
                }
                BuilderProgress::EmitContinue(shapes) => {
                    let drawstyle = self.gen_style_for_current_options();

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) = store.regenerate_rendering_for_stroke(
                            key,
                            camera.viewport(),
                            camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new line, Err {}", e);
                        }
                    }

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;

                    PenProgress::InProgress
                }
                BuilderProgress::Finished(shapes) => {
                    let drawstyle = self.gen_style_for_current_options();

                    if !shapes.is_empty() {
                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) = store.regenerate_rendering_for_stroke(
                            key,
                            camera.viewport(),
                            camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new shape, Err {}", e);
                        }
                    }

                    self.state = ShaperState::Idle;

                    surface_flags.redraw = true;

                    PenProgress::Finished
                }
            },
        };

        (pen_progress, surface_flags)
    }
}

impl DrawOnSheetBehaviour for Shaper {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        let style = self.gen_style_for_current_options();

        match &self.state {
            ShaperState::Idle => None,
            ShaperState::BuildShape { builder } => {
                Some(builder.bounds(&style, camera.total_zoom()))
            }
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        _sheet_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
        let style = self.gen_style_for_current_options();

        match &self.state {
            ShaperState::Idle => {}
            ShaperState::BuildShape { builder } => {
                builder.draw_styled(cx, &style, camera.total_zoom())
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

impl Shaper {
    pub const INPUT_OVERSHOOT: f64 = 30.0;

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
