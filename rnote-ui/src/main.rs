#![warn(missing_debug_implementations)]
#![allow(clippy::single_match)]
// Hides console window on windows
#![windows_subsystem = "windows"]

mod app;
mod appmenu;
mod appwindow;
mod canvas;
mod canvasmenu;
mod canvaswrapper;
mod colorpicker;
pub(crate) mod config;
pub(crate) mod dialogs;
pub(crate) mod globals;
pub(crate) mod groupediconpicker;
mod iconpicker;
mod mainheader;
mod overlays;
pub(crate) mod penssidebar;
mod settingspanel;
pub(crate) mod strokewidthpicker;
mod unitentry;
pub(crate) mod utils;
mod workspacebrowser;

// Re-exports
pub(crate) use app::RnApp;
pub(crate) use appmenu::RnAppMenu;
pub(crate) use appwindow::RnAppWindow;
pub(crate) use canvas::RnCanvas;
pub(crate) use canvasmenu::RnCanvasMenu;
pub(crate) use canvaswrapper::RnCanvasWrapper;
pub(crate) use colorpicker::RnColorPicker;
pub(crate) use groupediconpicker::RnGroupedIconPicker;
pub(crate) use iconpicker::RnIconPicker;
pub(crate) use mainheader::RnMainHeader;
pub(crate) use overlays::RnOverlays;
pub(crate) use penssidebar::RnPensSideBar;
pub(crate) use settingspanel::RnSettingsPanel;
pub(crate) use strokewidthpicker::RnStrokeWidthPicker;
pub(crate) use unitentry::RnUnitEntry;
pub(crate) use workspacebrowser::RnWorkspaceBrowser;

use gtk4::prelude::*;
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    if let Err(e) = setup_windows_env() {
        eprintln!("failed to setup env for windows, Err: {e:?}");
    }

    let app = RnApp::new();
    app.run();

    Ok(())
}

/// we need to set some env vars on windows
#[cfg(target_os = "windows")]
fn setup_windows_env() -> anyhow::Result<()> {
    use std::path::PathBuf;

    std::env::set_var(
        "XDG_DATA_DIRS",
        PathBuf::from(config::DATADIR).canonicalize()?,
    );
    std::env::set_var(
        "GDK_PIXBUF_MODULEDIR",
        PathBuf::from(config::LIBDIR)
            .canonicalize()?
            .join("/gdk-pixbuf-2.0/2.10.0/loaders"),
    );
    // for debugging
    //std::env::set_var("RUST_LOG", "rnote=debug");
    Ok(())
}
