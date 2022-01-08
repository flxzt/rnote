pub mod brush;
pub mod eraser;
pub mod marker;
pub mod penbehaviour;
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
    pub current_pen: PenStyle,

    pub marker: Marker,
    pub brush: Brush,
    pub shaper: Shaper,
    pub eraser: Eraser,
    pub selector: Selector,
    pub tools: Tools,
}

impl PenBehaviour for Pens {
    fn begin(
        &mut self,
        data_entries: std::collections::VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        self.set_shown(true);

        match self.current_pen() {
            PenStyle::Marker => {
                self.marker.begin(data_entries, appwindow);
            }
            PenStyle::Brush => {
                self.brush.begin(data_entries, appwindow);
            }
            PenStyle::Shaper => {
                self.shaper.begin(data_entries, appwindow);
            }
            PenStyle::Eraser => {
                self.eraser.begin(data_entries, appwindow);
            }
            PenStyle::Selector => {
                self.selector.begin(data_entries, appwindow);
            }
            PenStyle::Tools => {
                self.tools.begin(data_entries, appwindow);
            }
            PenStyle::Unknown => {}
        }
    }

    fn motion(
        &mut self,
        data_entries: std::collections::VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        match self.current_pen() {
            PenStyle::Marker => {
                self.marker.motion(data_entries, appwindow);
            }
            PenStyle::Brush => {
                self.brush.motion(data_entries, appwindow);
            }
            PenStyle::Shaper => {
                self.shaper.motion(data_entries, appwindow);
            }
            PenStyle::Eraser => {
                self.eraser.motion(data_entries, appwindow);
            }
            PenStyle::Selector => {
                self.selector.motion(data_entries, appwindow);
            }
            PenStyle::Tools => {
                self.tools.motion(data_entries, appwindow);
            }
            PenStyle::Unknown => {}
        }
    }

    fn end(
        &mut self,
        data_entries: std::collections::VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        match self.current_pen() {
            PenStyle::Marker => {
                self.marker.end(data_entries, appwindow);
            }
            PenStyle::Brush => {
                self.brush.end(data_entries, appwindow);
            }
            PenStyle::Shaper => {
                self.shaper.end(data_entries, appwindow);
            }
            PenStyle::Eraser => {
                self.eraser.end(data_entries, appwindow);
            }
            PenStyle::Selector => {
                self.selector.end(data_entries, appwindow);
            }
            PenStyle::Tools => {
                self.tools.end(data_entries, appwindow);
            }
            PenStyle::Unknown => {}
        }

        self.set_shown(false);
    }

    fn draw(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
        renderer: &Renderer,
        zoom: f64,
        snapshot: &Snapshot,
    ) -> Result<(), anyhow::Error> {
        if self.shown {
            match self.current_pen {
                PenStyle::Marker => {
                    self.marker.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Brush => {
                    self.brush.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Shaper => {
                    self.shaper.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Eraser => {
                    self.eraser.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Selector => {
                    self.selector.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Tools => {
                    self.tools.draw(sheet_bounds, renderer, zoom, snapshot)?;
                }
                PenStyle::Unknown => {}
            }
        }

        Ok(())
    }
}

impl Pens {
    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn current_pen(&self) -> PenStyle {
        self.current_pen
    }

    pub fn set_current_pen(&mut self, current_pen: PenStyle) {
        self.current_pen = current_pen;
    }
}
