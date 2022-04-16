use crate::{Camera, DrawOnSheetBehaviour, Sheet, StrokeStore, SurfaceFlags};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penpath::Element;
use rnote_compose::{color, PenEvent};

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
        let surface_flags = SurfaceFlags::default();

        let pen_progress = match (&mut self.state, event) {
            (
                EraserState::Up,
                PenEvent::Down {
                    element,
                    shortcut_key: _,
                },
            ) => {
                let eraser_bounds = AABB::from_half_extents(
                    na::Point2::from(element.pos),
                    na::Vector2::repeat(self.width),
                );
                store.trash_colliding_strokes(eraser_bounds, camera.viewport());

                self.state = EraserState::Down(element);
                PenProgress::InProgress
            }
            (EraserState::Down(current_element), PenEvent::Down { element, .. }) => {
                let eraser_bounds = AABB::from_half_extents(
                    na::Point2::from(element.pos),
                    na::Vector2::repeat(self.width),
                );
                store.trash_colliding_strokes(eraser_bounds, camera.viewport());

                *current_element = element;
                PenProgress::InProgress
            }
            (EraserState::Up, _) => PenProgress::Idle,
            (EraserState::Down { .. }, PenEvent::Up { .. }) => {
                self.state = EraserState::Up;
                PenProgress::Finished
            }
            (EraserState::Down { .. }, PenEvent::Proximity { .. }) => {
                self.state = EraserState::Up;
                PenProgress::Finished
            }
            (EraserState::Down { .. }, PenEvent::Cancel) => {
                self.state = EraserState::Up;
                PenProgress::Finished
            }
        };

        (pen_progress, surface_flags)
    }
}

impl Eraser {
    const OUTLINE_WIDTH: f64 = 2.0;
    const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[2].with_a8(0xf0);
    const FILL_COLOR: piet::Color = color::GNOME_REDS[0].with_a8(0x30);

    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 500.0;
    pub const WIDTH_DEFAULT: f64 = 30.0;

    pub fn new(width: f64) -> Self {
        Self {
            width,
            state: EraserState::Up,
        }
    }
}

impl DrawOnSheetBehaviour for Eraser {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        match &self.state {
            EraserState::Up => None,
            EraserState::Down(current_element) => Some(AABB::from_half_extents(
                na::Point2::from(current_element.pos),
                na::Vector2::from_element(self.width * 0.5),
            )),
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        sheet_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        if let Some(bounds) = self.bounds_on_sheet(sheet_bounds, camera) {
            let fill_rect = bounds.to_kurbo_rect();
            let outline_rect = bounds.tightened(Self::OUTLINE_WIDTH * 0.5).to_kurbo_rect();

            cx.fill(fill_rect, &Self::FILL_COLOR);
            cx.stroke(outline_rect, &Self::OUTLINE_COLOR, Self::OUTLINE_WIDTH);
        }

        Ok(())
    }
}
