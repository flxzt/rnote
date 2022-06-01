use crate::engine::EngineTaskSender;
use crate::{Camera, Document, DrawOnDocBehaviour, StrokeStore, SurfaceFlags};
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::penpath::Element;

use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

use super::penbehaviour::{PenBehaviour, PenProgress};
use super::AudioPlayer;

#[derive(Debug, Clone, Copy)]
pub enum EraserState {
    Up,
    Down(Element),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "eraser_style")]
pub enum EraserStyle {
    #[serde(rename = "trash_colliding_strokes")]
    TrashCollidingStrokes,
    #[serde(rename = "split_colliding_strokes")]
    SplitCollidingStrokes,
}

impl Default for EraserStyle {
    fn default() -> Self {
        Self::TrashCollidingStrokes
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "eraser")]
pub struct Eraser {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "style")]
    pub style: EraserStyle,
    #[serde(skip)]
    pub(crate) state: EraserState,
}

impl Default for Eraser {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            style: EraserStyle::default(),
            state: EraserState::Up,
        }
    }
}

impl PenBehaviour for Eraser {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _tasks_tx: EngineTaskSender,
        _doc: &mut Document,
        store: &mut StrokeStore,
        camera: &mut Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) -> (PenProgress, SurfaceFlags) {
        let mut surface_flags = SurfaceFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (
                EraserState::Up,
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                surface_flags.merge_with_other(store.record());

                match &self.style {
                    EraserStyle::TrashCollidingStrokes => {
                        surface_flags.merge_with_other(store.trash_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = store.split_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            camera.viewport(),
                        );

                        if let Err(e) = store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            camera.viewport(),
                            camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_strokes() failed while splitting colliding strokes, Err {}", e);
                        }
                    }
                }

                self.state = EraserState::Down(element);

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(true);
                surface_flags.store_changed = true;

                PenProgress::InProgress
            }
            (EraserState::Up, _) => PenProgress::Idle,
            (EraserState::Down(current_element), PenEvent::Down { element, .. }) => {
                match &self.style {
                    EraserStyle::TrashCollidingStrokes => {
                        surface_flags.merge_with_other(store.trash_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = store.split_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            camera.viewport(),
                        );

                        if let Err(e) = store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            camera.viewport(),
                            camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_strokes() failed while splitting colliding strokes, Err {}", e);
                        }
                    }
                }

                *current_element = element;

                surface_flags.redraw = true;
                surface_flags.store_changed = true;

                PenProgress::InProgress
            }
            (EraserState::Down { .. }, PenEvent::Up { element, .. }) => {
                match &self.style {
                    EraserStyle::TrashCollidingStrokes => {
                        surface_flags.merge_with_other(store.trash_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = store.split_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            camera.viewport(),
                        );

                        if let Err(e) = store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            camera.viewport(),
                            camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_strokes() failed while splitting colliding strokes, Err {}", e);
                        }
                    }
                }

                self.state = EraserState::Up;

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(false);
                surface_flags.store_changed = true;

                PenProgress::Finished
            }
            (EraserState::Down { .. }, PenEvent::Proximity { .. }) => PenProgress::InProgress,
            (EraserState::Down { .. }, PenEvent::Cancel) => {
                self.state = EraserState::Up;

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(false);

                PenProgress::Finished
            }
        };

        (pen_progress, surface_flags)
    }
}

impl Eraser {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 12.0;

    pub fn new(width: f64) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    fn eraser_bounds(eraser_width: f64, element: Element) -> AABB {
        AABB::from_half_extents(
            na::Point2::from(element.pos),
            na::Vector2::repeat(eraser_width * 0.5),
        )
    }
}

impl DrawOnDocBehaviour for Eraser {
    fn bounds_on_doc(&self, _doc_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        match &self.state {
            EraserState::Up => None,
            EraserState::Down(current_element) => {
                Some(Self::eraser_bounds(self.width, *current_element))
            }
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        doc_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[2].with_a8(0xf0);
        const FILL_COLOR: piet::Color = color::GNOME_REDS[0].with_a8(0x80);
        let outline_width = 2.0 / camera.total_zoom();

        if let Some(bounds) = self.bounds_on_doc(doc_bounds, camera) {
            let fill_rect = bounds.to_kurbo_rect();
            let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

            cx.fill(fill_rect, &FILL_COLOR);
            cx.stroke(outline_rect, &OUTLINE_COLOR, outline_width);
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}
