pub mod brush;
pub mod eraser;
pub mod marker;
pub mod selector;
pub mod shaper;
pub mod tools;

use self::tools::Tools;
use self::{brush::Brush, eraser::Eraser, marker::Marker, selector::Selector, shaper::Shaper};

use gtk4::Snapshot;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum PenStyle {
    Marker,
    Brush,
    Shaper,
    Eraser,
    Selector,
    Tools,
    Unknown,
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
    pub shaper: Shaper,
    pub eraser: Eraser,
    pub selector: Selector,
    pub tools: Tools,
}

impl Pens {
    pub fn draw(&self, current_pen: PenStyle, snapshot: &Snapshot, zoom: f64) {
        match current_pen {
            PenStyle::Eraser => {
                self.eraser.draw(zoom, snapshot);
            }
            PenStyle::Selector => {
                self.selector.draw(snapshot);
            }
            PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper | PenStyle::Tools | PenStyle::Unknown => {}
        }
    }
}
