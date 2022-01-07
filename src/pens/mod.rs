pub mod penbehaviour;
pub mod brush;
pub mod eraser;
pub mod marker;
pub mod selector;
pub mod shaper;
pub mod tools;

use crate::render::Renderer;

use self::penbehaviour::PenBehaviour;
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
    pub shown: bool,
    pub marker: Marker,
    pub brush: Brush,
    pub shaper: Shaper,
    pub eraser: Eraser,
    pub selector: Selector,
    pub tools: Tools,
}

impl Pens {
    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn draw(
        &self,
        current_pen: PenStyle,
        sheet_bounds: p2d::bounding_volume::AABB,
        zoom: f64,
        renderer: &Renderer,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        if self.shown {
            match current_pen {
                PenStyle::Eraser => {
                    self.eraser.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Selector => {
                    self.selector.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Tools => {
                    self.tools.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Marker | PenStyle::Brush | PenStyle::Shaper | PenStyle::Unknown => {}
            }
        }

        Ok(())
    }
}
