use crate::{Camera, DrawOnSheetBehaviour, Sheet, StrokeStore, SurfaceFlags};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penpath::Element;
use rnote_compose::{color, PenEvent};

use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;
use super::AudioPlayer;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "eraser")]
pub struct Eraser {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(skip)]
    pub current_input: Option<Element>,
}

impl Default for Eraser {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            current_input: None,
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
    ) -> SurfaceFlags {
        let surface_flags = SurfaceFlags::default();

        match event {
            PenEvent::Down {
                element,
                shortcut_key: _,
            } => {
                self.current_input = Some(element);

                let eraser_bounds = AABB::from_half_extents(
                    na::Point2::from(element.pos),
                    na::Vector2::repeat(self.width),
                );
                store.trash_colliding_strokes(eraser_bounds, camera.viewport());
            }
            PenEvent::Up { .. } => self.current_input = None,
            PenEvent::Proximity { .. } => self.current_input = None,
            PenEvent::Cancel => self.current_input = None,
        }

        surface_flags
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
            current_input: None,
        }
    }
}

impl DrawOnSheetBehaviour for Eraser {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _camera: &Camera) -> Option<AABB> {
        self.current_input.map(|current_input| {
            AABB::from_half_extents(
                na::Point2::from(current_input.pos),
                na::Vector2::from_element(self.width * 0.5),
            )
        })
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
            cx.stroke(
                outline_rect,
                &Self::OUTLINE_COLOR,
                Self::OUTLINE_WIDTH,
            );
        }

        Ok(())
    }
}
