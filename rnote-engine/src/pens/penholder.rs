use std::time::Instant;

use crate::engine::{EngineView, EngineViewMut};
use crate::pens::shortcuts::ShortcutAction;
use crate::pens::Tools;

use crate::widgetflags::WidgetFlags;
use crate::DrawOnDocBehaviour;
use piet::RenderContext;
use rnote_compose::penevents::{PenEvent, ShortcutKey};

use gtk4::{glib, glib::prelude::*};
use p2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenProgress;
use super::penmode::PenModeState;
use super::{Brush, Eraser, PenBehaviour, PenMode, Selector, Shaper, Shortcuts, Typewriter};

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
    num_derive::FromPrimitive,
)]
#[repr(u32)]
#[enum_type(name = "PenStyle")]
#[serde(rename = "pen_style")]
pub enum PenStyle {
    #[enum_value(name = "Brush", nick = "brush")]
    #[serde(rename = "brush")]
    Brush = 0,
    #[enum_value(name = "Shaper", nick = "shaper")]
    #[serde(rename = "shaper")]
    Shaper,
    #[enum_value(name = "Typewriter", nick = "typewriter")]
    #[serde(rename = "typewriter")]
    Typewriter,
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
        num_traits::FromPrimitive::from_u32(value)
            .ok_or_else(|| anyhow::anyhow!("PenStyle try_from::<u32>() for value {} failed", value))
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
            Self::Typewriter => String::from("pen-typewriter-symbolic"),
            Self::Eraser => String::from("pen-eraser-symbolic"),
            Self::Selector => String::from("pen-selector-symbolic"),
            Self::Tools => String::from("pen-tools-symbolic"),
        }
    }
}

/// This holds the pens and related state and handles pen events.
#[allow(missing_debug_implementations)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "penholder")]
pub struct PenHolder {
    #[serde(rename = "brush")]
    pub brush: Brush,
    #[serde(rename = "shaper")]
    pub shaper: Shaper,
    #[serde(rename = "typewriter")]
    pub typewriter: Typewriter,
    #[serde(rename = "eraser")]
    pub eraser: Eraser,
    #[serde(rename = "selector")]
    pub selector: Selector,
    #[serde(rename = "tools")]
    pub tools: Tools,
    #[serde(rename = "pen_mode_state")]
    pen_mode_state: PenModeState,
    #[serde(rename = "shortcuts")]
    shortcuts: Shortcuts,

    #[serde(skip)]
    pen_progress: PenProgress,
}

impl Default for PenHolder {
    fn default() -> Self {
        Self {
            brush: Brush::default(),
            shaper: Shaper::default(),
            eraser: Eraser::default(),
            selector: Selector::default(),
            typewriter: Typewriter::default(),
            tools: Tools::default(),
            pen_mode_state: PenModeState::default(),
            shortcuts: Shortcuts::default(),

            pen_progress: PenProgress::Idle,
        }
    }
}

impl PenHolder {
    /// Registers a new shortcut key and action
    pub fn register_new_shortcut(&mut self, key: ShortcutKey, action: ShortcutAction) {
        self.shortcuts.insert(key, action);
    }

    /// Removes the shortcut action for the given shortcut key, if it is registered
    pub fn remove_shortcut(&mut self, key: ShortcutKey) -> Option<ShortcutAction> {
        self.shortcuts.remove(&key)
    }

    // Gets the current registered action the the given shortcut key
    pub fn get_shortcut_action(&self, key: ShortcutKey) -> Option<ShortcutAction> {
        self.shortcuts.get(&key).cloned()
    }

    /// Lists all current registered shortcut keys and their action
    pub fn list_current_shortcuts(&self) -> Vec<(ShortcutKey, ShortcutAction)> {
        self.shortcuts
            .iter()
            .map(|(key, action)| (*key, *action))
            .collect()
    }

    /// Gets the current style, or the override if it is set.
    pub fn current_style_w_override(&self) -> PenStyle {
        self.pen_mode_state.current_style_w_override()
    }

    /// forces a new style without triggering any side effects
    pub fn force_style_without_sideeffects(&mut self, style: PenStyle) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        self.pen_mode_state.set_style_all_modes(style);

        widget_flags.merge_with_other(self.handle_changed_pen_style());

        widget_flags
    }

    /// forces a new style override without triggering any side effects
    pub fn force_style_override_without_sideeffects(
        &mut self,
        style_override: Option<PenStyle>,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        self.pen_mode_state.set_style_override(style_override);

        widget_flags.merge_with_other(self.handle_changed_pen_style());

        widget_flags
    }

    /// the current pen progress
    pub fn current_pen_progress(&self) -> PenProgress {
        self.pen_progress
    }

    /// change the pen style
    pub fn change_style(
        &mut self,
        new_style: PenStyle,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if self.pen_mode_state.style() != new_style {
            // Cancel current pen
            widget_flags.merge_with_other(self.handle_pen_event(
                PenEvent::Cancel,
                None,
                now,
                engine_view,
            ));

            // Deselecting when changing the style
            let all_strokes = engine_view.store.selection_keys_as_rendered();
            engine_view.store.set_selected_keys(&all_strokes, false);

            self.pen_mode_state.set_style(new_style);

            widget_flags.merge_with_other(self.handle_changed_pen_style());
        }

        widget_flags
    }

    /// change the style override
    pub fn change_style_override(
        &mut self,
        new_style_override: Option<PenStyle>,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        //log::debug!("current_style_override: {:?}, new_style_override: {:?}", self.style_override, new_style_override);

        if self.pen_mode_state.style_override() != new_style_override {
            // Cancel current pen
            widget_flags.merge_with_other(self.handle_pen_event(
                PenEvent::Cancel,
                None,
                now,
                engine_view,
            ));

            // Deselecting when changing the style override
            let all_strokes = engine_view.store.selection_keys_as_rendered();
            engine_view.store.set_selected_keys(&all_strokes, false);

            self.pen_mode_state.set_style_override(new_style_override);

            widget_flags.merge_with_other(self.handle_changed_pen_style());
        }

        widget_flags
    }

    /// change the pen mode (pen, eraser, etc.). Relevant for stylus input
    pub fn change_pen_mode(
        &mut self,
        pen_mode: PenMode,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if self.pen_mode_state.pen_mode() != pen_mode {
            widget_flags.merge_with_other(self.handle_pen_event(
                PenEvent::Cancel,
                None,
                now,
                engine_view,
            ));
            self.pen_mode_state.set_pen_mode(pen_mode);

            widget_flags.merge_with_other(self.handle_changed_pen_style());
        }

        widget_flags
    }

    /// Handle a pen event
    pub fn handle_pen_event(
        &mut self,
        event: PenEvent,
        pen_mode: Option<PenMode>,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        widget_flags.redraw = true;

        if let Some(pen_mode) = pen_mode {
            widget_flags.merge_with_other(self.change_pen_mode(pen_mode, now, engine_view));
        }

        /*
               log::debug!(
                   "handle_pen_event(), event: {:?}, pen_mode_state: {:?}",
                   event,
                   self.pen_mode_state,
               );
        */

        // First we handle certain pointer shortcut keys
        // TODO: handle this better
        match &event {
            PenEvent::Down { shortcut_keys, .. }
            | PenEvent::Up { shortcut_keys, .. }
            | PenEvent::Proximity { shortcut_keys, .. } => {
                if shortcut_keys.contains(&ShortcutKey::MouseSecondaryButton) {
                    widget_flags.merge_with_other(self.handle_pressed_shortcut_key(
                        ShortcutKey::MouseSecondaryButton,
                        now,
                        engine_view,
                    ));
                }
            }
            PenEvent::KeyPressed { .. } => {}
            PenEvent::Text { .. } => {}
            PenEvent::Cancel => {}
        }

        // Handle the events with the current pen
        let (pen_progress, other_widget_flags) = match self.current_style_w_override() {
            PenStyle::Brush => self.brush.handle_event(event, now, engine_view),
            PenStyle::Shaper => self.shaper.handle_event(event, now, engine_view),
            PenStyle::Typewriter => self.typewriter.handle_event(event, now, engine_view),
            PenStyle::Eraser => self.eraser.handle_event(event, now, engine_view),
            PenStyle::Selector => self.selector.handle_event(event, now, engine_view),
            PenStyle::Tools => self.tools.handle_event(event, now, engine_view),
        };

        widget_flags.merge_with_other(other_widget_flags);

        widget_flags.merge_with_other(self.handle_pen_progress(pen_progress));

        widget_flags
    }

    fn handle_pen_progress(&mut self, pen_progress: PenProgress) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        match pen_progress {
            PenProgress::Idle => {}
            PenProgress::InProgress => {}
            PenProgress::Finished => {
                // take the style override when pen is finished
                if self.pen_mode_state.take_style_override().is_some() {
                    widget_flags.refresh_ui = true;
                }
            }
        }

        self.pen_progress = pen_progress;

        widget_flags
    }

    pub fn handle_changed_pen_style(&mut self) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        match self.pen_mode_state.current_style_w_override() {
            PenStyle::Typewriter => {
                // Enable text preprocessing for typewriter
                widget_flags.enable_text_preprocessing = Some(true);
            }
            _ => {
                widget_flags.enable_text_preprocessing = Some(true);
            }
        }
        widget_flags.redraw = true;
        widget_flags.refresh_ui = true;

        widget_flags
    }

    /// Handle a pressed shortcut key
    pub fn handle_pressed_shortcut_key(
        &mut self,
        shortcut_key: ShortcutKey,
        now: Instant,
        engine_view: &mut EngineViewMut,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        if let Some(action) = self.get_shortcut_action(shortcut_key) {
            match action {
                ShortcutAction::ChangePenStyle {
                    style: new_style,
                    permanent,
                } => {
                    if permanent {
                        widget_flags.merge_with_other(self.change_style(
                            new_style,
                            now,
                            engine_view,
                        ));
                    } else {
                        widget_flags.merge_with_other(self.change_style_override(
                            Some(new_style),
                            now,
                            engine_view,
                        ));
                    }
                }
            }
        }

        widget_flags
    }

    /// fetches clipboard content from the current pen
    #[allow(clippy::type_complexity)]
    pub fn fetch_clipboard_content(
        &self,
        engine_view: &EngineView,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        match self.current_style_w_override() {
            PenStyle::Brush => self.brush.fetch_clipboard_content(engine_view),
            PenStyle::Shaper => self.shaper.fetch_clipboard_content(engine_view),
            PenStyle::Typewriter => self.typewriter.fetch_clipboard_content(engine_view),
            PenStyle::Eraser => self.eraser.fetch_clipboard_content(engine_view),
            PenStyle::Selector => self.selector.fetch_clipboard_content(engine_view),
            PenStyle::Tools => self.tools.fetch_clipboard_content(engine_view),
        }
    }

    /// cuts clipboard content from the current pen
    #[allow(clippy::type_complexity)]
    pub fn cut_clipboard_content(
        &mut self,
        engine_view: &mut EngineViewMut,
    ) -> anyhow::Result<(Option<(Vec<u8>, String)>, WidgetFlags)> {
        match self.current_style_w_override() {
            PenStyle::Brush => self.brush.cut_clipboard_content(engine_view),
            PenStyle::Shaper => self.shaper.cut_clipboard_content(engine_view),
            PenStyle::Typewriter => self.typewriter.cut_clipboard_content(engine_view),
            PenStyle::Eraser => self.eraser.cut_clipboard_content(engine_view),
            PenStyle::Selector => self.selector.cut_clipboard_content(engine_view),
            PenStyle::Tools => self.tools.cut_clipboard_content(engine_view),
        }
    }

    // Updates the penholder and pens internal state
    pub fn update_internal_state(&mut self, engine_view: &EngineView) {
        self.brush.update_internal_state(engine_view);
        self.shaper.update_internal_state(engine_view);
        self.typewriter.update_internal_state(engine_view);
        self.eraser.update_internal_state(engine_view);
        self.selector.update_internal_state(engine_view);
        self.tools.update_internal_state(engine_view);
    }
}

impl DrawOnDocBehaviour for PenHolder {
    fn bounds_on_doc(&self, engine_view: &EngineView) -> Option<Aabb> {
        match self.current_style_w_override() {
            PenStyle::Brush => self.brush.bounds_on_doc(engine_view),
            PenStyle::Shaper => self.shaper.bounds_on_doc(engine_view),
            PenStyle::Typewriter => self.typewriter.bounds_on_doc(engine_view),
            PenStyle::Eraser => self.eraser.bounds_on_doc(engine_view),
            PenStyle::Selector => self.selector.bounds_on_doc(engine_view),
            PenStyle::Tools => self.tools.bounds_on_doc(engine_view),
        }
    }
    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        match self.current_style_w_override() {
            PenStyle::Brush => self.brush.draw_on_doc(cx, engine_view),
            PenStyle::Shaper => self.shaper.draw_on_doc(cx, engine_view),
            PenStyle::Typewriter => self.typewriter.draw_on_doc(cx, engine_view),
            PenStyle::Eraser => self.eraser.draw_on_doc(cx, engine_view),
            PenStyle::Selector => self.selector.draw_on_doc(cx, engine_view),
            PenStyle::Tools => self.tools.draw_on_doc(cx, engine_view),
        }?;

        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }
}
