#![warn(missing_debug_implementations)]

pub mod app;
pub mod appmenu;
pub mod appwindow;
pub mod audioplayer;
pub mod canvas;
pub mod canvasmenu;
pub mod colorpicker;
pub mod config;
pub mod dialogs;
pub mod globals;
pub mod mainheader;
pub mod penssidebar;
pub mod selectionmodifier;
pub mod settingspanel;
pub mod unitentry;
pub mod utils;
pub mod workspacebrowser;

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
