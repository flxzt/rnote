use gtk4::{gdk, prelude::*, GestureDrag, GestureStylus};
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
pub fn filter_mouse_drawing_gesture_input(mouse_drawing_gesture: &GestureDrag) -> bool {
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
pub fn filter_stylus_drawing_gesture_input(_stylus_drawing_gesture: &GestureStylus) -> bool {
    false
}

pub fn debug_stylus_gesture(stylus_gesture: &GestureStylus) {
    log::debug!(
        "gesture modifier: {:?}",
        stylus_gesture.current_event_state()
    );
    log::debug!(
        "gesture current_button(): {:?}",
        stylus_gesture.current_button()
    );
    log::debug!(
        "gesture tool_type(): {:?}",
        stylus_gesture
            .device_tool()
            .map(|device_tool| { device_tool.tool_type() })
    );
    log::debug!(
        "gesture event.event_type(): {:?}",
        stylus_gesture
            .current_event()
            .map(|event| { event.event_type() })
    );
}

/// Retreive inputdata from a (emulated) pointer
/// X and Y is already available from closure, and should not retreived from .axis() (because of gtk weirdness)
pub fn retreive_pointer_drawing_gesture_inputdata(
    _mouse_gesture: &GestureDrag,
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

/// Retreives available input axes, defaults if not available.
/// X and Y is already available from closure, and should not retreived from .axis() (because of gtk weirdness)
pub fn retreive_stylus_drawing_gesture_inputdata(
    gesture_stylus: &GestureStylus,
    x: f64,
    y: f64,
) -> VecDeque<InputData> {
    let mut data_entries: VecDeque<InputData> = VecDeque::new();
    //std::thread::sleep(std::time::Duration::from_millis(100));

    // Get newest data
    let pressure = if let Some(pressure) = gesture_stylus.axis(gdk::AxisUse::Pressure) {
        pressure
    } else {
        InputData::PRESSURE_DEFAULT
    };

    data_entries.push_back(InputData::new(na::vector![x, y], pressure));

    data_entries
}

/// Process "Pen down"
pub fn process_pen_down(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
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

    appwindow.canvas().pens().borrow_mut().handle_event(
        PenEvent::DownEvent(data_entries),
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.canvas().queue_draw();
}

/// Process "Pen motions"
pub fn process_pen_motion(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
    let current_pen_style = appwindow.canvas().pens().borrow().style_w_override();

    appwindow.audioplayer().borrow().play_pen_sound_motion(
        RnoteAudioPlayer::PLAY_TIMEOUT_TIME,
        current_pen_style,
        &*appwindow.canvas().pens().borrow(),
    );

    appwindow.canvas().pens().borrow_mut().handle_event(
        PenEvent::MotionEvent(data_entries),
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.canvas().resize_sheet_autoexpand();
    appwindow.canvas().update_background_rendernode(true);
    appwindow.canvas().queue_draw();
}

/// Process "Pen up"
pub fn process_pen_up(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
    appwindow
        .canvas()
        .set_cursor(Some(&appwindow.canvas().cursor()));

    appwindow.canvas().pens().borrow_mut().handle_event(
        PenEvent::UpEvent(data_entries),
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.canvas().resize_sheet_autoexpand();
    appwindow.canvas().update_background_rendernode(true);

    appwindow
        .canvas()
        .selection_modifier()
        .update_state(&appwindow.canvas());

    appwindow.canvas().set_unsaved_changes(true);
    appwindow.canvas().set_empty(false);

    appwindow.canvas().queue_resize();
}
