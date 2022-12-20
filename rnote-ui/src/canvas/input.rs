use gtk4::{gdk, prelude::*, GestureDrag, GestureStylus};
use rnote_compose::penevents::KeyboardKey;
use rnote_compose::penevents::PenEvent;
use rnote_compose::penevents::ShortcutKey;
use rnote_compose::penpath::Element;
use rnote_engine::pens::PenMode;
use rnote_engine::WidgetFlags;
use std::collections::VecDeque;
use std::time::Instant;

use crate::appwindow::RnoteAppWindow;

/// Returns true if input should be rejected
pub(crate) fn filter_mouse_input(mouse_drawing_gesture: &GestureDrag) -> bool {
    match mouse_drawing_gesture.current_button() {
        gdk::BUTTON_PRIMARY | gdk::BUTTON_SECONDARY => {}
        _ => {
            return true;
        }
    }
    if let Some(event) = mouse_drawing_gesture.current_event() {
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

/// Returns true if input should be rejected
pub(crate) fn filter_touch_input(_touch_drawing_gesture: &GestureDrag) -> bool {
    false
}

/// Returns true if input should be rejected
pub(crate) fn filter_stylus_input(_stylus_drawing_gesture: &GestureStylus) -> bool {
    false
}

#[allow(unused)]
pub(crate) fn debug_stylus_gesture(stylus_gesture: &GestureStylus) {
    log::debug!(
        "stylus_gesture | modifier: {:?}, current_button: {:?}, tool_type: {:?}, event.event_type: {:?}",
        stylus_gesture.current_event_state(),
        stylus_gesture.current_button(),
        stylus_gesture
            .device_tool()
            .map(|device_tool| { device_tool.tool_type() }),
        stylus_gesture
            .current_event()
            .map(|event| { event.event_type() })
    );
}

#[allow(unused)]
pub(crate) fn debug_drag_gesture(drag_gesture: &GestureDrag) {
    log::debug!(
        "gesture modifier: {:?}, current_button: {:?}, event.event_type: {:?}",
        drag_gesture.current_event_state(),
        drag_gesture.current_button(),
        drag_gesture
            .current_event()
            .map(|event| { event.event_type() })
    );
}

/// retrieve elements from a (emulated) pointer
/// X and Y is already available from closure, and should not retrieved from .axis() (because of gtk weirdness)
pub(crate) fn retrieve_pointer_elements(
    _mouse_drawing_gesture: &GestureDrag,
    x: f64,
    y: f64,
) -> VecDeque<Element> {
    let mut data_entries: VecDeque<Element> = VecDeque::with_capacity(1);
    //std::thread::sleep(std::time::Duration::from_millis(100));

    data_entries.push_back(Element::new(na::vector![x, y], Element::PRESSURE_DEFAULT));
    data_entries
}

pub(crate) fn retrieve_mouse_shortcut_keys(
    mouse_drawing_gesture: &GestureDrag,
) -> Vec<ShortcutKey> {
    let mut shortcut_keys = vec![];

    match mouse_drawing_gesture.current_button() {
        gdk::BUTTON_SECONDARY => {
            shortcut_keys.push(ShortcutKey::MouseSecondaryButton);
        }
        _ => {}
    }

    shortcut_keys.append(&mut retrieve_modifier_shortcut_key(
        mouse_drawing_gesture.current_event_state(),
    ));

    shortcut_keys
}

pub(crate) fn retrieve_touch_shortcut_keys(
    touch_drawing_gesture: &GestureDrag,
) -> Vec<ShortcutKey> {
    let mut shortcut_keys = vec![];

    shortcut_keys.append(&mut retrieve_modifier_shortcut_key(
        touch_drawing_gesture.current_event_state(),
    ));

    shortcut_keys
}

/// Retrieving the shortcut keys for the stylus gesture
pub(crate) fn retrieve_stylus_shortcut_keys(
    stylus_drawing_gesture: &GestureStylus,
) -> Vec<ShortcutKey> {
    let mut shortcut_keys = vec![];

    // the middle / secondary buttons are the lower or upper buttons on the stylus, but the mapping on gtk's side is inconsistent.
    // Also, libinput sometimes picks one button as tool_type: Eraser, but this is not supported by all devices.
    match stylus_drawing_gesture.current_button() {
        gdk::BUTTON_MIDDLE => {
            shortcut_keys.push(ShortcutKey::StylusPrimaryButton);
        }
        gdk::BUTTON_SECONDARY => {
            shortcut_keys.push(ShortcutKey::StylusSecondaryButton);
        }
        _ => {}
    };

    shortcut_keys.append(&mut retrieve_modifier_shortcut_key(
        stylus_drawing_gesture.current_event_state(),
    ));

    shortcut_keys
}

pub(crate) fn retrieve_stylus_pen_mode(stylus_drawing_gesture: &GestureStylus) -> Option<PenMode> {
    if let Some(device_tool) = stylus_drawing_gesture.device_tool() {
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
    rnote_engine::utils::keyboard_key_from_gdk(gdk_key)
}

/// Retrieving modifier shortcut keys. Note that here Button modifiers are skipped, they have different meanings with different kind of pointers and have to be handled individually
pub(crate) fn retrieve_modifier_shortcut_key(modifier: gdk::ModifierType) -> Vec<ShortcutKey> {
    let mut shortcut_keys = vec![];
    if modifier.contains(gdk::ModifierType::SHIFT_MASK) {
        shortcut_keys.push(ShortcutKey::KeyboardShift);
    }
    if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
        shortcut_keys.push(ShortcutKey::KeyboardCtrl);
    }
    if modifier.contains(gdk::ModifierType::ALT_MASK) {
        shortcut_keys.push(ShortcutKey::KeyboardAlt);
    }

    shortcut_keys
}

/// retrieves available input axes, defaults if not available.
/// X and Y is already available from closure, and should not retrieved from .axis() (because of gtk weirdness)
pub(crate) fn retrieve_stylus_elements(
    stylus_drawing_gesture: &GestureStylus,
    x: f64,
    y: f64,
) -> VecDeque<Element> {
    let mut data_entries: VecDeque<Element> = VecDeque::new();
    //std::thread::sleep(std::time::Duration::from_millis(100));

    // Get newest data
    let pressure = if let Some(pressure) = stylus_drawing_gesture.axis(gdk::AxisUse::Pressure) {
        pressure
    } else {
        Element::PRESSURE_DEFAULT
    };

    data_entries.push_back(Element::new(na::vector![x, y], pressure));

    data_entries
}

/// Process "Pen down"
pub(crate) fn process_pen_down(
    element: Element,
    shortcut_keys: Vec<ShortcutKey>,
    pen_mode: Option<PenMode>,
    now: Instant,
    appwindow: &RnoteAppWindow,
) {
    let mut widget_flags = WidgetFlags::default();

    appwindow.canvas().switch_between_cursors(true);

    // GTK emits separate down / up events when pressing / releasing the stylus primary / secondary button (even when the pen is only in proximity),
    // so we skip handling those as a Pen Events and emit pressed shortcut key events
    // TODO: handle this better
    if shortcut_keys.contains(&ShortcutKey::StylusPrimaryButton) {
        widget_flags.merge(
            appwindow
                .canvas()
                .engine()
                .borrow_mut()
                .handle_pen_pressed_shortcut_key(ShortcutKey::StylusPrimaryButton, now),
        );

        appwindow.handle_widget_flags(widget_flags);
        return;
    }
    if shortcut_keys.contains(&ShortcutKey::StylusSecondaryButton) {
        widget_flags.merge(
            appwindow
                .canvas()
                .engine()
                .borrow_mut()
                .handle_pen_pressed_shortcut_key(ShortcutKey::StylusSecondaryButton, now),
        );

        appwindow.handle_widget_flags(widget_flags);
        return;
    }

    // Handle all other events as pen down
    widget_flags.merge(appwindow.canvas().engine().borrow_mut().handle_pen_event(
        PenEvent::Down {
            element,
            shortcut_keys,
        },
        pen_mode,
        now,
    ));

    appwindow.handle_widget_flags(widget_flags);
}

/// Process "Pen up"
pub(crate) fn process_pen_up(
    element: Element,
    shortcut_keys: Vec<ShortcutKey>,
    pen_mode: Option<PenMode>,
    now: Instant,
    appwindow: &RnoteAppWindow,
) {
    let mut widget_flags = WidgetFlags::default();

    appwindow.canvas().switch_between_cursors(false);

    // GTK emits separate down / up events when pressing / releasing the stylus primary / secondary button (even when the pen is only in proximity),
    // so we skip handling those as a Pen Events and emit pressed shortcut key events
    // TODO: handle this better
    if shortcut_keys.contains(&ShortcutKey::StylusPrimaryButton) {
        widget_flags.merge(
            appwindow
                .canvas()
                .engine()
                .borrow_mut()
                .handle_pen_pressed_shortcut_key(ShortcutKey::StylusPrimaryButton, now),
        );

        appwindow.handle_widget_flags(widget_flags);
        return;
    }
    if shortcut_keys.contains(&ShortcutKey::StylusSecondaryButton) {
        widget_flags.merge(
            appwindow
                .canvas()
                .engine()
                .borrow_mut()
                .handle_pen_pressed_shortcut_key(ShortcutKey::StylusSecondaryButton, now),
        );

        appwindow.handle_widget_flags(widget_flags);
        return;
    }

    // Handle all other events as pen up
    widget_flags.merge(appwindow.canvas().engine().borrow_mut().handle_pen_event(
        PenEvent::Up {
            element,
            shortcut_keys,
        },
        pen_mode,
        now,
    ));

    appwindow.handle_widget_flags(widget_flags);
}

/// Process "Pen proximity"
pub(crate) fn process_pen_proximity(
    element: Element,
    shortcut_keys: Vec<ShortcutKey>,
    pen_mode: Option<PenMode>,
    now: Instant,
    appwindow: &RnoteAppWindow,
) {
    let mut widget_flags = WidgetFlags::default();

    widget_flags.merge(appwindow.canvas().engine().borrow_mut().handle_pen_event(
        PenEvent::Proximity {
            element,
            shortcut_keys,
        },
        pen_mode,
        now,
    ));

    appwindow.handle_widget_flags(widget_flags);
}

/// Process shortcut key pressed
#[allow(unused)]
pub(crate) fn process_shortcut_key_pressed(
    shortcut_key: ShortcutKey,
    now: Instant,
    appwindow: &RnoteAppWindow,
) {
    let widget_flags = appwindow
        .canvas()
        .engine()
        .borrow_mut()
        .handle_pen_pressed_shortcut_key(shortcut_key, now);

    appwindow.handle_widget_flags(widget_flags);
}

/// Process keyboard key pressed
pub(crate) fn process_keyboard_key_pressed(
    keyboard_key: KeyboardKey,
    shortcut_keys: Vec<ShortcutKey>,
    now: Instant,
    appwindow: &RnoteAppWindow,
) {
    let widget_flags = appwindow.canvas().engine().borrow_mut().handle_pen_event(
        PenEvent::KeyPressed {
            keyboard_key,
            shortcut_keys,
        },
        None,
        now,
    );

    appwindow.handle_widget_flags(widget_flags);
}

/// Process keyboard text
pub(crate) fn process_keyboard_text(text: String, now: Instant, appwindow: &RnoteAppWindow) {
    let widget_flags = appwindow.canvas().engine().borrow_mut().handle_pen_event(
        PenEvent::Text { text },
        None,
        now,
    );

    appwindow.handle_widget_flags(widget_flags);
}
