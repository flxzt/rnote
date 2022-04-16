use crate::penpath::Element;
use serde::{Deserialize, Serialize};

/// Represents a Pen Event. Note that there is no "motion" event, because we want the events to be entirely stateless.
/// Motion event already encode state as they would only be valid if they are preceded by down events.
/// As a result, multiple down events are emitted if the pen is pressed down and drawing. This should be handled accordingly by the state machines which receives the events.
#[derive(Debug, Clone, Copy)]
pub enum PenEvent {
    /// A pen down event. Is repeatetly emitted while the pen is pressed and moved
    Down {
        /// The element for the down event
        element: Element,
        /// wether a shortcut key is pressed during the down event
        shortcut_key: Option<ShortcutKey>,
    },
    /// A pen up event.
    Up {
        /// The element for the down event
        element: Element,
        /// wether a shortcut key is pressed during the up event
        shortcut_key: Option<ShortcutKey>,
    },
    /// A pen down event. Is repeatetly emitted while the pen is in proximity and moved
    Proximity {
        /// The element for the proximity event
        element: Element,
        /// wether a shortcut key is pressed during the proximity event
        shortcut_key: Option<ShortcutKey>,
    },
    /// event when the pen vanishes unexpected. should reset all pending actions and state
    Cancel,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "shortcut_key")]
/// A input shortcut key
pub enum ShortcutKey {
    #[serde(rename = "keyboard_key")]
    /// A keyboard key
    KeyboardKey(char),
    #[serde(rename = "stylus_primary_button")]
    /// the primary button of the stylus
    StylusPrimaryButton,
    #[serde(rename = "stylus_secondary_button")]
    /// the secondary button of the stylus
    StylusSecondaryButton,
    #[serde(rename = "stylus_eraser_mode")]
    /// Pen is in eraser mode ( either a button, or on some pens the back side )
    StylusEraserMode,
    #[serde(rename = "mouse_secondary_button")]
    /// the secondary mouse button, usually right click
    MouseSecondaryButton,
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
