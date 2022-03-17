use gtk4::{gdk, prelude::*, GestureDrag, GestureStylus};
use rnote_engine::pens::shortcuts::ShortcutKey;
use rnote_engine::pens::PenEvent;
use std::collections::VecDeque;

use crate::appwindow::RnoteAppWindow;
use crate::audioplayer::RnoteAudioPlayer;
use rnote_engine::strokes::inputdata::InputData;

/// transform pen input with zoom and offset
pub fn transform_inputdata(
    data_entries: &mut VecDeque<InputData>,
    offset: na::Vector2<f64>,
    zoom: f64,
) {
    data_entries.iter_mut().for_each(|inputdata| {
        *inputdata = InputData::new(
            inputdata.pos().scale(1.0 / zoom) + offset,
            inputdata.pressure(),
        )
    });
}

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

/// Retreive inputdata from a (emulated) pointer
/// X and Y is already available from closure, and should not retreived from .axis() (because of gtk weirdness)
pub fn retreive_pointer_inputdata(
    _mouse_drawing_gesture: &GestureDrag,
    x: f64,
    y: f64,
) -> VecDeque<InputData> {
    let mut data_entries: VecDeque<InputData> = VecDeque::with_capacity(1);
    //std::thread::sleep(std::time::Duration::from_millis(100));

    data_entries.push_back(InputData::new(
        na::vector![x, y],
        InputData::PRESSURE_DEFAULT,
    ));
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
                shortcut_key = Some(ShortcutKey::StylusEraserButton);
            }
            _ => {}
        }
    }

    shortcut_key
}

/// Retreives available input axes, defaults if not available.
/// X and Y is already available from closure, and should not retreived from .axis() (because of gtk weirdness)
pub fn retreive_stylus_inputdata(
    stylus_drawing_gesture: &GestureStylus,
    x: f64,
    y: f64,
) -> VecDeque<InputData> {
    let mut data_entries: VecDeque<InputData> = VecDeque::new();
    //std::thread::sleep(std::time::Duration::from_millis(100));

    // Get newest data
    let pressure = if let Some(pressure) = stylus_drawing_gesture.axis(gdk::AxisUse::Pressure) {
        pressure
    } else {
        InputData::PRESSURE_DEFAULT
    };

    data_entries.push_back(InputData::new(na::vector![x, y], pressure));

    data_entries
}

/// Process "Pen down"
pub fn process_pen_down(
    data_entries: VecDeque<InputData>,
    appwindow: &RnoteAppWindow,
    shortcut_key: Option<ShortcutKey>,
) {
    let current_pen_style = appwindow.canvas().pens().borrow().style_w_override();
    appwindow
        .canvas()
        .set_cursor(Some(&appwindow.canvas().motion_cursor()));

    appwindow.audioplayer().borrow().play_pen_sound_begin(
        RnoteAudioPlayer::PLAY_TIMEOUT_TIME,
        current_pen_style,
        &*appwindow.canvas().pens().borrow(),
    );

    // We hide the selection modifier here already, but actually only deselect all strokes when ending the stroke (for responsiveness reasons)
    appwindow.canvas().selection_modifier().set_visible(false);

    let surface_flags = appwindow.canvas().pens().borrow_mut().handle_event(
        PenEvent::DownEvent {
            data_entries,
            shortcut_key,
        },
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.handle_surface_flags(surface_flags);
}

/// Process "Pen motions"
pub fn process_pen_motion(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
    let current_pen_style = appwindow.canvas().pens().borrow().style_w_override();

    appwindow.audioplayer().borrow().play_pen_sound_motion(
        RnoteAudioPlayer::PLAY_TIMEOUT_TIME,
        current_pen_style,
        &*appwindow.canvas().pens().borrow(),
    );

    let surface_flags = appwindow.canvas().pens().borrow_mut().handle_event(
        PenEvent::MotionEvent { data_entries },
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.handle_surface_flags(surface_flags);
}

/// Process "Pen up"
pub fn process_pen_up(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
    appwindow
        .canvas()
        .set_cursor(Some(&appwindow.canvas().cursor()));

    let surface_flags = appwindow.canvas().pens().borrow_mut().handle_event(
        PenEvent::UpEvent { data_entries },
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.handle_surface_flags(surface_flags);
}
