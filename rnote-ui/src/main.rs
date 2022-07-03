#![warn(missing_debug_implementations)]
#![allow(clippy::single_match)]

pub mod config;
pub mod dialogs;
pub mod globals;
pub mod utils;

/// Widgets
mod app;
mod appmenu;
mod appwindow;
mod canvas;
mod canvasmenu;
mod colorpicker;
mod mainheader;
pub mod penssidebar;
mod settingspanel;
mod unitentry;
mod iconpicker;
mod workspacebrowser;

// Re-exports
pub use app::RnoteApp;
pub use appmenu::AppMenu;
pub use appwindow::RnoteAppWindow;
pub use canvas::RnoteCanvas;
pub use canvasmenu::CanvasMenu;
pub use colorpicker::ColorPicker;
pub use mainheader::MainHeader;
pub use penssidebar::PensSideBar;
pub use settingspanel::SettingsPanel;
pub use unitentry::UnitEntry;
pub use iconpicker::IconPicker;
pub use workspacebrowser::WorkspaceBrowser;

use gettextrs::LocaleCategory;
use gtk4::prelude::*;
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
