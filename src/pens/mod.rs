pub mod brush;
pub mod eraser;
pub mod marker;
pub mod selector;

use self::{brush::Brush, eraser::Eraser, marker::Marker, selector::Selector};

use gtk4::Snapshot;

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
