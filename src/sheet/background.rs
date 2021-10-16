use std::error::Error;

use gtk4::{graphene, Snapshot};
use serde::{Deserialize, Serialize};
use svg::node::element;

use crate::utils;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Background {
    Solid(utils::Color),
}

impl Default for Background {
    fn default() -> Self {
        Self::Solid(utils::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
    }
}

impl Background {
    pub fn draw(&self, snapshot: &Snapshot, bounds: &graphene::Rect) {
        match self {
            Self::Solid(color) => {
                snapshot.append_color(&color.to_gdk(), bounds);
            }
        }
    }

    pub fn gen_svg_data(
        &self,
        bounds: p2d::bounding_volume::AABB,
    ) -> Result<String, Box<dyn Error>> {
        match self {
            Self::Solid(color) => {
                let rect = element::Rectangle::new()
                    .set("x", bounds.mins[0])
                    .set("y", bounds.mins[1])
                    .set("width", bounds.maxs[0] - bounds.mins[0])
                    .set("height", bounds.maxs[1] - bounds.mins[1])
                    .set("fill", color.to_css_color());

                rough_rs::node_to_string(&rect)
            }
        }
    }
}
