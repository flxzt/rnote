use crate::{Camera, DrawOnSheetBehaviour, Sheet, StrokesState};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::penpath::Element;
use rnote_compose::{Color, PenEvent};

use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

use super::AudioPlayer;
use super::penbehaviour::PenBehaviour;

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
        strokes_state: &mut StrokesState,
        camera: &Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) {
        match event {
            PenEvent::Down {
                element,
                shortcut_key: _,
            } => {
                self.current_input = Some(element);

                strokes_state.trash_colliding_strokes(&self, Some(camera.viewport()));
            }
            PenEvent::Up { .. } => self.current_input = None,
            PenEvent::Proximity { .. } => self.current_input = None,
            PenEvent::Cancel => self.current_input = None,
        }
    }
}

impl Eraser {
    const OUTLINE_WIDTH: f64 = 2.0;
    const OUTLINE_COLOR: Color = Color {
        r: 0.8,
        g: 0.1,
        b: 0.0,
        a: 0.5,
    };
    const FILL_COLOR: Color = Color {
        r: 0.7,
        g: 0.2,
        b: 0.1,
        a: 0.5,
    };
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
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, _viewport: AABB) -> Option<AABB> {
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
        viewport: AABB,
    ) -> Result<(), anyhow::Error> {
        if let Some(bounds) = self.bounds_on_sheet(sheet_bounds, viewport) {
            let fill_rect = bounds.to_kurbo_rect();
            let outline_rect = bounds.tightened(Self::OUTLINE_WIDTH * 0.5).to_kurbo_rect();

            cx.fill(fill_rect, &piet::PaintBrush::Color(Self::FILL_COLOR.into()));
            cx.stroke(
                outline_rect,
                &piet::PaintBrush::Color(Self::OUTLINE_COLOR.into()),
                Self::OUTLINE_WIDTH,
            );
        }

        Ok(())
    }
}
