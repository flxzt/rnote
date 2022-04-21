use crate::{Camera, DrawOnSheetBehaviour, Sheet, StrokeStore, SurfaceFlags};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "eraser")]
pub struct Eraser {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(skip)]
    pub(crate) state: EraserState,
}

impl Default for Eraser {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            state: EraserState::Up,
        }
    }
}

impl PenBehaviour for Eraser {
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
            (
                EraserState::Up,
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                store.trash_colliding_strokes(
                    Self::eraser_bounds(self.width, element),
                    camera.viewport(),
                );

                self.state = EraserState::Down(element);

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(true);
                surface_flags.sheet_changed = true;

                PenProgress::InProgress
            }
            (EraserState::Up, _) => PenProgress::Idle,
            (EraserState::Down(current_element), PenEvent::Down { element, .. }) => {
                store.trash_colliding_strokes(
                    Self::eraser_bounds(self.width, element),
                    camera.viewport(),
                );

                *current_element = element;

                surface_flags.redraw = true;
                surface_flags.sheet_changed = true;

                PenProgress::InProgress
            }
            (EraserState::Down { .. }, PenEvent::Up { element, .. }) => {
                store.trash_colliding_strokes(
                    Self::eraser_bounds(self.width, element),
                    camera.viewport(),
                );

                self.state = EraserState::Up;

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(false);
                surface_flags.sheet_changed = true;

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
    pub const WIDTH_DEFAULT: f64 = 20.0;

    pub fn new(width: f64) -> Self {
        Self {
            width,
            state: EraserState::Up,
        }
    }

    fn eraser_bounds(eraser_width: f64, element: Element) -> AABB {
        AABB::from_half_extents(
            na::Point2::from(element.pos),
            na::Vector2::repeat(eraser_width / 2.0),
        )
    }
}

impl DrawOnSheetBehaviour for Eraser {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        match &self.state {
            EraserState::Up => None,
            EraserState::Down(current_element) => {
                Some(Self::eraser_bounds(self.width, *current_element))
            }
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        sheet_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[2].with_a8(0xf0);
        const FILL_COLOR: piet::Color = color::GNOME_REDS[0].with_a8(0x80);
        let outline_width = 2.0 / camera.total_zoom();

        if let Some(bounds) = self.bounds_on_sheet(sheet_bounds, camera) {
            let fill_rect = bounds.to_kurbo_rect();
            let outline_rect = bounds.tightened(outline_width * 0.5).to_kurbo_rect();

            cx.fill(fill_rect, &FILL_COLOR);
            cx.stroke(outline_rect, &OUTLINE_COLOR, outline_width);
        }

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}
