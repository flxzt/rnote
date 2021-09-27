#![warn(missing_debug_implementations)]
pub mod app;
pub mod config;
pub mod globals;
pub mod pens;
pub mod sheet;
pub mod strokes;
pub mod ui;
pub mod utils;

use gtk4::prelude::*;
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

/*
Conventions:
Coordinates in 2d space: origin is thought of in top-left corner of the screen.
Vectors / Matrices in 2D space:
    Vector2: first element is the x-axis, second element is the y-axis
    Matrix2: representing bounds / a rectangle, the coordinate (0,0) is the x-axis of the upper-left corner, (0,1) is the y-axis of the upper-left corner,
        (1,0) is the x-axis of the bottom-right corner, (1,1) is the y-axis of the bottom-right corner.
*/

fn main() {
    pretty_env_logger::init();
    log::info!("... env_logger initialized");

    let app = app::RnoteApp::new();
    app.run();
}
