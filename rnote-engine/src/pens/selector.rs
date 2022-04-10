use super::AudioPlayer;
use super::penbehaviour::PenBehaviour;
use crate::sheet::Sheet;
use crate::{Camera, DrawOnSheetBehaviour, StrokesState, SurfaceFlags};
use rnote_compose::helpers::Vector2Helpers;
use rnote_compose::penpath::Element;
use rnote_compose::{Color, PenEvent};

use gtk4::glib;
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, glib::Enum)]
#[serde(rename = "selector_style")]
#[enum_type(name = "SelectorStyle")]
pub enum SelectorType {
    #[serde(rename = "polygon")]
    #[enum_value(name = "Polygon", nick = "polygon")]
    Polygon,
    #[serde(rename = "rectangle")]
    #[enum_value(name = "Rectangle", nick = "rectangle")]
    Rectangle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "selector")]
pub struct Selector {
    #[serde(rename = "style")]
    pub style: SelectorType,
    #[serde(skip)]
    pub path: Vec<Element>,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            style: SelectorType::Polygon,
            path: vec![],
        }
    }
}

impl PenBehaviour for Selector {
    fn handle_event(
        &mut self,
        event: PenEvent,
        _sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        camera: &mut Camera,
        _audioplayer: Option<&mut AudioPlayer>,
    ) -> SurfaceFlags {
        let surface_flags = SurfaceFlags::default();

        match event {
            PenEvent::Down {
                element,
                shortcut_key: _,
            } => {
                let style = self.style;

                match style {
                    SelectorType::Polygon => {
                        self.path.push(element);
                    }
                    SelectorType::Rectangle => {
                        self.path.push(element);

                        if self.path.len() > 2 {
                            self.path.resize(2, Element::default());
                            self.path.insert(1, element);
                        }
                    }
                }
            }
            PenEvent::Up { .. } => {
                strokes_state.update_selection_for_selector(&self, Some(camera.viewport()));

                let selection_keys = strokes_state.selection_keys_as_rendered();
                strokes_state
                    .regenerate_rendering_for_strokes(&selection_keys, camera.image_scale());

                self.path.clear();
            }
            PenEvent::Proximity { .. } => {}
            PenEvent::Cancel => {
                strokes_state.update_selection_for_selector(&self, Some(camera.viewport()));

                let selection_keys = strokes_state.selection_keys_as_rendered();
                strokes_state
                    .regenerate_rendering_for_strokes(&selection_keys, camera.image_scale());

                self.path.clear();
            }
        }

        surface_flags
    }
}

impl DrawOnSheetBehaviour for Selector {
    fn bounds_on_sheet(&self, _sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        // Making sure bounds are always outside of coord + width
        let mut path_iter = self.path.iter();
        if let Some(first) = path_iter.next() {
            let mut new_bounds = AABB::from_half_extents(na::Point2::from(first.pos), na::Vector2::repeat(Self::PATH_WIDTH / camera.zoom()));

            path_iter.for_each(|element| {
                let pos_bounds = AABB::from_half_extents(na::Point2::from(element.pos), na::Vector2::repeat(Self::PATH_WIDTH / camera.zoom()));
                new_bounds.merge(&pos_bounds);
            });

            Some(new_bounds)
        } else {
            None
        }
    }

    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        _sheet_bounds: AABB,
        camera: &Camera,
    ) -> Result<(), anyhow::Error> {
        let total_zoom = camera.total_zoom();
        let mut bez_path = kurbo::BezPath::new();

        match self.style {
            SelectorType::Polygon => {
                for (i, element) in self.path.iter().enumerate() {
                    if i == 0 {
                        bez_path.move_to((element.pos).to_kurbo_point());
                    } else {
                        bez_path.line_to((element.pos).to_kurbo_point());
                    }
                }
            }
            SelectorType::Rectangle => {
                if let (Some(first), Some(last)) = (self.path.first(), self.path.last()) {
                    bez_path.move_to(first.pos.to_kurbo_point());
                    bez_path.line_to(kurbo::Point::new(last.pos[0], first.pos[1]));
                    bez_path.line_to(kurbo::Point::new(last.pos[0], last.pos[1]));
                    bez_path.line_to(kurbo::Point::new(first.pos[0], last.pos[1]));
                    bez_path.line_to(kurbo::Point::new(first.pos[0], first.pos[1]));
                }
            }
        }
        bez_path.close_path();

        cx.fill(
            bez_path.clone(),
            &piet::PaintBrush::Color(Self::FILL_COLOR.into()),
        );
        cx.stroke_styled(
            bez_path,
            &piet::PaintBrush::Color(Self::OUTLINE_COLOR.into()),
            Self::PATH_WIDTH / total_zoom,
            &piet::StrokeStyle::new().dash_pattern(&Self::DASH_PATTERN),
        );

        Ok(())
    }
}

impl Selector {
    pub const PATH_WIDTH: f64 = 1.8;
    pub const OUTLINE_COLOR: Color = Color {
        r: 0.6,
        g: 0.6,
        b: 0.6,
        a: 0.8,
    };
    pub const FILL_COLOR: Color = Color {
        r: 0.85,
        g: 0.85,
        b: 0.85,
        a: 0.15,
    };

    pub const DASH_PATTERN: [f64; 2] = [8.0, 12.0];
}
