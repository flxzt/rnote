#![warn(missing_debug_implementations)]
#![allow(clippy::single_match)]

pub(crate) mod config;
pub(crate) mod dialogs;
pub(crate) mod globals;
pub(crate) mod utils;

/// Widgets
mod app;
mod appmenu;
mod appwindow;
mod canvas;
mod canvasmenu;
mod canvaswrapper;
mod colorpicker;
mod iconpicker;
mod mainheader;
pub(crate) mod penssidebar;
mod settingspanel;
mod unitentry;
mod workspacebrowser;

// Re-exports
pub(crate) use app::RnoteApp;
pub(crate) use appmenu::AppMenu;
pub(crate) use appwindow::RnoteAppWindow;
pub(crate) use canvas::RnoteCanvas;
pub(crate) use canvasmenu::CanvasMenu;
pub(crate) use canvaswrapper::RnoteCanvasWrapper;
pub(crate) use colorpicker::ColorPicker;
pub(crate) use iconpicker::IconPicker;
pub(crate) use mainheader::MainHeader;
pub(crate) use penssidebar::PensSideBar;
pub(crate) use settingspanel::SettingsPanel;
pub(crate) use unitentry::UnitEntry;
pub(crate) use workspacebrowser::WorkspaceBrowser;

use gettextrs::LocaleCategory;
use gtk4::prelude::*;
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

use self::config::{GETTEXT_PACKAGE, LOCALEDIR};

fn main() {
    pretty_env_logger::init();
    log::debug!("... env_logger initialized");

    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let app = RnoteApp::new();
    app.run();
}
