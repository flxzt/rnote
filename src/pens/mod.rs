pub mod brush;
pub mod eraser;
pub mod marker;
pub mod penbehaviour;
pub mod selector;
pub mod shaper;
pub mod tools;

use crate::ui::canvas::Canvas;

use self::penbehaviour::PenBehaviour;
use self::tools::Tools;
use self::{brush::Brush, eraser::Eraser, marker::Marker, selector::Selector, shaper::Shaper};
use gtk4::glib;
use serde::{Deserialize, Serialize};

use gtk4::Snapshot;

#[derive(Eq, PartialEq, Clone, Copy, Debug, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "PenStyle")]
#[serde(rename = "pen_style")]
pub enum PenStyle {
    #[enum_value(name = "MarkerStyle", nick = "marker_style")]
    #[serde(rename = "marker_style")]
    MarkerStyle = 0,
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
        Self::MarkerStyle
    }
}

impl PenStyle {
    pub fn begin(
        self,
        data_entries: std::collections::VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        match self {
            PenStyle::MarkerStyle => {
                Marker::begin(data_entries, appwindow);
            }
            PenStyle::BrushStyle => {
                Brush::begin(data_entries, appwindow);
            }
            PenStyle::ShaperStyle => {
                Shaper::begin(data_entries, appwindow);
            }
            PenStyle::EraserStyle => {
                Eraser::begin(data_entries, appwindow);
            }
            PenStyle::SelectorStyle => {
                Selector::begin(data_entries, appwindow);
            }
            PenStyle::ToolsStyle => {
                Tools::begin(data_entries, appwindow);
            }
        }
    }

    pub fn motion(
        self,
        data_entries: std::collections::VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        match self {
            PenStyle::MarkerStyle => {
                Marker::motion(data_entries, appwindow);
            }
            PenStyle::BrushStyle => {
                Brush::motion(data_entries, appwindow);
            }
            PenStyle::ShaperStyle => {
                Shaper::motion(data_entries, appwindow);
            }
            PenStyle::EraserStyle => {
                Eraser::motion(data_entries, appwindow);
            }
            PenStyle::SelectorStyle => {
                Selector::motion(data_entries, appwindow);
            }
            PenStyle::ToolsStyle => {
                Tools::motion(data_entries, appwindow);
            }
        }
    }

    pub fn end(
        self,
        data_entries: std::collections::VecDeque<crate::strokes::strokestyle::InputData>,
        appwindow: &crate::ui::appwindow::RnoteAppWindow,
    ) {
        match self {
            PenStyle::MarkerStyle => {
                Marker::end(data_entries, appwindow);
            }
            PenStyle::BrushStyle => {
                Brush::end(data_entries, appwindow);
            }
            PenStyle::ShaperStyle => {
                Shaper::end(data_entries, appwindow);
            }
            PenStyle::EraserStyle => {
                Eraser::end(data_entries, appwindow);
            }
            PenStyle::SelectorStyle => {
                Selector::end(data_entries, appwindow);
            }
            PenStyle::ToolsStyle => {
                Tools::end(data_entries, appwindow);
            }
        }
    }

    pub fn draw(self, canvas: &Canvas, snapshot: &Snapshot) -> Result<(), anyhow::Error> {
        let sheet_bounds = canvas.sheet().borrow().bounds();
        let renderer = canvas.renderer();
        let zoom = canvas.zoom();

        match self {
            PenStyle::MarkerStyle => {
                canvas
                    .pens()
                    .borrow()
                    .marker
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
            PenStyle::BrushStyle => {
                canvas
                    .pens()
                    .borrow()
                    .brush
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
            PenStyle::ShaperStyle => {
                canvas
                    .pens()
                    .borrow()
                    .shaper
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
            PenStyle::EraserStyle => {
                canvas
                    .pens()
                    .borrow()
                    .eraser
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
            PenStyle::SelectorStyle => {
                canvas
                    .pens()
                    .borrow()
                    .selector
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
            PenStyle::ToolsStyle => {
                canvas
                    .pens()
                    .borrow()
                    .tools
                    .draw(sheet_bounds, zoom, snapshot, renderer)?;
            }
        }

        Ok(())
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "pens")]
pub struct Pens {
    #[serde(rename = "current_pen")]
    pub current_pen: PenStyle,

    #[serde(rename = "marker")]
    pub marker: Marker,
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
}
