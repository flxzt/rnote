use crate::penpath::Element;
use serde::{Deserialize, Serialize};

/// Represents a Pen Event. Note that there is no "motion" event, because we want the events to be entirely stateless.
/// Motion event already encode state as they would only be valid if they are preceded by down events.
/// As a result, multiple down events are emitted if the pen is pressed down and drawing. This should be handled accordingly by the state machines which receives the events.
#[derive(Debug, Clone, Copy)]
pub enum PenEvent {
    Down {
        element: Element,
        shortcut_key: Option<ShortcutKey>,
    },
    Up {
        element: Element,
        shortcut_key: Option<ShortcutKey>,
    },
    Proximity {
        element: Element,
        shortcut_key: Option<ShortcutKey>,
    },
    Cancel,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "shortcut_key")]
pub enum ShortcutKey {
    #[serde(rename = "keyboard_key")]
    KeyboardKey(char),
    #[serde(rename = "stylus_primary_button")]
    StylusPrimaryButton,
    #[serde(rename = "stylus_secondary_button")]
    StylusSecondaryButton,
    #[serde(rename = "stylus_eraser_mode")]
    StylusEraserMode,
    #[serde(rename = "mouse_secondary_button")]
    MouseSecondaryButton,
}
