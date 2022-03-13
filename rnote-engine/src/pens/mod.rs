pub mod brush;
pub mod eraser;
pub mod penbehaviour;
pub mod selector;
pub mod shaper;
pub mod tools;

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::render::Renderer;
use crate::sheet::Sheet;
use crate::strokes::inputdata::InputData;

use self::penbehaviour::PenBehaviour;
use self::tools::Tools;
use self::{brush::Brush, eraser::Eraser, selector::Selector, shaper::Shaper};
use gtk4::{glib, Snapshot};
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Copy, Debug, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "PenStyle")]
#[serde(rename = "pen_style")]
pub enum PenStyle {
    #[enum_value(name = "BrushStyle", nick = "brush_style")]
    #[serde(rename = "brush_style")]
    BrushStyle,
    #[enum_value(name = "ShaperStyle", nick = "shaper_style")]
    #[serde(rename = "shaper_style")]
    ShaperStyle,
    #[enum_value(name = "EraserStyle", nick = "eraser_style")]
    #[serde(rename = "eraser_style")]
    EraserStyle,
    #[enum_value(name = "SelectorStyle", nick = "selector_style")]
    #[serde(rename = "selector_style")]
    SelectorStyle,
    #[enum_value(name = "ToolsStyle", nick = "tools_style")]
    #[serde(rename = "tools_style")]
    ToolsStyle,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self::BrushStyle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenState {
    Up,
    Down,
}

impl Default for PenState {
    fn default() -> Self {
        Self::Up
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenEvent {
    DownEvent,
    MotionEvent,
    UpEvent,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "pens")]
pub struct Pens {
    #[serde(rename = "style")]
    style: PenStyle,
    #[serde(skip)]
    style_override: Option<PenStyle>,

    #[serde(rename = "brush")]
    pub brush: Brush,
    #[serde(rename = "shaper")]
    pub shaper: Shaper,
    #[serde(rename = "eraser")]
    pub eraser: Eraser,
    #[serde(rename = "selector")]
    pub selector: Selector,
    #[serde(rename = "tools")]
    pub tools: Tools,

    #[serde(skip)]
    pen_shown: bool,
    #[serde(skip)]
    state: PenState,
}

impl Pens {
    /// If the pen is currently shown.
    pub fn pen_shown(&self) -> bool {
        self.pen_shown
    }

    /// gets the pen style. May be overriden by style_override.
    pub fn style(&self) -> PenStyle {
        self.style
    }

    /// Sets the style. Only has an effect if the current pen state is PenState::Up
    pub fn set_style(&mut self, style: PenStyle) {
        if self.state == PenState::Up {
            self.style = style;
        }
    }

    /// Gets the current override
    pub fn style_override(&self) -> Option<PenStyle> {
        self.style_override
    }

    /// Sets the style override. Only has an effect if the current pen state is PenState::Up
    pub fn set_style_override(&mut self, style_override: Option<PenStyle>) {
        if self.state == PenState::Up {
            self.style_override = style_override;
        }
    }

    /// Gets the current style, or the override if it is set.
    pub fn style_w_override(&self) -> PenStyle {
        self.style_override.unwrap_or(self.style)
    }

    pub fn handle_event(
        &mut self,
        event: PenEvent,
        data_entries: VecDeque<InputData>,
        sheet: &mut crate::sheet::Sheet,
        viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) {
/*         log::debug!(
            "handle_event() with state: {:?}, event: {:?}, style: {:?}, style_override: {:?}",
            self.state,
            event,
            self.style,
            self.style_override
        ); */

        match (self.state, event) {
            (PenState::Up, PenEvent::DownEvent) => {
                self.state = PenState::Down;
                self.pen_shown = true;

                match self.style_w_override() {
                    PenStyle::BrushStyle => {
                        self.brush
                            .begin(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::ShaperStyle => {
                        self.shaper
                            .begin(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::EraserStyle => {
                        self.eraser
                            .begin(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::SelectorStyle => {
                        self.selector
                            .begin(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::ToolsStyle => {
                        self.tools
                            .begin(data_entries, sheet, viewport, zoom, renderer);
                    }
                }
            }
            (PenState::Down, PenEvent::DownEvent) => {}
            (PenState::Up, PenEvent::MotionEvent) => {}
            (PenState::Down, PenEvent::MotionEvent) => match self.style_w_override() {
                PenStyle::BrushStyle => {
                    self.brush
                        .motion(data_entries, sheet, viewport, zoom, renderer);
                }
                PenStyle::ShaperStyle => {
                    self.shaper
                        .motion(data_entries, sheet, viewport, zoom, renderer);
                }
                PenStyle::EraserStyle => {
                    self.eraser
                        .motion(data_entries, sheet, viewport, zoom, renderer);
                }
                PenStyle::SelectorStyle => {
                    self.selector
                        .motion(data_entries, sheet, viewport, zoom, renderer);
                }
                PenStyle::ToolsStyle => {
                    self.tools
                        .motion(data_entries, sheet, viewport, zoom, renderer);
                }
            },
            (PenState::Up, PenEvent::UpEvent) => {}
            (PenState::Down, PenEvent::UpEvent) => {
                self.state = PenState::Up;

                // We deselect the selection here, before updating it when the current style is the selector
                let all_strokes = sheet.strokes_state.keys_sorted_chrono();
                sheet.strokes_state.set_selected_keys(&all_strokes, false);

                match self.style_w_override() {
                    PenStyle::BrushStyle => {
                        self.brush
                            .end(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::ShaperStyle => {
                        self.shaper
                            .end(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::EraserStyle => {
                        self.eraser
                            .end(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::SelectorStyle => {
                        self.selector
                            .end(data_entries, sheet, viewport, zoom, renderer);
                    }
                    PenStyle::ToolsStyle => {
                        self.tools
                            .end(data_entries, sheet, viewport, zoom, renderer);
                    }
                }

                self.pen_shown = false;
                self.style_override = None;
            }
        }
    }

    pub fn draw(
        &self,
        snapshot: &Snapshot,
        sheet: &Sheet,
        viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<(), anyhow::Error> {
        if self.pen_shown {
            match self.style_w_override() {
                PenStyle::BrushStyle => self.brush.draw(snapshot, sheet, viewport, zoom, renderer),
                PenStyle::ShaperStyle => {
                    self.shaper.draw(snapshot, sheet, viewport, zoom, renderer)
                }
                PenStyle::EraserStyle => {
                    self.eraser.draw(snapshot, sheet, viewport, zoom, renderer)
                }
                PenStyle::SelectorStyle => self
                    .selector
                    .draw(snapshot, sheet, viewport, zoom, renderer),
                PenStyle::ToolsStyle => self.tools.draw(snapshot, sheet, viewport, zoom, renderer),
            }
        } else {
            Ok(())
        }
    }
}
