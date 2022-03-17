use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use super::PenStyle;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    ChangePenStyle { style: PenStyle, permanent: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShortcutKey {
    KeyboardKey(String),
    StylusPrimaryButton,
    StylusSecondaryButton,
    StylusEraserButton,
    MouseSecondaryButton,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcuts(HashMap<ShortcutKey, ShortcutAction>);

impl Default for Shortcuts {
    fn default() -> Self {
        let mut map = HashMap::<ShortcutKey, ShortcutAction>::default();
        map.insert(
            ShortcutKey::StylusPrimaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::SelectorStyle,
                permanent: false,
            },
        );
        map.insert(
            ShortcutKey::StylusSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::SelectorStyle,
                permanent: false,
            },
        );
        map.insert(
            ShortcutKey::StylusEraserButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::EraserStyle,
                permanent: false,
            },
        );
        map.insert(
            ShortcutKey::MouseSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::ShaperStyle,
                permanent: false,
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
