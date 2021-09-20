pub mod brush;
pub mod eraser;
pub mod marker;
pub mod selector;

use self::{brush::Brush, eraser::Eraser, marker::Marker, selector::Selector};

use gtk4::{gdk, Snapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32, // between 0.0 and 1.0
    pub g: f32, // between 0.0 and 1.0
    pub b: f32, // between 0.0 and 1.0
    pub a: f32, // between 0.0 and 1.0
}

impl Color {
    pub fn from_gdk(gdk_color: gdk::RGBA) -> Self {
        Self {
            r: gdk_color.red,
            g: gdk_color.green,
            b: gdk_color.blue,
            a: gdk_color.alpha,
        }
    }

    pub fn to_gdk(&self) -> gdk::RGBA {
        gdk::RGBA {
            red: self.r,
            green: self.g,
            blue: self.b,
            alpha: self.a,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum PenStyle {
    Marker,
    Brush,
    Eraser,
    Selector,
    Unkown,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self::Marker
    }
}

#[derive(Default, Clone, Debug)]
pub struct Pens {
    pub marker: Marker,
    pub brush: Brush,
    pub eraser: Eraser,
    pub selector: Selector,
}

impl Pens {
    pub fn draw_pens(&self, current_pen: PenStyle, snapshot: &Snapshot, scalefactor: f64) {
        match current_pen {
            PenStyle::Eraser => {
                self.eraser.draw(scalefactor, snapshot);
            }
            PenStyle::Selector => {
                self.selector.draw(&snapshot);
            }
            PenStyle::Marker | PenStyle::Brush | PenStyle::Unkown => {}
        }
    }
}
