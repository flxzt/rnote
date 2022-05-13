use crate::penpath::Element;
use serde::{Deserialize, Serialize};

/// Represents a Pen Event. Note that there is no "motion" event, because we want the events to be entirely stateless.
/// Motion event already encode state as they would only be valid if they are preceded by down events.
/// As a result, multiple down events are emitted if the pen is pressed down and drawing. This should be handled accordingly by the state machines which receives the events.
#[derive(Debug, Clone)]
pub enum PenEvent {
    /// A pen down event. Is repeatetly emitted while the pen is pressed and moved
    Down {
        /// The element for the down event
        element: Element,
        /// wether a shortcut key is pressed during the down event
        shortcut_keys: Vec<ShortcutKey>,
    },
    /// A pen up event.
    Up {
        /// The element for the down event
        element: Element,
        /// wether a shortcut key is pressed during the up event
        shortcut_keys: Vec<ShortcutKey>,
    },
    /// A pen down event. Is repeatetly emitted while the pen is in proximity and moved
    Proximity {
        /// The element for the proximity event
        element: Element,
        /// wether a shortcut key is pressed during the proximity event
        shortcut_keys: Vec<ShortcutKey>,
    },
    /// event when the pen vanishes unexpected. should reset all pending actions and state
    Cancel,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "shortcut_key")]
/// A input shortcut key
pub enum ShortcutKey {
    /// A keyboard key
    #[serde(rename = "keyboard_key")]
    KeyboardKey(char),
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
