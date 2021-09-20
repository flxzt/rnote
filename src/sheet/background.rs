use gtk4::{graphene, Snapshot};
use serde::{Deserialize, Serialize};

use crate::pens;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Background {
    Solid(pens::Color),
}

impl Default for Background {
    fn default() -> Self {
        Self::Solid(pens::Color {
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
            Background::Solid(color) => {
                snapshot.append_color(&color.to_gdk(), bounds);
            }
        }
    }
}
