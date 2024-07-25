// Imports
use crate::penpath::Element;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A Pen Event.
///
/// Note that there is no "motion" event, because we want the events to be entirely stateless.
/// Motion event already encode state as they would only be valid if they are preceded by a down event.
/// As a result, multiple down events are emitted while the pen is pressed down and being moved.
/// This should be handled accordingly by the state machines which receive the events.
#[derive(Debug, Clone)]
pub enum PenEvent {
    /// A pen down event. Is repeatedly emitted while the pen is pressed down and moved.
    Down {
        /// The element for the down event.
        element: Element,
        /// Modifier keys pressed during the event.
        modifier_keys: HashSet<ModifierKey>,
    },
    /// A pen up event.
    Up {
        /// The element for the up event.
        element: Element,
        /// Modifier keys pressed during the event.
        modifier_keys: HashSet<ModifierKey>,
    },
    /// A pen down event. Is repeatedly emitted while the pen is in proximity and moved.
    Proximity {
        /// The element for the proximity event.
        element: Element,
        /// Modifier keys pressed during the event.
        modifier_keys: HashSet<ModifierKey>,
    },
    /// A keyboard key pressed event.
    KeyPressed {
        /// the key
        keyboard_key: KeyboardKey,
        /// Modifier keys pressed during the event.
        modifier_keys: HashSet<ModifierKey>,
    },
    /// Text input event.
    Text {
        /// The committed text.
        text: String,
    },
    /// Cancel event when the pen vanishes unexpected.
    ///
    /// Should finish all current actions and reset all state.
    Cancel,
}

/// A key on the keyboard.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyboardKey {
    /// A Unicode character.
    ///
    /// Expects that control characters are already converted and filtered out with the method [KeyboardKey::filter_convert_unicode_control_chars].
    Unicode(char),
    /// Backspace.
    BackSpace,
    /// Tab.
    HorizontalTab,
    /// Line feed.
    Linefeed,
    /// Carriage return.
    CarriageReturn,
    /// Escape.
    Escape,
    /// Delete.
    Delete,
    /// Arrow up.
    NavUp,
    /// Arrow down.
    NavDown,
    /// Arrow left.
    NavLeft,
    /// Arrow right.
    NavRight,
    /// Shift left.
    ShiftLeft,
    /// Shift right.
    ShiftRight,
    /// Ctrl left.
    CtrlLeft,
    /// Ctrl right.
    CtrlRight,
    /// Home.
    Home,
    /// End.
    End,
    /// Unsupported Key.
    Unsupported,
}

impl KeyboardKey {
    /// Filter and convert unicode control characters to a fitting variant, or if unsupported [KeyboardKey::Unsupported].
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "shortcut_key")]
/// A Shortcut key.
pub enum ShortcutKey {
    /// Primary button of the stylus.
    #[serde(rename = "stylus_primary_button")]
    StylusPrimaryButton,
    /// Secondary button of the stylus.
    #[serde(rename = "stylus_secondary_button")]
    StylusSecondaryButton,
    /// Secondary mouse button.
    #[serde(rename = "mouse_secondary_button")]
    MouseSecondaryButton,
    /// Touch two finger long press gesture.
    #[serde(rename = "touch_two_finger_long_press")]
    TouchTwoFingerLongPress,
    /// Keyboard Shift plus Spacebar shortcut.
    #[serde(rename = "keyboard_ctrl_space")]
    KeyboardCtrlSpace,
    /// Button 0 on a drawing pad.
    #[serde(rename = "drawing_pad_button_0")]
    DrawingPadButton0,
    /// Button 1 on a drawing pad.
    #[serde(rename = "drawing_pad_button_1")]
    DrawingPadButton1,
    /// Button 2 on a drawing pad.
    #[serde(rename = "drawing_pad_button_2")]
    DrawingPadButton2,
    /// Button 3 on a drawing pad.
    #[serde(rename = "drawing_pad_button_3")]
    DrawingPadButton3,
}

/// A modifier key.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename = "modifier_key")]
pub enum ModifierKey {
    /// Shift.
    #[serde(rename = "keyboard_shift")]
    KeyboardShift,
    /// Ctrl.
    #[serde(rename = "keyboard_ctrl")]
    KeyboardCtrl,
    /// Alt.
    #[serde(rename = "keyboard_alt")]
    KeyboardAlt,
}

/// The current pen state. Used wherever there is internal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenState {
    /// Up.
    Up,
    /// Proximity.
    Proximity,
    /// Down.
    Down,
}

/// The pen progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenProgress {
    /// In idle state.
    Idle,
    /// In progress state.
    InProgress,
    /// Pen is finished.
    Finished,
}
