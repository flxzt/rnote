use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::compose::color::Color;
use crate::compose::geometry::AABBHelpers;
use crate::render::Renderer;
use crate::sheet::Sheet;
use crate::strokes::inputdata::InputData;

use gtk4::{graphene, gsk, Snapshot};
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenBehaviour;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "eraser")]
pub struct Eraser {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(skip)]
    pub current_input: Option<InputData>,
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
    fn begin(
        &mut self,
        mut data_entries: VecDeque<InputData>,
        _sheet: &mut Sheet,
        _viewport: Option<AABB>,
        _zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) {
        self.current_input = data_entries.pop_back();
    }

    fn motion(
        &mut self,
        mut data_entries: VecDeque<InputData>,
        sheet: &mut Sheet,
        viewport: Option<AABB>,
        _zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) {
        self.current_input = data_entries.pop_back();

        sheet.strokes_state.trash_colliding_strokes(&self, viewport);
    }

    fn end(
        &mut self,
        _data_entries: VecDeque<InputData>,
        _sheet: &mut Sheet,
        _viewport: Option<AABB>,
        _zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) {
        self.current_input = None;

        /*         // Reset to previous if tmperaser was enabled
        gtk4::prelude::ActionGroupExt::activate_action(
            appwindow,
            "tmperaser",
            Some(&false.to_variant()),
        );
        */
    }

    fn draw(
        &self,
        snapshot: &Snapshot,
        _sheet: &Sheet,
        _viewport: Option<AABB>,
        zoom: f64,
        _renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        if let Some(bounds) = self.gen_bounds(zoom) {
            let border_color = Self::OUTLINE_COLOR_DEFAULT.to_gdk();
            let border_width = 2.0;

            snapshot.append_color(
                &Self::FILL_COLOR_DEFAULT.to_gdk(),
                &bounds.to_graphene_rect(),
            );

            snapshot.append_border(
                &gsk::RoundedRect::new(
                    bounds.to_graphene_rect(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                ),
                &[border_width, border_width, border_width, border_width],
                &[border_color, border_color, border_color, border_color],
            );
        }
        Ok(())
    }
}

impl Eraser {
    pub const OUTLINE_COLOR_DEFAULT: Color = Color {
        r: 0.8,
        g: 0.1,
        b: 0.0,
        a: 0.5,
    };
    pub const FILL_COLOR_DEFAULT: Color = Color {
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

    pub fn gen_bounds(&self, zoom: f64) -> Option<AABB> {
        self.current_input.map_or_else(
            || None,
            |current_input| {
                Some(AABB::new(
                    na::point![
                        ((current_input.pos()[0]) - self.width / 2.0) * zoom,
                        ((current_input.pos()[1]) - self.width / 2.0) * zoom
                    ],
                    na::point![
                        ((current_input.pos()[0]) + self.width / 2.0) * zoom,
                        ((current_input.pos()[1]) + self.width / 2.0) * zoom
                    ],
                ))
            },
        )
    }
}
