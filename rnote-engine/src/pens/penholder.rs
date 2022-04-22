use crate::pens::shortcuts::ShortcutAction;
use crate::pens::Tools;

use crate::sheet::Sheet;
use crate::surfaceflags::SurfaceFlags;
use crate::{Camera, DrawOnSheetBehaviour, StrokeStore};
use piet::RenderContext;
use rnote_compose::penhelpers::{PenEvent, ShortcutKey};

use gtk4::{glib, glib::prelude::*};
use num_derive::FromPrimitive;
use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

use super::penbehaviour::PenProgress;
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

#[derive(Debug, Clone)]
pub enum PenHolderEvent {
    PenEvent(PenEvent),
    ChangeStyle(PenStyle),
    ChangeStyleOverride(Option<PenStyle>),
    PressedShortcutkey(ShortcutKey),
}

/// This holds the pens and is the main interaction point when changing the pen style / emitting pen events.
#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
#[serde(default, rename = "penholder")]
pub struct PenHolder {
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

    #[serde(rename = "style")]
    style: PenStyle,
    #[serde(skip)]
    style_override: Option<PenStyle>,
    #[serde(rename = "shortcuts")]
    shortcuts: Shortcuts,
    #[serde(rename = "pen_sounds")]
    // we need this outside of the audioplayer, because we skip (de) serializing it.
    pen_sounds: bool,
    #[serde(skip)]
    audioplayer: Option<AudioPlayer>,
}

impl Default for PenHolder {
    fn default() -> Self {
        let pen_sounds = false;
        let audioplayer = AudioPlayer::new()
            .map_err(|e| {
                log::error!(
                    "failed to create a new audio player in PenHolder::default(), Err {}",
                    e
                );
            })
            .map(|mut audioplayer| {
                audioplayer.enabled = pen_sounds;
                audioplayer
            })
            .ok();

        Self {
            brush: Brush::default(),
            shaper: Shaper::default(),
            eraser: Eraser::default(),
            selector: Selector::default(),
            tools: Tools::default(),

            style: PenStyle::default(),
            style_override: None,
            shortcuts: Shortcuts::default(),
            pen_sounds,
            audioplayer,
        }
    }
}

impl PenHolder {
    /// Use this to import and overwrite self (e.g. when loading from settings)
    pub fn import(&mut self, penholder: Self) {
        *self = penholder;
        // Set the pen sounds to update the audioplayer
        self.set_pen_sounds(self.pen_sounds)
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

    pub fn pen_sounds(&self) -> bool {
        self.pen_sounds
    }

    pub fn set_pen_sounds(&mut self, pen_sounds: bool) {
        self.pen_sounds = pen_sounds;
        if let Some(audioplayer) = self.audioplayer.as_mut() {
            audioplayer.enabled = pen_sounds;
        }
    }

    /// Changes the internal state according to events
    #[must_use]
    pub(crate) fn handle_penholder_event(
        &mut self,
        event: PenHolderEvent,
        sheet: &mut crate::sheet::Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
    ) -> SurfaceFlags {
/* 
        log::debug!(
            "handle_event(), event: {:?}, style: {:?}, style_override: {:?}",
            event,
            self.style,
            self.style_override
        );
 */

        let mut surface_flags = SurfaceFlags::default();
        surface_flags.redraw = true;

        match event {
            PenHolderEvent::PenEvent(pen_event) => {
                let (pen_progress, pen_surface_flags) =
                    self.handle_pen_event(pen_event, sheet, store, camera);
                surface_flags.merge_with_other(pen_surface_flags);
                surface_flags.merge_with_other(self.handle_pen_progress(pen_progress));
            }
            PenHolderEvent::ChangeStyle(new_style) => {
                surface_flags.merge_with_other(self.change_style(new_style, sheet, store, camera));
            }
            PenHolderEvent::ChangeStyleOverride(new_style_override) => {
                surface_flags.merge_with_other(self.change_style_override(
                    new_style_override,
                    sheet,
                    store,
                    camera,
                ));
            }
            PenHolderEvent::PressedShortcutkey(shortcut_key) => {
                if let Some(&action) = self.shortcuts.get(&shortcut_key) {
                    match action {
                        ShortcutAction::ChangePenStyle {
                            style: new_style,
                            permanent,
                        } => {
                            if permanent {
                                surface_flags.merge_with_other(
                                    self.change_style(new_style, sheet, store, camera),
                                );
                            } else {
                                surface_flags.merge_with_other(self.change_style_override(
                                    Some(new_style),
                                    sheet,
                                    store,
                                    camera,
                                ));
                            }
                        }
                    }
                }
            }
        }

        surface_flags
    }

    #[must_use]
    fn change_style(
        &mut self,
        new_style: PenStyle,
        sheet: &mut crate::sheet::Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        if self.style != new_style {
            // Cancel current pen
            let (pen_progress, pen_surface_flags) =
                self.handle_pen_event(PenEvent::Cancel, sheet, store, camera);
            surface_flags.merge_with_other(pen_surface_flags);

            surface_flags.merge_with_other(self.handle_pen_progress(pen_progress));

            // Deselecting when changing the style
            let all_strokes = store.keys_sorted_chrono();
            store.set_selected_keys(&all_strokes, false);

            self.style = new_style;

            surface_flags.penholder_changed = true;
            surface_flags.redraw = true;
        }

        surface_flags
    }

    #[must_use]
    fn change_style_override(
        &mut self,
        new_style_override: Option<PenStyle>,
        sheet: &mut crate::sheet::Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
    ) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        //log::debug!("current_style_override: {:?}, new_style_override: {:?}", self.style_override, new_style_override);

        if self.style_override != new_style_override {
            // Cancel current pen
            let (pen_progress, pen_surface_flags) =
                self.handle_pen_event(PenEvent::Cancel, sheet, store, camera);
            surface_flags.merge_with_other(pen_surface_flags);
            surface_flags.merge_with_other(self.handle_pen_progress(pen_progress));

            // Deselecting when changing the style override
            let all_strokes = store.keys_sorted_chrono();
            store.set_selected_keys(&all_strokes, false);

            self.style_override = new_style_override;

            surface_flags.penholder_changed = true;
            surface_flags.redraw = true;
        }

        surface_flags
    }

    #[must_use]
    fn handle_pen_progress(&mut self, pen_progress: PenProgress) -> SurfaceFlags {
        let mut surface_flags = SurfaceFlags::default();

        match pen_progress {
            PenProgress::Idle => {}
            PenProgress::InProgress => {}
            PenProgress::Finished => {
                // Disable the style override when pen is finished
                if self.style_override.take().is_some() {
                    surface_flags.penholder_changed = true;
                }
            }
        }

        surface_flags
    }

    fn handle_pen_event(
        &mut self,
        event: PenEvent,
        sheet: &mut Sheet,
        store: &mut StrokeStore,
        camera: &mut Camera,
    ) -> (PenProgress, SurfaceFlags) {
        match self.style_w_override() {
            PenStyle::Brush => {
                self.brush
                    .handle_event(event, sheet, store, camera, self.audioplayer.as_mut())
            }
            PenStyle::Shaper => {
                self.shaper
                    .handle_event(event, sheet, store, camera, self.audioplayer.as_mut())
            }
            PenStyle::Eraser => {
                self.eraser
                    .handle_event(event, sheet, store, camera, self.audioplayer.as_mut())
            }
            PenStyle::Selector => {
                self.selector
                    .handle_event(event, sheet, store, camera, self.audioplayer.as_mut())
            }
            PenStyle::Tools => {
                self.tools
                    .handle_event(event, sheet, store, camera, self.audioplayer.as_mut())
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
        cx: &mut piet_cairo::CairoRenderContext,
        sheet_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

        match self.style_w_override() {
            PenStyle::Brush => self.brush.draw_on_sheet(cx, sheet_bounds, camera),
            PenStyle::Shaper => self.shaper.draw_on_sheet(cx, sheet_bounds, camera),
            PenStyle::Eraser => self.eraser.draw_on_sheet(cx, sheet_bounds, camera),
            PenStyle::Selector => self.selector.draw_on_sheet(cx, sheet_bounds, camera),
            PenStyle::Tools => self.tools.draw_on_sheet(cx, sheet_bounds, camera),
        }?;

        cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}
