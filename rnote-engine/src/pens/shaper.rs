use super::penbehaviour::{PenBehaviour, PenProgress};
use super::AudioPlayer;
use crate::sheet::Sheet;
use crate::strokes::ShapeStroke;
use crate::strokes::Stroke;
use crate::{Camera, DrawOnSheetBehaviour, StrokeStore, SurfaceFlags};

use p2d::bounding_volume::AABB;
use rand::{Rng, SeedableRng};
use rnote_compose::builders::{CubBezBuilder, QuadBezBuilder, ShapeBuilderType};
use rnote_compose::builders::{
    EllipseBuilder, FociEllipseBuilder, LineBuilder, RectangleBuilder, ShapeBuilderBehaviour,
};
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::style::rough::RoughOptions;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::style::Composer;
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

#[derive(Debug, Clone)]
enum ShaperState {
    Idle,
    BuildLine {
        line_builder: LineBuilder,
    },
    BuildRectangle {
        rect_builder: RectangleBuilder,
    },
    BuildEllipse {
        ellipse_builder: EllipseBuilder,
    },
    BuildFociEllipse {
        foci_ellipse_builder: FociEllipseBuilder,
    },
    BuildQuadBez {
        quadbez_builder: QuadBezBuilder,
    },
    BuildCubBez {
        cubbez_builder: CubBezBuilder,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    state: ShaperState,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            builder_type: ShapeBuilderType::default(),
            style: ShaperStyle::default(),
            smooth_options: SmoothOptions::default(),
            rough_options: RoughOptions::default(),
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
                // A new seed for a new shape
                let seed = Some(rand_pcg::Pcg64::from_entropy().gen());
                self.rough_options.seed = seed;

                match self.builder_type {
                    ShapeBuilderType::Line => {
                        self.state = ShaperState::BuildLine {
                            line_builder: LineBuilder::start(element),
                        }
                    }
                    ShapeBuilderType::Rectangle => {
                        self.state = ShaperState::BuildRectangle {
                            rect_builder: RectangleBuilder::start(element),
                        }
                    }
                    ShapeBuilderType::Ellipse => {
                        self.state = ShaperState::BuildEllipse {
                            ellipse_builder: EllipseBuilder::start(element),
                        }
                    }
                    ShapeBuilderType::FociEllipse => {
                        self.state = ShaperState::BuildFociEllipse {
                            foci_ellipse_builder: FociEllipseBuilder::start(element),
                        }
                    }
                    ShapeBuilderType::QuadBez => {
                        self.state = ShaperState::BuildQuadBez {
                            quadbez_builder: QuadBezBuilder::start(element),
                        }
                    }
                    ShapeBuilderType::CubBez => {
                        self.state = ShaperState::BuildCubBez {
                            cubbez_builder: CubBezBuilder::start(element),
                        }
                    }
                }

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (ShaperState::Idle, _) => PenProgress::Idle,
            (ShaperState::BuildLine { line_builder }, event @ PenEvent::Down { .. }) => {
                // we know the builder only emits a shape on up events, so we don't handle the return
                line_builder.handle_event(event);

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (ShaperState::BuildLine { line_builder }, event @ PenEvent::Up { .. }) => {
                if let Some(shapes) = line_builder.handle_event(event) {
                    let drawstyle = self.gen_style_for_current_options();

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) =
                            store.regenerate_rendering_for_stroke(key, camera.image_scale())
                        {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new line, Err {}", e);
                        }
                    }

                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;
                }

                surface_flags.redraw = true;

                self.state = ShaperState::Idle;
                PenProgress::Finished
            }
            (ShaperState::BuildLine { .. }, PenEvent::Proximity { .. }) => PenProgress::InProgress,
            (ShaperState::BuildRectangle { rect_builder }, event @ PenEvent::Down { .. }) => {
                // we know the builder only emits a shape on up events, so we don't handle the return
                rect_builder.handle_event(event);

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (ShaperState::BuildRectangle { rect_builder }, PenEvent::Up { .. }) => {
                if let Some(shapes) = rect_builder.handle_event(event) {
                    let drawstyle = self.gen_style_for_current_options();

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) =
                            store.regenerate_rendering_for_stroke(key, camera.image_scale())
                        {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new rectangle, Err {}", e);
                        }
                    }

                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;
                }

                self.state = ShaperState::Idle;

                surface_flags.redraw = true;

                PenProgress::Finished
            }
            (ShaperState::BuildRectangle { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (ShaperState::BuildEllipse { ellipse_builder }, event @ PenEvent::Down { .. }) => {
                // we know the builder only emits a shape on up events, so we don't handle the return
                ellipse_builder.handle_event(event);

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (ShaperState::BuildEllipse { ellipse_builder }, PenEvent::Up { .. }) => {
                if let Some(shapes) = ellipse_builder.handle_event(event) {
                    let drawstyle = self.gen_style_for_current_options();

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) =
                            store.regenerate_rendering_for_stroke(key, camera.image_scale())
                        {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new ellipse, Err {}", e);
                        }
                    }

                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;
                }

                self.state = ShaperState::Idle;

                surface_flags.redraw = true;

                PenProgress::Finished
            }
            (ShaperState::BuildEllipse { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (
                ShaperState::BuildFociEllipse {
                    foci_ellipse_builder,
                },
                PenEvent::Down { .. },
            ) => {
                // we know the builder only emits a shape on up events, so we don't handle the return
                foci_ellipse_builder.handle_event(event);

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (
                ShaperState::BuildFociEllipse {
                    foci_ellipse_builder,
                },
                PenEvent::Up { .. },
            ) => {
                let mut pen_progress = PenProgress::InProgress;

                if let Some(shapes) = foci_ellipse_builder.handle_event(event) {
                    let drawstyle = self.gen_style_for_current_options();

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) =
                            store.regenerate_rendering_for_stroke(key, camera.image_scale())
                        {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new foci ellipse, Err {}", e);
                        }

                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }

                    self.state = ShaperState::Idle;

                    surface_flags.redraw = true;

                    pen_progress = PenProgress::Finished
                }

                pen_progress
            }
            (ShaperState::BuildFociEllipse { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (ShaperState::BuildQuadBez { quadbez_builder }, PenEvent::Down { .. }) => {
                // we know the builder only emits a shape on up events, so we don't handle the return
                quadbez_builder.handle_event(event);

                surface_flags.redraw = true;

                PenProgress::InProgress
            }
            (ShaperState::BuildQuadBez { quadbez_builder }, PenEvent::Up { .. }) => {
                let mut pen_progress = PenProgress::InProgress;

                if let Some(shapes) = quadbez_builder.handle_event(event) {
                    let drawstyle = self.gen_style_for_current_options();

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) =
                            store.regenerate_rendering_for_stroke(key, camera.image_scale())
                        {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new quadbez, Err {}", e);
                        }

                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }

                    self.state = ShaperState::Idle;

                    surface_flags.redraw = true;

                    pen_progress = PenProgress::Finished
                }

                pen_progress
            }
            (ShaperState::BuildQuadBez { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (ShaperState::BuildCubBez { cubbez_builder }, PenEvent::Down { .. }) => {
                // we know the builder only emits a shape on up events, so we don't handle the return
                cubbez_builder.handle_event(event);

                surface_flags.redraw = true;
                PenProgress::InProgress
            }
            (ShaperState::BuildCubBez { cubbez_builder }, PenEvent::Up { .. }) => {
                let mut pen_progress = PenProgress::InProgress;

                if let Some(shapes) = cubbez_builder.handle_event(event) {
                    let drawstyle = self.gen_style_for_current_options();

                    for shape in shapes {
                        let key = store.insert_stroke(Stroke::ShapeStroke(ShapeStroke::new(
                            shape,
                            drawstyle.clone(),
                        )));
                        if let Err(e) =
                            store.regenerate_rendering_for_stroke(key, camera.image_scale())
                        {
                            log::error!("regenerate_rendering_for_stroke() failed after inserting new quadbez, Err {}", e);
                        }

                        surface_flags.resize = true;
                        surface_flags.sheet_changed = true;
                    }

                    self.state = ShaperState::Idle;
                    surface_flags.redraw = true;
                    pen_progress = PenProgress::Finished
                }

                pen_progress
            }
            (ShaperState::BuildCubBez { .. }, PenEvent::Proximity { .. }) => {
                PenProgress::InProgress
            }
            (_, PenEvent::Cancel) => {
                // Same behaviour for any state for cancel events
                self.state = ShaperState::Idle;

                surface_flags.redraw = true;
                PenProgress::Finished
            }
        };

        (pen_progress, surface_flags)
    }
}

impl DrawOnSheetBehaviour for Shaper {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        match (&self.state, &self.style) {
            (ShaperState::Idle, ShaperStyle::Smooth) => None,
            (ShaperState::Idle, ShaperStyle::Rough) => None,
            (ShaperState::BuildLine { line_builder }, ShaperStyle::Smooth) => {
                Some(line_builder.composed_bounds(&self.smooth_options))
            }
            (ShaperState::BuildLine { line_builder }, ShaperStyle::Rough) => {
                Some(line_builder.composed_bounds(&self.rough_options))
            }
            (ShaperState::BuildRectangle { rect_builder }, ShaperStyle::Smooth) => {
                Some(rect_builder.composed_bounds(&self.smooth_options))
            }
            (ShaperState::BuildRectangle { rect_builder }, ShaperStyle::Rough) => {
                Some(rect_builder.composed_bounds(&self.rough_options))
            }
            (ShaperState::BuildEllipse { ellipse_builder }, ShaperStyle::Smooth) => {
                Some(ellipse_builder.composed_bounds(&self.smooth_options))
            }
            (ShaperState::BuildEllipse { ellipse_builder }, ShaperStyle::Rough) => {
                Some(ellipse_builder.composed_bounds(&self.rough_options))
            }
            (
                ShaperState::BuildFociEllipse {
                    foci_ellipse_builder,
                },
                ShaperStyle::Smooth,
            ) => Some(foci_ellipse_builder.composed_bounds(&self.smooth_options)),
            (
                ShaperState::BuildFociEllipse {
                    foci_ellipse_builder,
                },
                ShaperStyle::Rough,
            ) => Some(foci_ellipse_builder.composed_bounds(&self.rough_options)),
            (ShaperState::BuildQuadBez { quadbez_builder }, ShaperStyle::Smooth) => {
                Some(quadbez_builder.composed_bounds(&self.smooth_options))
            }
            (ShaperState::BuildQuadBez { quadbez_builder }, ShaperStyle::Rough) => {
                Some(quadbez_builder.composed_bounds(&self.rough_options))
            }
            (ShaperState::BuildCubBez { cubbez_builder }, ShaperStyle::Smooth) => {
                Some(cubbez_builder.composed_bounds(&self.smooth_options))
            }
            (ShaperState::BuildCubBez { cubbez_builder }, ShaperStyle::Rough) => {
                Some(cubbez_builder.composed_bounds(&self.rough_options))
            }
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        _camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        match (&self.state, &self.style) {
            (ShaperState::Idle, _) => {}
            (ShaperState::BuildLine { line_builder }, ShaperStyle::Smooth) => {
                line_builder.draw_composed(cx, &self.smooth_options);
            }
            (ShaperState::BuildLine { line_builder }, ShaperStyle::Rough) => {
                line_builder.draw_composed(cx, &self.rough_options);
            }
            (ShaperState::BuildRectangle { rect_builder }, ShaperStyle::Smooth) => {
                rect_builder.draw_composed(cx, &self.smooth_options);
            }
            (ShaperState::BuildRectangle { rect_builder }, ShaperStyle::Rough) => {
                rect_builder.draw_composed(cx, &self.rough_options);
            }
            (ShaperState::BuildEllipse { ellipse_builder }, ShaperStyle::Smooth) => {
                ellipse_builder.draw_composed(cx, &self.smooth_options);
            }
            (ShaperState::BuildEllipse { ellipse_builder }, ShaperStyle::Rough) => {
                ellipse_builder.draw_composed(cx, &self.rough_options);
            }
            (
                ShaperState::BuildFociEllipse {
                    foci_ellipse_builder,
                },
                ShaperStyle::Smooth,
            ) => {
                foci_ellipse_builder.draw_composed(cx, &self.smooth_options);
            }
            (
                ShaperState::BuildFociEllipse {
                    foci_ellipse_builder,
                },
                ShaperStyle::Rough,
            ) => {
                foci_ellipse_builder.draw_composed(cx, &self.rough_options);
            }
            (ShaperState::BuildQuadBez { quadbez_builder }, ShaperStyle::Smooth) => {
                quadbez_builder.draw_composed(cx, &self.smooth_options);
            }
            (ShaperState::BuildQuadBez { quadbez_builder }, ShaperStyle::Rough) => {
                quadbez_builder.draw_composed(cx, &self.rough_options);
            }
            (ShaperState::BuildCubBez { cubbez_builder }, ShaperStyle::Smooth) => {
                cubbez_builder.draw_composed(cx, &self.smooth_options);
            }
            (ShaperState::BuildCubBez { cubbez_builder }, ShaperStyle::Rough) => {
                cubbez_builder.draw_composed(cx, &self.rough_options);
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
