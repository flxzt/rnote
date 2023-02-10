use gtk4::{gdk, prelude::*, Inhibit};
use rnote_compose::penevents::ShortcutKey;
use rnote_compose::penevents::{KeyboardKey, PenState};
use rnote_compose::penevents::{ModifierKey, PenEvent};
use rnote_compose::penpath::Element;
use rnote_engine::pens::PenMode;
use rnote_engine::WidgetFlags;
use std::time::Instant;

use super::RnCanvas;

// Returns whether the event should be inhibited from propagating, and the new pen state
pub(crate) fn handle_pointer_controller_event(
    canvas: &RnCanvas,
    event: &gdk::Event,
    mut state: PenState,
) -> (Inhibit, PenState) {
    //std::thread::sleep(std::time::Duration::from_millis(100));
    let touch_drawing = canvas.touch_drawing();
    let event_type = event.event_type();

    //super::input::debug_gdk_event(event);

    if reject_pointer_input(event, touch_drawing) {
        return (Inhibit(false), state);
    }

    let now = Instant::now();
    let mut widget_flags = WidgetFlags::default();
    let modifiers = event.modifier_state();
    let _input_source = event.device().unwrap().source();
    let is_stylus = event_is_stylus(event);
    let mut handle_pen_event = false;
    let mut inhibit = false;

    match event_type {
        gdk::EventType::MotionNotify => {
            if is_stylus {
                handle_pen_event = true;
                inhibit = true;

                // As in gtk4 'gesturestylus.c:120' proximity with stylus is also detected in this way, in case ProximityIn & ProximityOut is not reported
                if modifiers.contains(gdk::ModifierType::BUTTON1_MASK) {
                    state = PenState::Down;
                } else {
                    state = PenState::Proximity;
                }
            } else {
                // only handle no pressed button, primary and secondary mouse buttons
                if modifiers.is_empty()
                    || modifiers.contains(gdk::ModifierType::BUTTON1_MASK)
                    || modifiers.contains(gdk::ModifierType::BUTTON3_MASK)
                {
                    handle_pen_event = true;
                    inhibit = true;
                }
            }
        }
        gdk::EventType::ButtonPress => {
            let button_event = event.downcast_ref::<gdk::ButtonEvent>().unwrap();
            let gdk_button = button_event.button();

            //log::debug!("ButtonPress - button: {gdk_button}, is_stylus: {is_stylus}");

            if is_stylus {
                // even though it is a button press, we handle it also as pen event so the engine gets the chance to switch pen mode, pen style, etc.
                if gdk_button == gdk::BUTTON_PRIMARY
                    || gdk_button == gdk::BUTTON_SECONDARY
                    || gdk_button == gdk::BUTTON_MIDDLE
                {
                    handle_pen_event = true;
                    inhibit = true;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if gdk_button == gdk::BUTTON_PRIMARY || gdk_button == gdk::BUTTON_SECONDARY {
                    handle_pen_event = true;
                    inhibit = true;
                    state = PenState::Down;
                }
            }

            let shortcut_key = retrieve_button_shortcut_key(gdk_button, is_stylus);

            if let Some(shortcut_key) = shortcut_key {
                widget_flags.merge(
                    canvas
                        .engine()
                        .borrow_mut()
                        .handle_pressed_shortcut_key(shortcut_key, now),
                );
            }
        }
        gdk::EventType::ButtonRelease => {
            let button_event = event.downcast_ref::<gdk::ButtonEvent>().unwrap();
            let gdk_button = button_event.button();

            //log::debug!("ButtonRelease - button: {gdk_button}, is_stylus: {is_stylus}");

            if is_stylus {
                if gdk_button == gdk::BUTTON_PRIMARY
                    || gdk_button == gdk::BUTTON_SECONDARY
                    || gdk_button == gdk::BUTTON_MIDDLE
                {
                    handle_pen_event = true;
                    inhibit = true;
                }

                // again, this is the method to detect proximity on stylus
                if gdk_button == gdk::BUTTON_PRIMARY {
                    state = PenState::Up;
                } else {
                    state = PenState::Proximity;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if gdk_button == gdk::BUTTON_PRIMARY || gdk_button == gdk::BUTTON_SECONDARY {
                    state = PenState::Up;
                    handle_pen_event = true;
                    inhibit = true;
                }
            };
        }
        gdk::EventType::ProximityIn => {
            state = PenState::Proximity;
            handle_pen_event = true;
            inhibit = true;
        }
        gdk::EventType::ProximityOut => {
            state = PenState::Up;
            handle_pen_event = true;
            inhibit = true;
        }
        // We early-returned when detecting touch input and touch-drawing is not enabled, so it is fine to always handle it here
        gdk::EventType::TouchBegin => {
            state = PenState::Down;
            handle_pen_event = true;
            inhibit = true;
        }
        gdk::EventType::TouchUpdate => {
            state = PenState::Down;
            handle_pen_event = true;
            inhibit = true;
        }
        gdk::EventType::TouchEnd => {
            state = PenState::Up;
            handle_pen_event = true;
            inhibit = true;
        }
        gdk::EventType::TouchCancel => {
            state = PenState::Up;
            handle_pen_event = true;
            inhibit = true;
        }
        _ => {}
    };

    if handle_pen_event {
        let Some(element) = retrieve_pointer_element(canvas, event) else {
                    return (Inhibit(false), state);
                };
        let modifier_keys = retrieve_modifier_keys(event.modifier_state());
        let pen_mode = retrieve_pen_mode(event);

        //log::debug!("handle event, state: {state:?}, modifier_keys: {modifier_keys:?}, pen_mode: {pen_mode:?}");

        match state {
            PenState::Up => {
                canvas.switch_between_cursors(false);

                widget_flags.merge(canvas.engine().borrow_mut().handle_pen_event(
                    PenEvent::Up {
                        element,
                        modifier_keys,
                    },
                    pen_mode,
                    now,
                ));
            }
            PenState::Proximity => {
                canvas.switch_between_cursors(false);

                widget_flags.merge(canvas.engine().borrow_mut().handle_pen_event(
                    PenEvent::Proximity {
                        element,
                        modifier_keys,
                    },
                    pen_mode,
                    now,
                ));
            }
            PenState::Down => {
                canvas.grab_focus();
                canvas.switch_between_cursors(true);

                widget_flags.merge(canvas.engine().borrow_mut().handle_pen_event(
                    PenEvent::Down {
                        element,
                        modifier_keys,
                    },
                    pen_mode,
                    now,
                ));
            }
        }
    }

    canvas.emit_handle_widget_flags(widget_flags);
    (Inhibit(inhibit), state)
}

pub(crate) fn handle_key_controller_key_pressed(
    canvas: &RnCanvas,
    key: gdk::Key,
    modifier: gdk::ModifierType,
) -> Inhibit {
    //log::debug!("key pressed - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
    canvas.grab_focus();

    let now = Instant::now();
    let keyboard_key = retrieve_keyboard_key(key);
    let modifier_keys = retrieve_modifier_keys(modifier);

    //log::debug!("keyboard key: {:?}", keyboard_key);

    let widget_flags = canvas.engine().borrow_mut().handle_pen_event(
        PenEvent::KeyPressed {
            keyboard_key,
            modifier_keys,
        },
        None,
        now,
    );
    canvas.emit_handle_widget_flags(widget_flags);

    Inhibit(true)
}

pub(crate) fn handle_imcontext_text_commit(canvas: &RnCanvas, text: &str) {
    let now = Instant::now();
    let widget_flags = canvas.engine().borrow_mut().handle_pen_event(
        PenEvent::Text {
            text: text.to_string(),
        },
        None,
        now,
    );
    canvas.emit_handle_widget_flags(widget_flags);
}

#[allow(unused)]
fn debug_gdk_event(event: &gdk::Event) {
    log::debug!(
        "pos: {:?}, modifier: {:?}, event_type: {:?}, input source: {:?}",
        event.position(),
        event.modifier_state(),
        event.event_type(),
        event.device().map(|d| d.source())
    );
}

/// Returns true if input should be rejected
fn reject_pointer_input(event: &gdk::Event, touch_drawing: bool) -> bool {
    if !touch_drawing {
        let event_type = event.event_type();
        if event.is_pointer_emulated()
            || event_type == gdk::EventType::TouchBegin
            || event_type == gdk::EventType::TouchUpdate
            || event_type == gdk::EventType::TouchEnd
            || event_type == gdk::EventType::TouchCancel
        {
            return true;
        }
    }
    false
}

fn event_is_stylus(event: &gdk::Event) -> bool {
    // As in gtk4 'gtkgesturestylus.c:106' we detect if the pointer is a stylus when it has a device_tool
    event.device_tool().is_some()
}

fn retrieve_pointer_element(canvas: &RnCanvas, event: &gdk::Event) -> Option<Element> {
    let Some((x, y)) = event.position() else {
        return None;
    };
    let root = canvas.root().unwrap();
    let (surface_trans_x, surface_trans_y) = root.surface_transform();

    let pos = root
        .translate_coordinates(canvas, x - surface_trans_x, y - surface_trans_y)
        .map(|(x, y)| {
            let pos = na::vector![x, y];
            (canvas.engine().borrow().camera.transform().inverse() * na::Point2::from(pos)).coords
        })
        .unwrap();

    // getting the pressure only works when the event has a device tool (== is a stylus),
    // else we get SIGSEGV when trying to access (TODO: report this to bindings)
    let is_stylus = event_is_stylus(event);

    let pressure = if is_stylus {
        event.axis(gdk::AxisUse::Pressure).unwrap()
    } else {
        Element::PRESSURE_DEFAULT
    };

    Some(Element::new(pos, pressure))
}

pub(crate) fn retrieve_button_shortcut_key(
    gdk_button: u32,
    is_stylus: bool,
) -> Option<ShortcutKey> {
    match gdk_button {
        gdk::BUTTON_PRIMARY => None,
        gdk::BUTTON_SECONDARY => {
            if is_stylus {
                Some(ShortcutKey::StylusPrimaryButton)
            } else {
                Some(ShortcutKey::MouseSecondaryButton)
            }
        }
        gdk::BUTTON_MIDDLE => {
            if is_stylus {
                Some(ShortcutKey::StylusSecondaryButton)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn retrieve_modifier_keys(modifier: gdk::ModifierType) -> Vec<ModifierKey> {
    let mut keys = vec![];

    if modifier.contains(gdk::ModifierType::SHIFT_MASK) {
        keys.push(ModifierKey::KeyboardShift);
    }
    if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
        keys.push(ModifierKey::KeyboardCtrl);
    }
    if modifier.contains(gdk::ModifierType::ALT_MASK) {
        keys.push(ModifierKey::KeyboardAlt);
    }
    keys
}

fn retrieve_pen_mode(event: &gdk::Event) -> Option<PenMode> {
    if let Some(device_tool) = event.device_tool() {
        match device_tool.tool_type() {
            gdk::DeviceToolType::Pen => {
                return Some(PenMode::Pen);
            }
            gdk::DeviceToolType::Eraser => {
                return Some(PenMode::Eraser);
            }
            _ => {}
        }
    }

    None
}

pub(crate) fn retrieve_keyboard_key(gdk_key: gdk::Key) -> KeyboardKey {
    //log::debug!("gdk: pressed key: {:?}", gdk_key);

    if let Some(keychar) = gdk_key.to_unicode() {
        KeyboardKey::Unicode(keychar).filter_convert_unicode_control_chars()
    } else {
        match gdk_key {
            gdk::Key::BackSpace => KeyboardKey::BackSpace,
            gdk::Key::Tab => KeyboardKey::HorizontalTab,
            gdk::Key::Linefeed => KeyboardKey::Linefeed,
            gdk::Key::Return => KeyboardKey::CarriageReturn,
            gdk::Key::Escape => KeyboardKey::Escape,
            gdk::Key::Delete => KeyboardKey::Delete,
            gdk::Key::Down => KeyboardKey::NavDown,
            gdk::Key::Up => KeyboardKey::NavUp,
            gdk::Key::Left => KeyboardKey::NavLeft,
            gdk::Key::Right => KeyboardKey::NavRight,
            gdk::Key::Shift_L => KeyboardKey::ShiftLeft,
            gdk::Key::Shift_R => KeyboardKey::ShiftRight,
            gdk::Key::Control_L => KeyboardKey::CtrlLeft,
            gdk::Key::Control_R => KeyboardKey::CtrlRight,
            _ => KeyboardKey::Unsupported,
        }
    }
}
