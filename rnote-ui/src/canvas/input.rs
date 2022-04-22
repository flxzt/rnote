use gtk4::{gdk, prelude::*, GestureDrag, GestureStylus};
use rnote_compose::penhelpers::PenEvent;
use rnote_compose::penhelpers::ShortcutKey;
use rnote_compose::penpath::Element;
use rnote_engine::pens::penholder::PenHolderEvent;
use rnote_engine::SurfaceFlags;
use std::collections::VecDeque;

use crate::appwindow::RnoteAppWindow;

/// Returns true if input should be rejected
pub fn filter_mouse_input(mouse_drawing_gesture: &GestureDrag) -> bool {
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
pub fn filter_touch_input(_touch_drawing_gesture: &GestureDrag) -> bool {
    false
}

/// Returns true if input should be rejected
pub fn filter_stylus_input(_stylus_drawing_gesture: &GestureStylus) -> bool {
    false
}

#[allow(dead_code)]
pub fn debug_stylus_gesture(stylus_gesture: &GestureStylus) {
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

#[allow(dead_code)]
pub fn debug_drag_gesture(drag_gesture: &GestureDrag) {
    log::debug!(
        "gesture modifier: {:?}, current_button: {:?}, event.event_type: {:?}",
        drag_gesture.current_event_state(),
        drag_gesture.current_button(),
        drag_gesture
            .current_event()
            .map(|event| { event.event_type() })
    );
}

/// Retreive elements from a (emulated) pointer
/// X and Y is already available from closure, and should not retreived from .axis() (because of gtk weirdness)
pub fn retreive_pointer_elements(
    _mouse_drawing_gesture: &GestureDrag,
    x: f64,
    y: f64,
) -> VecDeque<Element> {
    let mut data_entries: VecDeque<Element> = VecDeque::with_capacity(1);
    //std::thread::sleep(std::time::Duration::from_millis(100));

    data_entries.push_back(Element::new(na::vector![x, y], Element::PRESSURE_DEFAULT));
    data_entries
}

pub fn retreive_mouse_shortcut_keys(mouse_drawing_gesture: &GestureDrag) -> Vec<ShortcutKey> {
    let mut shortcut_keys = vec![];

    match mouse_drawing_gesture.current_button() {
        gdk::BUTTON_SECONDARY => {
            shortcut_keys.push(ShortcutKey::MouseSecondaryButton);
        }
        _ => {}
    }

    shortcut_keys.append(&mut retreive_modifier_shortcut_key(
        mouse_drawing_gesture.current_event_state(),
    ));

    shortcut_keys
}

pub fn retreive_touch_shortcut_keys(touch_drawing_gesture: &GestureDrag) -> Vec<ShortcutKey> {
    let mut shortcut_keys = vec![];

    shortcut_keys.append(&mut retreive_modifier_shortcut_key(
        touch_drawing_gesture.current_event_state(),
    ));

    shortcut_keys
}

/// Retreiving the shortcut keys for the stylus gesture
pub fn retreive_stylus_shortcut_keys(stylus_drawing_gesture: &GestureStylus) -> Vec<ShortcutKey> {
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

    if let Some(device_tool) = stylus_drawing_gesture.device_tool() {
        // Eraser is the lower stylus button
        match device_tool.tool_type() {
            gdk::DeviceToolType::Pen => {}
            gdk::DeviceToolType::Eraser => {
                shortcut_keys.push(ShortcutKey::StylusEraserMode);
            }
            _ => {}
        }
    }

    shortcut_keys.append(&mut retreive_modifier_shortcut_key(
        stylus_drawing_gesture.current_event_state(),
    ));

    shortcut_keys
}

pub fn retreive_keyboard_key_shortcut_key(key: gdk::Key) -> Option<ShortcutKey> {
    key.to_unicode()
        .map(|key_char| ShortcutKey::KeyboardKey(key_char))
}

/// Retreiving modifier shortcut keys. Note that here Button modifiers are skipped, they have different meanings with different kind of pointers and have to be handled individually
pub fn retreive_modifier_shortcut_key(modifier: gdk::ModifierType) -> Vec<ShortcutKey> {
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

/// Retreives available input axes, defaults if not available.
/// X and Y is already available from closure, and should not retreived from .axis() (because of gtk weirdness)
pub fn retreive_stylus_elements(
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
pub fn process_pen_down(
    element: Element,
    shortcut_keys: Vec<ShortcutKey>,
    appwindow: &RnoteAppWindow,
) {
    let mut surface_flags = SurfaceFlags::default();

    appwindow
        .canvas()
        .set_cursor(Some(&appwindow.canvas().motion_cursor()));

    // GTK emits separate down events when pressing the stylus primary / secondary button (even when the pen is only in proximity), so we skip handling those as a DownEvent and emit pressed shortcut key eventas
    if shortcut_keys.contains(&ShortcutKey::StylusPrimaryButton)
        || shortcut_keys.contains(&ShortcutKey::StylusSecondaryButton)
    {
        for shortcut_key in shortcut_keys.iter() {
            surface_flags.merge_with_other(
                appwindow
                    .canvas()
                    .engine()
                    .borrow_mut()
                    .handle_penholder_event(PenHolderEvent::PressedShortcutkey(*shortcut_key)),
            );
        }

        appwindow.handle_surface_flags(surface_flags);
        return;
    }

    surface_flags.merge_with_other(
        appwindow
            .canvas()
            .engine()
            .borrow_mut()
            .handle_penholder_event(PenHolderEvent::PenEvent(PenEvent::Down {
                element,
                shortcut_keys,
            })),
    );

    appwindow.handle_surface_flags(surface_flags);
}

/// Process "Pen motion"
pub fn process_pen_motion(
    data_entries: VecDeque<Element>,
    shortcut_keys: Vec<ShortcutKey>,
    appwindow: &RnoteAppWindow,
) {
    let surface_flags = data_entries
        .into_iter()
        .map(|element| {
            appwindow
                .canvas()
                .engine()
                .borrow_mut()
                .handle_penholder_event(PenHolderEvent::PenEvent(PenEvent::Down {
                    element,
                    shortcut_keys: shortcut_keys.clone(),
                }))
        })
        .fold(SurfaceFlags::default(), |acc, x| acc.merged_with_other(x));

    appwindow.handle_surface_flags(surface_flags);
}

/// Process "Pen up"
pub fn process_pen_up(
    element: Element,
    shortcut_keys: Vec<ShortcutKey>,
    appwindow: &RnoteAppWindow,
) {
    let mut surface_flags = SurfaceFlags::default();

    // GTK emits separate down events when pressing the stylus primary / secondary button (even when the pen is only in proximity), so we skip handling those as a DownEvent and emit pressed shortcut key eventas
    if shortcut_keys.contains(&ShortcutKey::StylusPrimaryButton)
        || shortcut_keys.contains(&ShortcutKey::StylusSecondaryButton)
    {
        for shortcut_key in shortcut_keys.iter() {
            surface_flags.merge_with_other(
                appwindow
                    .canvas()
                    .engine()
                    .borrow_mut()
                    .handle_penholder_event(PenHolderEvent::PressedShortcutkey(*shortcut_key)),
            );
        }

        appwindow.handle_surface_flags(surface_flags);
        return;
    }

    surface_flags.merge_with_other(
        appwindow
            .canvas()
            .engine()
            .borrow_mut()
            .handle_penholder_event(PenHolderEvent::PenEvent(PenEvent::Up {
                element,
                shortcut_keys,
            })),
    );

    appwindow.handle_surface_flags(surface_flags);
}

/// Process "Pen proximity"
pub fn process_pen_proximity(
    data_entries: VecDeque<Element>,
    shortcut_keys: Vec<ShortcutKey>,
    appwindow: &RnoteAppWindow,
) {
    let surface_flags = data_entries
        .into_iter()
        .map(|element| {
            appwindow
                .canvas()
                .engine()
                .borrow_mut()
                .handle_penholder_event(PenHolderEvent::PenEvent(PenEvent::Proximity {
                    element,
                    shortcut_keys: shortcut_keys.clone(),
                }))
        })
        .fold(SurfaceFlags::default(), |acc, x| acc.merged_with_other(x));

    appwindow.handle_surface_flags(surface_flags);
}

/// Process keyboard key pressed
pub fn process_keyboard_pressed(shortcut_key: ShortcutKey, appwindow: &RnoteAppWindow) {
    let surface_flags = appwindow
        .canvas()
        .engine()
        .borrow_mut()
        .handle_penholder_event(PenHolderEvent::PressedShortcutkey(shortcut_key));

    appwindow.handle_surface_flags(surface_flags);
}
