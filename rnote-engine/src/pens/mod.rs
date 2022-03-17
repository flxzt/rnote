pub mod penbehaviour;
pub mod shortcuts;

pub mod brush;
pub mod eraser;
pub mod selector;
pub mod shaper;
pub mod tools;

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::render::Renderer;
use crate::sheet::Sheet;
use crate::strokes::inputdata::InputData;
use crate::surfaceflags::SurfaceFlags;

use self::penbehaviour::PenBehaviour;
use self::shortcuts::{ShortcutAction, ShortcutKey, Shortcuts};
use self::tools::Tools;
use self::{brush::Brush, eraser::Eraser, selector::Selector, shaper::Shaper};
use gtk4::{glib, glib::prelude::*, Snapshot};
use num_derive::FromPrimitive;
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

#[derive(
    Eq,
    PartialEq,
    Clone,
    Copy,
    Debug,
    glib::Enum,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    Hash,
    FromPrimitive,
)]
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

impl TryFrom<u32> for PenStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or(anyhow::anyhow!(
            "PenStyle try_from::<u32>() for value {} failed",
            value
        ))
    }
}

impl PenStyle {
    pub fn nick(self) -> String {
        glib::EnumValue::from_value(&self.to_value())
            .unwrap()
            .1
            .nick()
            .to_string()
    }

    pub fn display_name(self) -> String {
        match self {
            PenStyle::BrushStyle => String::from("Brush style"),
            PenStyle::ShaperStyle => String::from("Shaper style"),
            PenStyle::EraserStyle => String::from("Eraser style"),
            PenStyle::SelectorStyle => String::from("Selector style"),
            PenStyle::ToolsStyle => String::from("Tools style"),
        }
    }
    pub fn icon_name(self) -> String {
        match self {
            Self::BrushStyle => String::from("pen-brush-symbolic"),
            Self::ShaperStyle => String::from("pen-shaper-symbolic"),
            Self::EraserStyle => String::from("pen-eraser-symbolic"),
            Self::SelectorStyle => String::from("pen-selector-symbolic"),
            Self::ToolsStyle => String::from("pen-tools-symbolic"),
        }
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

#[derive(Debug, Clone)]
pub enum PenEvent {
    DownEvent {
        data_entries: VecDeque<InputData>,
        shortcut_key: Option<ShortcutKey>,
    },
    MotionEvent {
        data_entries: VecDeque<InputData>,
    },
    UpEvent {
        data_entries: VecDeque<InputData>,
    },
    ChangeStyle(PenStyle),
    ChangeStyleOverride(Option<PenStyle>),
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename = "pens")]
pub struct Pens {
    #[serde(rename = "style")]
    style: PenStyle,
    #[serde(rename = "shortcuts")]
    shortcuts: Shortcuts,

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
    #[serde(skip)]
    style_override: Option<PenStyle>,
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

    /// Gets the current override
    pub fn style_override(&self) -> Option<PenStyle> {
        self.style_override
    }

    /// Gets the current style, or the override if it is set.
    pub fn style_w_override(&self) -> PenStyle {
        self.style_override.unwrap_or(self.style)
    }

    pub fn register_new_shortcut(&mut self, key: ShortcutKey, action: ShortcutAction) {
        self.shortcuts.insert(key, action);
    }

    pub fn remove_shortcut(&mut self, key: ShortcutKey) -> Option<ShortcutAction> {
        self.shortcuts.remove(&key)
    }

    pub fn list_current_shortcuts(&self) -> Vec<(ShortcutKey, ShortcutAction)> {
        self.shortcuts
            .iter()
            .map(|(key, action)| (key.clone(), action.clone()))
            .collect()
    }

    pub fn handle_event(
        &mut self,
        event: PenEvent,
        sheet: &mut crate::sheet::Sheet,
        viewport: Option<AABB>,
        zoom: f64,
        renderer: Arc<RwLock<Renderer>>,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();
        /*
               log::debug!(
                   "handle_event() with state: {:?}, event: {:?}, style: {:?}, style_override: {:?}",
                   self.state,
                   event,
                   self.style,
                   self.style_override
               );
        */
        match (self.state, event) {
            (
                PenState::Up,
                PenEvent::DownEvent {
                    data_entries,
                    shortcut_key,
                },
            ) => {
                if let Some(shortcut_key) = shortcut_key {
                    self.handle_shortcut_key(shortcut_key, &mut surface_flags);
                }

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

                surface_flags.redraw = true;
                self.state = PenState::Down;
                self.pen_shown = true;
            }
            (
                PenState::Down,
                PenEvent::DownEvent {
                    data_entries: _,
                    shortcut_key: _,
                },
            ) => {}
            (PenState::Up, PenEvent::MotionEvent { data_entries: _ }) => {}
            (PenState::Down, PenEvent::MotionEvent { data_entries }) => {
                match self.style_w_override() {
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
                }

                surface_flags.redraw = true;
            }
            (PenState::Up, PenEvent::UpEvent { data_entries: _ }) => {}
            (PenState::Down, PenEvent::UpEvent { data_entries }) => {
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

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.sheet_changed = true;
                surface_flags.selection_changed = true;

                self.state = PenState::Up;
                self.pen_shown = false;
                // Disable the style override after finishing the stroke
                self.style_override = None;
            }
            (PenState::Down, PenEvent::ChangeStyle(new_style)) => {
                if self.style() != new_style {
                    // before changing the style, the current stroke is finished
                    match self.style_w_override() {
                        PenStyle::BrushStyle => {
                            self.brush
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::ShaperStyle => {
                            self.shaper
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::EraserStyle => {
                            self.eraser
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::SelectorStyle => {
                            self.selector
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::ToolsStyle => {
                            self.tools
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                    }

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;
                    surface_flags.selection_changed = true;

                    self.state = PenState::Up;
                    self.pen_shown = false;
                    self.style = new_style;
                }
            }
            (PenState::Up, PenEvent::ChangeStyle(new_style)) => {
                self.style = new_style;
            }
            (PenState::Down, PenEvent::ChangeStyleOverride(new_style_override)) => {
                if self.style_override() != new_style_override {
                    // before changing the style override, the current stroke is finished
                    match self.style_w_override() {
                        PenStyle::BrushStyle => {
                            self.brush
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::ShaperStyle => {
                            self.shaper
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::EraserStyle => {
                            self.eraser
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::SelectorStyle => {
                            self.selector
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                        PenStyle::ToolsStyle => {
                            self.tools
                                .end(VecDeque::new(), sheet, viewport, zoom, renderer);
                        }
                    }

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.sheet_changed = true;
                    surface_flags.selection_changed = true;

                    self.pen_shown = false;
                    self.state = PenState::Up;
                    self.style_override = new_style_override;
                }
            }
            (PenState::Up, PenEvent::ChangeStyleOverride(new_style_override)) => {
                self.style_override = new_style_override;
            }
        }

        surface_flags
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

    fn handle_shortcut_key(&mut self, shortcut_key: ShortcutKey, surface_flags: &mut SurfaceFlags) {
        if let Some(&action) = self.shortcuts.get(&shortcut_key) {
            match action {
                ShortcutAction::ChangePenStyle { style, permanent } => {
                    if permanent {
                        self.style = style;
                        surface_flags.pen_change = Some(style);
                    } else {
                        self.style_override = Some(style);
                    }
                }
            }
        }
    }
}
