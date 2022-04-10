use crate::pens::shortcuts::ShortcutAction;
use crate::pens::Tools;

use crate::sheet::Sheet;
use crate::surfaceflags::SurfaceFlags;
use crate::{Camera, DrawOnSheetBehaviour, StrokesState};
use rnote_compose::penevent::ShortcutKey;
use rnote_compose::PenEvent;

use gtk4::{glib, glib::prelude::*};
use num_derive::FromPrimitive;
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::{AudioPlayer, Brush, Eraser, PenBehaviour, Selector, Shaper, Shortcuts};

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
    #[enum_value(name = "Brush", nick = "brush")]
    #[serde(rename = "brush")]
    Brush,
    #[enum_value(name = "Shaper", nick = "shaper")]
    #[serde(rename = "shaper")]
    Shaper,
    #[enum_value(name = "Eraser", nick = "eraser")]
    #[serde(rename = "eraser")]
    Eraser,
    #[enum_value(name = "Selector", nick = "selector")]
    #[serde(rename = "selector")]
    Selector,
    #[enum_value(name = "Tools", nick = "tools")]
    #[serde(rename = "tools")]
    Tools,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self::Brush
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
    pub fn name(self) -> String {
        glib::EnumValue::from_value(&self.to_value())
            .unwrap()
            .1
            .name()
            .to_string()
    }

    pub fn nick(self) -> String {
        glib::EnumValue::from_value(&self.to_value())
            .unwrap()
            .1
            .nick()
            .to_string()
    }

    pub fn icon_name(self) -> String {
        match self {
            Self::Brush => String::from("pen-brush-symbolic"),
            Self::Shaper => String::from("pen-shaper-symbolic"),
            Self::Eraser => String::from("pen-eraser-symbolic"),
            Self::Selector => String::from("pen-selector-symbolic"),
            Self::Tools => String::from("pen-tools-symbolic"),
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
pub enum PenHolderEvent {
    PenEvent(rnote_compose::PenEvent),
    ChangeStyle(PenStyle),
    ChangeStyleOverride(Option<PenStyle>),
    PressedShortcutkey(ShortcutKey),
}

/// This holds the pens and is the main interaction point when changing the pen style / emitting pen events.
#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
#[serde(default, rename = "penholder")]
pub struct PenHolder {
    // brushes are configurable from the public
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

    // Managed by the internal state machine
    #[serde(rename = "style")]
    style: PenStyle,
    #[serde(rename = "shortcuts")]
    shortcuts: Shortcuts,
    #[serde(skip)]
    pen_shown: bool,
    #[serde(skip)]
    state: PenState,
    #[serde(skip)]
    style_override: Option<PenStyle>,
    #[serde(skip)]
    pub audioplayer: Option<AudioPlayer>,
}

impl Default for PenHolder {
    fn default() -> Self {
        let audioplayer = AudioPlayer::new().map_err(|e| {
            log::error!("failed to create a new audio player in PenHolder::default(), Err {}", e);
        }).ok();

        Self {
            brush: Brush::default(),
            shaper: Shaper::default(),
            eraser: Eraser::default(),
            selector: Selector::default(),
            tools: Tools::default(),

            style: PenStyle::default(),
            shortcuts: Shortcuts::default(),
            pen_shown: false,
            state: PenState::default(),
            style_override: None,
            audioplayer,
        }
    }
}

impl PenHolder {
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

    /// Changes the internal state according to events
    pub(crate) fn handle_event(
        &mut self,
        event: PenHolderEvent,
        sheet: &mut crate::sheet::Sheet,
        strokes_state: &mut StrokesState,
        camera: &mut Camera,
    ) -> SurfaceFlags {
        /*         log::debug!(
            "handle_event() with state: {:?}, event: {:?}, style: {:?}, style_override: {:?}",
            self.state,
            event,
            self.style,
            self.style_override
        ); */

        let mut surface_flags = SurfaceFlags::default();

        match (self.state, event) {
            (
                PenState::Up,
                PenHolderEvent::PenEvent(
                    pen_event @ PenEvent::Down {
                        element: _,
                        shortcut_key,
                    },
                ),
            ) => {
                if let Some(shortcut_key) = shortcut_key {
                    self.change_state_for_shortcut_key(shortcut_key, &mut surface_flags);
                }

                surface_flags.merge_with_other(self.handle_pen_event(pen_event, sheet, strokes_state, camera));

                self.state = PenState::Down;
                self.pen_shown = true;

                surface_flags.redraw = true;
                surface_flags.hide_scrollbars = Some(true);
            }
            (PenState::Down, PenHolderEvent::PenEvent(pen_event @ PenEvent::Down { .. })) => {
                surface_flags.merge_with_other(self.handle_pen_event(pen_event, sheet, strokes_state, camera));

                surface_flags.redraw = true;
            }
            (PenState::Up, PenHolderEvent::PenEvent(PenEvent::Up { .. })) => {}
            (PenState::Down, PenHolderEvent::PenEvent(pen_event @ PenEvent::Up { .. })) => {
                // We deselect the selection here, before updating it when the current style is the selector
                let all_strokes = strokes_state.keys_sorted_chrono();
                strokes_state.set_selected_keys(&all_strokes, false);

                surface_flags.merge_with_other(self.handle_pen_event(pen_event, sheet, strokes_state, camera));

                self.state = PenState::Up;
                self.pen_shown = false;

                // Disable the style override after finishing the stroke
                if self.style_override.take().is_some() {
                    surface_flags.pen_changed = true;
                }

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.sheet_changed = true;
                surface_flags.selection_changed = true;
                surface_flags.hide_scrollbars = Some(false);
            }
            (_, PenHolderEvent::PenEvent(pen_event @ PenEvent::Proximity { .. })) => {
                surface_flags.merge_with_other(self.handle_pen_event(pen_event, sheet, strokes_state, camera));
            }
            (_, PenHolderEvent::PenEvent(pen_event @ PenEvent::Cancel)) => {
                surface_flags.merge_with_other(self.handle_pen_event(pen_event, sheet, strokes_state, camera));

                self.state = PenState::Up;

                surface_flags.redraw = true;
                surface_flags.resize = true;
                surface_flags.sheet_changed = true;
                surface_flags.selection_changed = true;
            }
            (PenState::Down, PenHolderEvent::ChangeStyle(new_style)) => {
                if self.style != new_style {
                    // before changing the style, the current stroke is finished
                    surface_flags.merge_with_other(self.handle_pen_event(PenEvent::Cancel, sheet, strokes_state, camera));

                    self.state = PenState::Up;
                    self.pen_shown = false;
                    self.style = new_style;

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.pen_changed = true;
                    surface_flags.sheet_changed = true;
                    surface_flags.selection_changed = true;
                }
            }
            (PenState::Up, PenHolderEvent::ChangeStyle(new_style)) => {
                if self.style != new_style {
                    self.style = new_style;
                    //self.style_override = None;

                    surface_flags.pen_changed = true;
                    surface_flags.redraw = true;
                }
            }
            (PenState::Down, PenHolderEvent::ChangeStyleOverride(new_style_override)) => {
                if self.style_override != new_style_override {
                    // before changing the style override, the current stroke is finished
                    surface_flags.merge_with_other(self.handle_pen_event(PenEvent::Cancel, sheet, strokes_state, camera));

                    self.pen_shown = false;
                    self.state = PenState::Up;
                    self.style_override = new_style_override;

                    surface_flags.redraw = true;
                    surface_flags.resize = true;
                    surface_flags.pen_changed = true;
                    surface_flags.sheet_changed = true;
                    surface_flags.selection_changed = true;
                }
            }
            (PenState::Up, PenHolderEvent::ChangeStyleOverride(new_style_override)) => {
                if self.style_override != new_style_override {
                    self.style_override = new_style_override;

                    surface_flags.pen_changed = true;
                    surface_flags.redraw = true;
                }
            }
            (PenState::Down, PenHolderEvent::PressedShortcutkey(_)) => {
                // Dont change anything while drawing
                /*                 self.pen_handle_event(PenEvent::Cancel, sheet, viewport, zoom, renderer);

                self.change_state_for_shortcut_key(shortcut_key, &mut surface_flags); */
            }
            (PenState::Up, PenHolderEvent::PressedShortcutkey(shortcut_key)) => {
                self.change_state_for_shortcut_key(shortcut_key, &mut surface_flags);
            }
        }

        surface_flags
    }

    fn handle_pen_event(
        &mut self,
        event: PenEvent,
        sheet: &mut Sheet,
        strokes_state: &mut StrokesState,
        camera: &mut Camera,
    ) -> SurfaceFlags {
        match self.style_w_override() {
            PenStyle::Brush => {
                self.brush
                    .handle_event(event, sheet, strokes_state, camera, self.audioplayer.as_mut())
            }
            PenStyle::Shaper => {
                self.shaper
                    .handle_event(event, sheet, strokes_state, camera, self.audioplayer.as_mut())
            }
            PenStyle::Eraser => {
                self.eraser
                    .handle_event(event, sheet, strokes_state, camera, self.audioplayer.as_mut())
            }
            PenStyle::Selector => {
                self.selector
                    .handle_event(event, sheet, strokes_state, camera, self.audioplayer.as_mut())
            }
            PenStyle::Tools => {
                self.tools
                    .handle_event(event, sheet, strokes_state, camera, self.audioplayer.as_mut())
            }
        }
    }

    fn change_state_for_shortcut_key(
        &mut self,
        shortcut_key: ShortcutKey,
        surface_flags: &mut SurfaceFlags,
    ) {
        if let Some(&action) = self.shortcuts.get(&shortcut_key) {
            match action {
                ShortcutAction::ChangePenStyle { style, permanent } => {
                    if permanent {
                        self.style = style;

                        surface_flags.pen_changed = true;
                    } else {
                        self.style_override = Some(style);

                        surface_flags.pen_changed = true;
                    }
                }
            }
        }
    }
}

impl DrawOnSheetBehaviour for PenHolder {
    fn bounds_on_sheet(&self, sheet_bounds: AABB, camera: &Camera) -> Option<AABB> {
        match self.style_w_override() {
            PenStyle::Brush => self.brush.bounds_on_sheet(sheet_bounds, camera),
            PenStyle::Shaper => self.shaper.bounds_on_sheet(sheet_bounds, camera),
            PenStyle::Eraser => self.eraser.bounds_on_sheet(sheet_bounds, camera),
            PenStyle::Selector => self.selector.bounds_on_sheet(sheet_bounds, camera),
            PenStyle::Tools => self.tools.bounds_on_sheet(sheet_bounds, camera),
        }
    }
    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        sheet_bounds: AABB,
        camera: &Camera,
    ) -> Result<(), anyhow::Error> {
        if self.pen_shown {
            match self.style_w_override() {
                PenStyle::Brush => self.brush.draw_on_sheet(cx, sheet_bounds, camera),
                PenStyle::Shaper => self.shaper.draw_on_sheet(cx, sheet_bounds, camera),
                PenStyle::Eraser => self.eraser.draw_on_sheet(cx, sheet_bounds, camera),
                PenStyle::Selector => self.selector.draw_on_sheet(cx, sheet_bounds, camera),
                PenStyle::Tools => self.tools.draw_on_sheet(cx, sheet_bounds, camera),
            }
        } else {
            Ok(())
        }
    }
}
