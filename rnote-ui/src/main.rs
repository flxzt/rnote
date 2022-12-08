#![warn(missing_debug_implementations)]
#![allow(clippy::single_match)]
// Hides console window on windows
#![windows_subsystem = "windows"]

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

use gtk4::prelude::*;
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

fn main() -> anyhow::Result<()> {
    // we need to set some environment vars on windows
    #[cfg(target_os = "windows")]
    {
        std::env::set_var(
            "GSETTINGS_SCHEMA_DIR",
            config::DATA_DIR.to_string() + "/glib-2.0/schemas",
        );
    }

    let app = RnoteApp::new();
    app.run();

    Ok(())
}
