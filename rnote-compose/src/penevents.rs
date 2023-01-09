use crate::penpath::Element;
use serde::{Deserialize, Serialize};

/// Represents a Pen Event. Note that there is no "motion" event, because we want the events to be entirely stateless.
/// Motion event already encode state as they would only be valid if they are preceded by down events.
/// As a result, multiple down events are emitted if the pen is pressed down and drawing. This should be handled accordingly by the state machines which receives the events.
#[derive(Debug, Clone)]
pub enum PenEvent {
    /// A pen down event. Is repeatedly emitted while the pen is pressed and moved
    Down {
        /// The element for the down event
        element: Element,
        /// pressed shortcut keys pressed during the down event
        shortcut_keys: Vec<ShortcutKey>,
    },
    /// A pen up event.
    Up {
        /// The element for the up event
        element: Element,
        /// pressed shortcut keys pressed during the up event
        shortcut_keys: Vec<ShortcutKey>,
    },
    /// A pen down event. Is repeatedly emitted while the pen is in proximity and moved
    Proximity {
        /// The element for the proximity event
        element: Element,
        /// pressed shortcut keys during the proximity event
        shortcut_keys: Vec<ShortcutKey>,
    },
    /// A keyboard key pressed event
    KeyPressed {
        /// the key
        keyboard_key: KeyboardKey,
        /// pressed shortcut keys during the keyboard key event
        shortcut_keys: Vec<ShortcutKey>,
    },
    /// Text input
    Text {
        /// The committed text
        text: String,
    },
    /// event when the pen vanishes unexpected. should reset all pending actions and state
    Cancel,
}

/// A key on the keyboard
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyboardKey {
    /// a unicode character. Expects that control characters are already converted and filtered out with the method `filter_convert_unicode_control_chars`
    Unicode(char),
    /// backspace
    BackSpace,
    /// Tab
    HorizontalTab,
    /// Line feed
    Linefeed,
    /// Carriage return
    CarriageReturn,
    /// Escape
    Escape,
    /// delete
    Delete,
    /// Arrow up
    NavUp,
    /// Arrow down
    NavDown,
    /// Arrow left
    NavLeft,
    /// Arrow right
    NavRight,
    /// Shift left
    ShiftLeft,
    /// Shift right
    ShiftRight,
    /// Ctrl left
    CtrlLeft,
    /// Ctrl right
    CtrlRight,
    /// Unsupported
    Unsupported,
}

impl KeyboardKey {
    /// Filters and converts unicode control characters to a fitting variant, or `Unsupported`
    pub fn filter_convert_unicode_control_chars(self) -> Self {
        match self {
            key @ Self::Unicode(keychar) => {
                if keychar.is_control() {
                    match keychar as u32 {
                        0x08 => Self::BackSpace,
                        0x09 => Self::HorizontalTab,
                        0x0a => Self::Linefeed,
                        0x0d => Self::CarriageReturn,
                        0x1b => Self::Escape,
                        0x7f => Self::Delete,
                        _ => Self::Unsupported,
                    }
                } else {
                    key
                }
            }
            other => other,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "shortcut_key")]
/// A input shortcut key (could also be named modifier key)
pub enum ShortcutKey {
    /// the primary button of the stylus
    #[serde(rename = "stylus_primary_button")]
    StylusPrimaryButton,
    /// the secondary button of the stylus
    #[serde(rename = "stylus_secondary_button")]
    StylusSecondaryButton,
    /// the secondary mouse button, usually right click
    #[serde(rename = "mouse_secondary_button")]
    MouseSecondaryButton,
    /// Shift
    KeyboardShift,
    /// Ctrl
    KeyboardCtrl,
    /// Alt
    KeyboardAlt,
}

/// The current pen state. Used wherever the we have internal state
#[derive(Debug, Clone, Copy)]
pub enum PenState {
    /// Up
    Up,
    /// Proximity
    Proximity,
    /// Down
    Down,
}
