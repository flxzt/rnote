use std::time::Instant;

use super::penbehaviour::{PenBehaviour, PenProgress};
use crate::engine::{EngineView, EngineViewMut};
use crate::store::chrono_comp::StrokeLayer;
use crate::store::StrokeKey;
use crate::strokes::BrushStroke;
use crate::strokes::Stroke;
use crate::AudioPlayer;
use crate::{DrawOnDocBehaviour, WidgetFlags};
use rnote_compose::builders::{
    Constraints, PenPathBuilderBehaviour, PenPathBuilderCreator, PenPathBuilderProgress,
    PenPathModeledBuilder,
};
use rnote_compose::builders::{PenPathCurvedBuilder, PenPathSimpleBuilder};
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::style::textured::TexturedOptions;
use rnote_compose::style::PressureCurve;
use rnote_compose::Style;

use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use rand::{Rng, SeedableRng};
use rnote_compose::builders::PenPathBuilderType;
use rnote_compose::style::smooth::SmoothOptions;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "brush_style")]
pub enum BrushStyle {
    #[serde(rename = "marker")]
    Marker = 0,
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "textured")]
    Textured,
}

impl Default for BrushStyle {
    fn default() -> Self {
        Self::Solid
    }
}

impl TryFrom<u32> for BrushStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("BrushStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "marker_options")]
pub struct MarkerOptions(SmoothOptions);

impl Default for MarkerOptions {
    fn default() -> Self {
        let mut options = SmoothOptions::default();
        options.pressure_curve = PressureCurve::Const;
        options.stroke_width = 12.0;

        Self(options)
    }
}

impl std::ops::Deref for MarkerOptions {
    type Target = SmoothOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MarkerOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "solid_options")]
pub struct SolidOptions(SmoothOptions);

impl Default for SolidOptions {
    fn default() -> Self {
        Self(SmoothOptions::default())
    }
}

impl std::ops::Deref for SolidOptions {
    type Target = SmoothOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SolidOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
enum BrushState {
    Idle,
    Drawing {
        path_builder: Box<dyn PenPathBuilderBehaviour>,
        current_stroke_key: StrokeKey,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "brush")]
pub struct Brush {
    pub builder_type: PenPathBuilderType,
    #[serde(rename = "style")]
    pub style: BrushStyle,
    #[serde(rename = "marker_options")]
    pub marker_options: MarkerOptions,
    #[serde(rename = "solid_options")]
    pub solid_options: SolidOptions,
    #[serde(rename = "textured_options")]
    pub textured_options: TexturedOptions,

    #[serde(skip)]
    state: BrushState,
}

impl Clone for Brush {
    fn clone(&self) -> Self {
        Self {
            style: self.style,
            builder_type: self.builder_type,
            marker_options: self.marker_options.clone(),
            solid_options: self.solid_options.clone(),
            textured_options: self.textured_options.clone(),
            state: BrushState::Idle,
        }
    }
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            style: BrushStyle::default(),
            builder_type: PenPathBuilderType::default(),
            marker_options: MarkerOptions::default(),
            solid_options: SolidOptions::default(),
            textured_options: TexturedOptions::default(),
            state: BrushState::Idle,
        }
    }
}

impl PenBehaviour for Brush {
    fn handle_event(
        &mut self,
        event: PenEvent,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (
                BrushState::Idle,
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                if !element
                    .filter_by_bounds(engine_view.doc.bounds().loosened(Self::INPUT_OVERSHOOT))
                {
                    widget_flags.merge_with_other(engine_view.store.record());
                    self.start_audio(engine_view.audioplayer);
                    self.new_style_seeds();

                    let brushstroke = Stroke::BrushStroke(BrushStroke::new(
                        element,
                        self.style_for_current_options(),
                    ));
                    let current_stroke_key = engine_view
                        .store
                        .insert_stroke(brushstroke, Some(self.layer_for_current_options()));

                    let path_builder: Box<dyn PenPathBuilderBehaviour> = match self.builder_type {
                        PenPathBuilderType::Simple => {
                            Box::new(PenPathSimpleBuilder::start(element, Instant::now()))
                        }
                        PenPathBuilderType::Curved => {
                            Box::new(PenPathCurvedBuilder::start(element, Instant::now()))
                        }
                        PenPathBuilderType::Modeled => {
                            Box::new(PenPathModeledBuilder::start(element, Instant::now()))
                        }
                    };

                    if let Err(e) = engine_view.store.regenerate_rendering_for_stroke(
                        current_stroke_key,
                        engine_view.camera.viewport(),
                        engine_view.camera.image_scale(),
                    ) {
                        log::error!("regenerate_rendering_for_stroke() failed after inserting brush stroke, Err: {e:?}");
                    }

                    self.state = BrushState::Drawing {
                        path_builder,
                        current_stroke_key,
                    };

                    widget_flags.redraw = true;

                    PenProgress::InProgress
                } else {
                    PenProgress::Idle
                }
            }
            (BrushState::Idle, _) => PenProgress::Idle,
            (
                BrushState::Drawing {
                    current_stroke_key, ..
                },
                PenEvent::Cancel,
            ) => {
                // Finish up the last stroke
                engine_view
                    .store
                    .update_geometry_for_stroke(*current_stroke_key);
                engine_view.store.regenerate_rendering_for_stroke_threaded(
                    engine_view.tasks_tx.clone(),
                    *current_stroke_key,
                    engine_view.camera.viewport(),
                    engine_view.camera.image_scale(),
                );

                self.state = BrushState::Idle;

                engine_view
                    .doc
                    .resize_autoexpand(engine_view.store, engine_view.camera);

                widget_flags.redraw = true;
                widget_flags.resize = true;
                widget_flags.indicate_changed_store = true;

                self.stop_audio(engine_view.audioplayer);

                PenProgress::Finished
            }
            (
                BrushState::Drawing {
                    path_builder,
                    current_stroke_key,
                },
                pen_event,
            ) => {
                match path_builder.handle_event(pen_event, Instant::now(), Constraints::default()) {
                    PenPathBuilderProgress::InProgress => {
                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    PenPathBuilderProgress::EmitContinue(segments) => {
                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.indicate_changed_store = true;
                            }

                            if let Err(e) = engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            ) {
                                log::error!("append_rendering_last_segments() for penevent down in brush failed with Err: {e:?}");
                            }
                        }

                        widget_flags.redraw = true;

                        PenProgress::InProgress
                    }
                    PenPathBuilderProgress::Finished(segments) => {
                        let n_segments = segments.len();

                        if n_segments != 0 {
                            if let Some(Stroke::BrushStroke(brushstroke)) =
                                engine_view.store.get_stroke_mut(*current_stroke_key)
                            {
                                brushstroke.extend_w_segments(segments);
                                widget_flags.indicate_changed_store = true;
                            }

                            // First we draw the last segments immediately,
                            if let Err(e) = engine_view.store.append_rendering_last_segments(
                                engine_view.tasks_tx.clone(),
                                *current_stroke_key,
                                n_segments,
                                engine_view.camera.viewport(),
                                engine_view.camera.image_scale(),
                            ) {
                                log::error!("append_rendering_last_segments() for penevent down in brush failed with Err: {e:?}");
                            }
                        }

                        // but then regenerate the entire stroke rendering because it gets rid of some artifacts
                        // Finish up the last stroke
                        engine_view
                            .store
                            .update_geometry_for_stroke(*current_stroke_key);
                        engine_view.store.regenerate_rendering_for_stroke_threaded(
                            engine_view.tasks_tx.clone(),
                            *current_stroke_key,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        );

                        self.stop_audio(engine_view.audioplayer);

                        self.state = BrushState::Idle;

                        engine_view
                            .doc
                            .resize_autoexpand(engine_view.store, engine_view.camera);

                        widget_flags.redraw = true;
                        widget_flags.resize = true;
                        widget_flags.indicate_changed_store = true;

                        PenProgress::Finished
                    }
                }
            }
        };

        (pen_progress, widget_flags)
    }
}

impl DrawOnDocBehaviour for Brush {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<AABB> {
        let style = self.style_for_current_options();

        match &self.state {
            BrushState::Idle => None,
            BrushState::Drawing { path_builder, .. } => {
                path_builder.bounds(&style, engine_view.camera.zoom())
            }
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        match &self.state {
            BrushState::Idle => {}
            BrushState::Drawing { path_builder, .. } => {
                match self.style {
                    BrushStyle::Marker => {
                        // Don't draw the marker, as the pen would render on top of other strokes, while the stroke itself would render underneath them.
                    }
                    BrushStyle::Solid | BrushStyle::Textured => {
                        let style = self.style_for_current_options();

                        /*
                                               // Change color for debugging
                                               match &mut style {
                                                   Style::Smooth(options) => {
                                                       options.stroke_color = Some(rnote_compose::Color::RED)
                                                   }
                                                   Style::Rough(_) | Style::Textured(_) => {}
                                               }
                        */

                        path_builder.draw_styled(cx, &style, engine_view.camera.total_zoom());
                    }
                }
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}

impl Brush {
    const INPUT_OVERSHOOT: f64 = 30.0;

    pub const STROKE_WIDTH_MIN: f64 = 0.1;
    pub const STROKE_WIDTH_MAX: f64 = 500.0;

    fn start_audio(&self, audioplayer: &mut Option<AudioPlayer>) {
        if let Some(audioplayer) = audioplayer {
            match self.style {
                BrushStyle::Marker => {
                    audioplayer.play_random_marker_sound();
                }
                BrushStyle::Solid | BrushStyle::Textured => {
                    audioplayer.start_random_brush_sound();
                }
            }
        }
    }

    fn stop_audio(&self, audioplayer: &mut Option<AudioPlayer>) {
        if let Some(audioplayer) = audioplayer {
            audioplayer.stop_random_brush_sond();
        }
    }

    pub fn layer_for_current_options(&self) -> StrokeLayer {
        match &self.style {
            BrushStyle::Marker => StrokeLayer::Highlighter,
            BrushStyle::Solid | BrushStyle::Textured => StrokeLayer::UserLayer(0),
        }
    }

    fn new_style_seeds(&mut self) {
        // A new seed for new shapes
        let seed = Some(rand_pcg::Pcg64::from_entropy().gen());
        self.textured_options.seed = seed;
    }

    pub fn style_for_current_options(&self) -> Style {
        match &self.style {
            BrushStyle::Marker => {
                let options = self.marker_options.clone();

                Style::Smooth(options.0)
            }
            BrushStyle::Solid => {
                let options = self.solid_options.clone();

                Style::Smooth(options.0)
            }
            BrushStyle::Textured => {
                let options = self.textured_options.clone();

                Style::Textured(options)
            }
        }
    }
}
