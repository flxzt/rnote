#![warn(missing_debug_implementations)]
pub mod app;
pub mod compose;
pub mod config;
pub mod curves;
pub mod globals;
pub mod input;
pub mod pens;
pub mod render;
pub mod sheet;
pub mod strokes;
pub mod ui;
pub mod utils;

use gtk4::prelude::*;
extern crate nalgebra as na;
extern crate nalgebra_glm as glm;
extern crate parry2d_f64 as p2d;

fn main() {
    pretty_env_logger::init();
    log::info!("... env_logger initialized");

    let app = app::RnoteApp::new();
    app.run();
}
