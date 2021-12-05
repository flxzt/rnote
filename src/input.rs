use gtk4::{gdk, GestureStylus};
use std::collections::VecDeque;

use crate::strokes::strokestyle::InputData;

/// Map Stylus input to the position on a sheet
pub fn map_inputdata(
    zoom: f64,
    data_entries: &mut VecDeque<InputData>,
    offset: na::Vector2<f64>,
) {
    *data_entries = data_entries
        .iter()
        .map(|inputdata| {
            InputData::new(
                (inputdata.pos() + offset).scale(1.0 / zoom),
                inputdata.pressure(),
            )
        })
        .collect();
}

/// Filter inputdata to sheet bounds
pub fn filter_mapped_inputdata(
    filter_bounds: p2d::bounding_volume::AABB,
    data_entries: &mut VecDeque<InputData>,
) {
    data_entries.retain(|data| filter_bounds.contains_local_point(&na::Point2::from(data.pos())));
}

pub fn retreive_pointer_inputdata(x: f64, y: f64) -> VecDeque<InputData> {
    let mut data_entries: VecDeque<InputData> = VecDeque::with_capacity(1);

    data_entries.push_back(InputData::new(
        na::vector![x, y],
        InputData::PRESSURE_DEFAULT,
    ));
    data_entries
}

/// Retreives available input axes, defaults if not available. X and Y is already available from closure, and should not retreived from .axis() (because of gtk-rs weirdness)
pub fn retreive_stylus_inputdata(
    gesture_stylus: &GestureStylus,
    with_backlog: bool,
    x: f64,
    y: f64,
) -> VecDeque<InputData> {
    let mut data_entries: VecDeque<InputData> = VecDeque::new();

    if with_backlog {
        if let Some(backlog) = gesture_stylus.backlog() {
            for logentry in backlog {
                let axes = logentry.axes();
                let x = axes[1];
                let y = axes[2];
                let pressure = axes[5];
                //log::debug!("{:?}", axes);
                data_entries.push_back(InputData::new(na::vector![x, y], pressure));
            }
        }
    }

    // Get newest data
    let pressure = if let Some(pressure) = gesture_stylus.axis(gdk::AxisUse::Pressure) {
        pressure
    } else {
        InputData::PRESSURE_DEFAULT
    };

    data_entries.push_back(InputData::new(na::vector![x, y], pressure));

    data_entries
}
