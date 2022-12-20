use std::time::Instant;

use super::penbehaviour::{PenBehaviour, PenProgress};
use crate::engine::{EngineView, EngineViewMut};
use crate::{DrawOnDocBehaviour, WidgetFlags};
use once_cell::sync::Lazy;
use piet::RenderContext;
use rnote_compose::color;
use rnote_compose::helpers::AabbHelpers;
use rnote_compose::penevents::PenEvent;
use rnote_compose::penpath::Element;

use p2d::bounding_volume::{Aabb, BoundingVolume};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum EraserState {
    Up,
    Proximity(Element),
    Down(Element),
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
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

impl TryFrom<u32> for EraserStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("EraserStyle try_from::<u32>() for value {} failed", value)
        })
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
        _now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> (PenProgress, WidgetFlags) {
        let mut widget_flags = WidgetFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (
                EraserState::Up | EraserState::Proximity { .. },
                PenEvent::Down {
                    element,
                    shortcut_keys: _,
                },
            ) => {
                widget_flags.merge_with_other(engine_view.store.record());

                match &self.style {
                    EraserStyle::TrashCollidingStrokes => {
                        widget_flags.merge_with_other(engine_view.store.trash_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            engine_view.camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = engine_view.store.split_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            engine_view.camera.viewport(),
                        );

                        if let Err(e) = engine_view.store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_strokes() failed while splitting colliding strokes, Err: {e:?}");
                        }
                    }
                }

                self.state = EraserState::Down(element);

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
            (EraserState::Up | EraserState::Down { .. }, PenEvent::Proximity { element, .. }) => {
                self.state = EraserState::Proximity(element);
                widget_flags.redraw = true;

                PenProgress::Idle
            }
            (
                EraserState::Up,
                PenEvent::KeyPressed { .. } | PenEvent::Up { .. } | PenEvent::Cancel,
            ) => PenProgress::Idle,
            (EraserState::Down(current_element), PenEvent::Down { element, .. }) => {
                match &self.style {
                    EraserStyle::TrashCollidingStrokes => {
                        widget_flags.merge_with_other(engine_view.store.trash_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            engine_view.camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = engine_view.store.split_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            engine_view.camera.viewport(),
                        );

                        if let Err(e) = engine_view.store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_strokes() failed while splitting colliding strokes, Err: {e:?}");
                        }
                    }
                }

                *current_element = element;

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::InProgress
            }
            (EraserState::Down { .. }, PenEvent::Up { element, .. }) => {
                match &self.style {
                    EraserStyle::TrashCollidingStrokes => {
                        widget_flags.merge_with_other(engine_view.store.trash_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            engine_view.camera.viewport(),
                        ));
                    }
                    EraserStyle::SplitCollidingStrokes => {
                        let new_strokes = engine_view.store.split_colliding_strokes(
                            Self::eraser_bounds(self.width, element),
                            engine_view.camera.viewport(),
                        );

                        if let Err(e) = engine_view.store.regenerate_rendering_for_strokes(
                            &new_strokes,
                            engine_view.camera.viewport(),
                            engine_view.camera.image_scale(),
                        ) {
                            log::error!("regenerate_rendering_for_strokes() failed while splitting colliding strokes, Err: {e:?}");
                        }
                    }
                }

                self.state = EraserState::Up;

                widget_flags.redraw = true;
                widget_flags.indicate_changed_store = true;

                PenProgress::Finished
            }
            (EraserState::Down(_), PenEvent::KeyPressed { .. }) => PenProgress::InProgress,
            (EraserState::Proximity(_), PenEvent::Up { .. }) => {
                self.state = EraserState::Up;
                widget_flags.redraw = true;

                PenProgress::Idle
            }
            (EraserState::Proximity(current_element), PenEvent::Proximity { element, .. }) => {
                *current_element = element;
                widget_flags.redraw = true;

                PenProgress::Idle
            }
            (EraserState::Proximity { .. } | EraserState::Down { .. }, PenEvent::Cancel) => {
                self.state = EraserState::Up;

                widget_flags.redraw = true;

                PenProgress::Finished
            }
            (EraserState::Proximity(_), PenEvent::KeyPressed { .. }) => PenProgress::Idle,
            (EraserState::Up, PenEvent::Text { .. }) => PenProgress::Idle,
            (EraserState::Proximity(_), PenEvent::Text { .. }) => PenProgress::Idle,
            (EraserState::Down(_), PenEvent::Text { .. }) => PenProgress::InProgress,
        };

        (pen_progress, widget_flags)
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

    fn eraser_bounds(eraser_width: f64, element: Element) -> Aabb {
        Aabb::from_half_extents(
            na::Point2::from(element.pos),
            na::Vector2::repeat(eraser_width * 0.5),
        )
    }
}

impl DrawOnDocBehaviour for Eraser {
    fn bounds_on_doc(&self, _engine_view: &EngineView) -> Option<Aabb> {
        match &self.state {
            EraserState::Up => None,
            EraserState::Proximity(current_element) | EraserState::Down(current_element) => {
                Some(Self::eraser_bounds(self.width, *current_element))
            }
        }
    }

    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        static OUTLINE_COLOR: Lazy<piet::Color> =
            Lazy::new(|| color::GNOME_REDS[2].with_alpha(0.941));
        static FILL_COLOR: Lazy<piet::Color> = Lazy::new(|| color::GNOME_REDS[0].with_alpha(0.627));
        static PROXIMITY_FILL_COLOR: Lazy<piet::Color> =
            Lazy::new(|| color::GNOME_REDS[0].with_alpha(0.5));
        let outline_width = 2.0 / engine_view.camera.total_zoom();

        match &self.state {
            EraserState::Up => {}
            EraserState::Proximity(current_element) => {
                let bounds = Self::eraser_bounds(self.width, *current_element);

                let fill_rect = bounds.to_kurbo_rect();
                let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

                cx.fill(fill_rect, &*PROXIMITY_FILL_COLOR);
                cx.stroke(outline_rect, &*OUTLINE_COLOR, outline_width);
            }
            EraserState::Down(current_element) => {
                let bounds = Self::eraser_bounds(self.width, *current_element);

                let fill_rect = bounds.to_kurbo_rect();
                let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

                cx.fill(fill_rect, &*FILL_COLOR);
                cx.stroke(outline_rect, &*OUTLINE_COLOR, outline_width);
            }
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
