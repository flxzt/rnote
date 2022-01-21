#![warn(missing_debug_implementations)]
#![allow(dead_code)]

pub mod app;
pub mod audioplayer;
pub mod compose;
pub mod config;
pub mod drawbehaviour;
pub mod globals;
pub mod input;
pub mod pens;
pub mod render;
pub mod sheet;
pub mod strokes;
pub mod strokesstate;
pub mod ui;
pub mod utils;

use gettextrs::LocaleCategory;
use gtk4::prelude::*;
extern crate gstreamer as gst;
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

use self::config::{GETTEXT_PACKAGE, LOCALEDIR};

fn main() {
    pretty_env_logger::init();
    log::info!("... env_logger initialized");

    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let app = app::RnoteApp::new();
    app.run();
}
