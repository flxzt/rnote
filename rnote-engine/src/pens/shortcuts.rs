use super::PenStyle;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use rnote_compose::penevents::ShortcutKey;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "shortcut_action")]
pub enum ShortcutAction {
    #[serde(rename = "change_pen_style")]
    ChangePenStyle {
        #[serde(rename = "style")]
        style: PenStyle,
        #[serde(rename = "permanent")]
        permanent: bool,
    },
}

/// holds the registered shortcut actions for the given shortcut keys
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shortcuts")]
pub struct Shortcuts(HashMap<ShortcutKey, ShortcutAction>);

impl Default for Shortcuts {
    fn default() -> Self {
        let mut map = HashMap::<ShortcutKey, ShortcutAction>::default();
        map.insert(
            ShortcutKey::StylusPrimaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Selector,
                permanent: false,
            },
        );
        map.insert(
            ShortcutKey::StylusSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Selector,
                permanent: false,
            },
        );
        map.insert(
            ShortcutKey::MouseSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Shaper,
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
