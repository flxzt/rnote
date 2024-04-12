// Imports
use super::PenStyle;
use rnote_compose::penevent::ShortcutKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[repr(u32)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "shortcut_mode")]
pub enum ShortcutMode {
    #[serde(rename = "temporary")]
    Temporary,
    #[serde(rename = "permanent")]
    Permanent,
    #[serde(rename = "toggle")]
    Toggle,
    #[serde(rename = "disabled")]
    Disabled,
}

impl TryFrom<u32> for ShortcutMode {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("ShortcutMode try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename = "shortcut_action")]
pub enum ShortcutAction {
    #[serde(rename = "change_pen_style")]
    ChangePenStyle {
        #[serde(rename = "style")]
        style: PenStyle,
        #[serde(rename = "mode")]
        mode: ShortcutMode,
    },
}

/// The registered shortcut actions for the given shortcut keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shortcuts")]
pub struct Shortcuts(HashMap<ShortcutKey, ShortcutAction>);

impl Default for Shortcuts {
    fn default() -> Self {
        let mut map = HashMap::<ShortcutKey, ShortcutAction>::default();
        map.insert(
            ShortcutKey::StylusPrimaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Eraser,
                mode: ShortcutMode::Temporary,
            },
        );
        map.insert(
            ShortcutKey::StylusSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Selector,
                mode: ShortcutMode::Temporary,
            },
        );
        map.insert(
            ShortcutKey::MouseSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Shaper,
                mode: ShortcutMode::Temporary,
            },
        );
        map.insert(
            ShortcutKey::TouchTwoFingerLongPress,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Eraser,
                mode: ShortcutMode::Toggle,
            },
        );
        map.insert(
            ShortcutKey::KeyboardCtrlSpace,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Tools,
                mode: ShortcutMode::Toggle,
            },
        );
        map.insert(
            ShortcutKey::DrawingPadButton0,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Brush,
                mode: ShortcutMode::Permanent,
            },
        );
        map.insert(
            ShortcutKey::DrawingPadButton1,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Shaper,
                mode: ShortcutMode::Permanent,
            },
        );
        map.insert(
            ShortcutKey::DrawingPadButton2,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Typewriter,
                mode: ShortcutMode::Permanent,
            },
        );
        map.insert(
            ShortcutKey::DrawingPadButton3,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Eraser,
                mode: ShortcutMode::Permanent,
            },
        );

        Self(map)
    }
}

impl Deref for Shortcuts {
    type Target = HashMap<ShortcutKey, ShortcutAction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Shortcuts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
