use gtk4::{gdk, prelude::*, GestureDrag, GestureStylus};
use rnote_compose::penevent::ShortcutKey;
use rnote_compose::penpath::Element;
use rnote_compose::PenEvent;
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

pub fn debug_stylus_gesture(stylus_gesture: &GestureStylus) {
    log::debug!(
        "gesture modifier: {:?}, current_button: {:?}, tool_type: {:?}, event.event_type: {:?}",
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

pub fn retreive_mouse_shortcut_key(mouse_drawing_gesture: &GestureDrag) -> Option<ShortcutKey> {
    let mut shortcut_key = None;

    match mouse_drawing_gesture.current_button() {
        gdk::BUTTON_SECONDARY => {
            shortcut_key = Some(ShortcutKey::MouseSecondaryButton);
        }
        _ => {}
    }

    shortcut_key
}

/// Retreiving any shortcut key for the stylus gesture
pub fn retreive_stylus_shortcut_key(stylus_drawing_gesture: &GestureStylus) -> Option<ShortcutKey> {
    let mut shortcut_key = None;

    // the middle / secondary buttons are the lower or upper buttons on the stylus, but the mapping on gtk's side is inconsistent.
    // Also, libinput sometimes picks one button as tool_type: Eraser, but this is not supported by all devices.
    match stylus_drawing_gesture.current_button() {
        gdk::BUTTON_MIDDLE => {
            shortcut_key = Some(ShortcutKey::StylusPrimaryButton);
        }
        gdk::BUTTON_SECONDARY => {
            shortcut_key = Some(ShortcutKey::StylusSecondaryButton);
        }
        _ => {}
    };
    if let Some(device_tool) = stylus_drawing_gesture.device_tool() {
        // Eraser is the lower stylus button
        match device_tool.tool_type() {
            gdk::DeviceToolType::Pen => {}
            gdk::DeviceToolType::Eraser => {
                shortcut_key = Some(ShortcutKey::StylusEraserMode);
            }
            _ => {}
        }
    }

    shortcut_key
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
    shortcut_key: Option<ShortcutKey>,
    appwindow: &RnoteAppWindow,
) {
    let pen_event = match shortcut_key {
        // Stylus button presses are emitting separate down / up events, so we handle them here differently to only change the pen, not start / stop drawing
        Some(ShortcutKey::StylusPrimaryButton) | Some(ShortcutKey::StylusSecondaryButton) => {
            PenHolderEvent::PressedShortcutkey(shortcut_key.unwrap())
        }
        _ => {
            appwindow
                .canvas()
                .set_cursor(Some(&appwindow.canvas().motion_cursor()));

            // We hide the selection modifier here already, but actually only deselect all strokes when ending the stroke (for performance reasons)
            appwindow.canvas().selection_modifier().set_visible(false);

            PenHolderEvent::PenEvent(PenEvent::Down {
                element,
                shortcut_key,
            })
        }
    };

    let surface_flags = appwindow
        .canvas()
        .engine()
        .borrow_mut()
        .handle_event(pen_event);

    appwindow.handle_surface_flags(surface_flags);
}

/// Process "Pen motion"
pub fn process_pen_motion(
    data_entries: VecDeque<Element>,
    shortcut_key: Option<ShortcutKey>,
    appwindow: &RnoteAppWindow,
) {
    let surface_flags = data_entries
        .into_iter()
        .map(|element| {
            appwindow
                .canvas()
                .engine()
                .borrow_mut()
                .handle_event(PenHolderEvent::PenEvent(PenEvent::Down {
                    element,
                    shortcut_key,
                }))
        })
        .fold(SurfaceFlags::default(), |acc, x| acc.merged_with_other(x));

    appwindow.handle_surface_flags(surface_flags);
}

/// Process "Pen up"
pub fn process_pen_up(
    element: Element,
    shortcut_key: Option<ShortcutKey>,
    appwindow: &RnoteAppWindow,
) {
    let pen_event = match shortcut_key {
        // Stylus button presses are emitting separate down / up events, so we handle them here differently to only change the pen, not start /stop drawing
        Some(ShortcutKey::StylusPrimaryButton) | Some(ShortcutKey::StylusSecondaryButton) => {
            PenHolderEvent::PressedShortcutkey(shortcut_key.unwrap())
        }
        _ => {
            appwindow
                .canvas()
                .set_cursor(Some(&appwindow.canvas().cursor()));

            PenHolderEvent::PenEvent(PenEvent::Up {
                element,
                shortcut_key,
            })
        }
    };
    let surface_flags = appwindow
        .canvas()
        .engine()
        .borrow_mut()
        .handle_event(pen_event);

    appwindow.handle_surface_flags(surface_flags);
}
