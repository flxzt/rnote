use gtk4::{gdk, prelude::*, GestureStylus};
use rnote_engine::pens::penbehaviour::PenBehaviour;
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

/// Retreive inputdata from a (emulated) pointer
/// X and Y is already available from closure, and should not retreived from .axis() (because of gtk weirdness)
pub fn retreive_pointer_inputdata(x: f64, y: f64) -> VecDeque<InputData> {
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
pub fn retreive_stylus_inputdata(
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

/// Process the start of the pen input ( "Pen down" )
pub fn process_peninput_start(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
    let current_pen_style = appwindow.canvas().pens().borrow().current_style();
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

    appwindow.canvas().pens().borrow_mut().begin(
        data_entries,
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.canvas().queue_draw();
}

/// Process the motion of the pen input ( "Pen moves while down" )
pub fn process_peninput_motion(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
    let current_pen_style = appwindow.canvas().pens().borrow().current_style();

    appwindow.audioplayer().borrow().play_pen_sound_motion(
        RnoteAudioPlayer::PLAY_TIMEOUT_TIME,
        current_pen_style,
        &*appwindow.canvas().pens().borrow(),
    );

    appwindow.canvas().pens().borrow_mut().motion(
        data_entries,
        &mut *appwindow.canvas().sheet().borrow_mut(),
        Some(appwindow.canvas().viewport_in_sheet_coords()),
        appwindow.canvas().zoom(),
        appwindow.canvas().renderer(),
    );

    appwindow.canvas().resize_sheet_autoexpand();
    appwindow.canvas().update_background_rendernode(true);
    appwindow.canvas().queue_draw();
}

/// Process the end of the pen input ( "Pen up" )
pub fn process_peninput_end(data_entries: VecDeque<InputData>, appwindow: &RnoteAppWindow) {
    appwindow
        .canvas()
        .set_cursor(Some(&appwindow.canvas().cursor()));

    // We deselect the selection here. (before current_pen.end()!)
    let all_strokes = appwindow
        .canvas()
        .sheet()
        .borrow()
        .strokes_state
        .keys_sorted_chrono();
    appwindow
        .canvas()
        .sheet()
        .borrow_mut()
        .strokes_state
        .set_selected_keys(&all_strokes, false);

    appwindow.canvas().pens().borrow_mut().end(
        data_entries,
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
