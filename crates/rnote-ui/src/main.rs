#![warn(missing_debug_implementations)]
#![allow(clippy::single_match)]
// Hides console window on windows
#![windows_subsystem = "windows"]

// Modules
pub(crate) mod app;
pub(crate) mod appmenu;
pub(crate) mod appwindow;
pub(crate) mod canvas;
pub(crate) mod canvasmenu;
pub(crate) mod canvaswrapper;
pub(crate) mod colorpicker;
pub(crate) mod config;
pub(crate) mod dialogs;
pub(crate) mod env;
pub(crate) mod filetype;
pub(crate) mod globals;
pub(crate) mod groupediconpicker;
pub(crate) mod iconpicker;
pub(crate) mod mainheader;
pub(crate) mod overlays;
pub(crate) mod penssidebar;
pub(crate) mod settingspanel;
pub(crate) mod sidebar;
pub(crate) mod strokecontentpaintable;
pub(crate) mod strokecontentpreview;
pub(crate) mod strokewidthpicker;
pub(crate) mod unitentry;
pub(crate) mod utils;
pub(crate) mod workspacebrowser;

// Re-exports
pub(crate) use app::RnApp;
pub(crate) use appmenu::RnAppMenu;
pub(crate) use appwindow::RnAppWindow;
pub(crate) use canvas::RnCanvas;
pub(crate) use canvasmenu::RnCanvasMenu;
pub(crate) use canvaswrapper::RnCanvasWrapper;
pub(crate) use colorpicker::RnColorPicker;
pub(crate) use filetype::FileType;
pub(crate) use groupediconpicker::RnGroupedIconPicker;
pub(crate) use iconpicker::RnIconPicker;
pub(crate) use mainheader::RnMainHeader;
pub(crate) use overlays::RnOverlays;
pub(crate) use penssidebar::RnPensSideBar;
pub(crate) use settingspanel::RnSettingsPanel;
pub(crate) use sidebar::RnSidebar;
pub(crate) use strokecontentpaintable::StrokeContentPaintable;
pub(crate) use strokecontentpreview::RnStrokeContentPreview;
pub(crate) use strokewidthpicker::RnStrokeWidthPicker;
pub(crate) use unitentry::RnUnitEntry;
pub(crate) use workspacebrowser::RnWorkspaceBrowser;

// Renames
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

// Imports
use gtk4::{glib, prelude::*};

fn main() -> glib::ExitCode {
    if let Err(e) = setup_logging() {
        eprintln!("failed to setup logging, Err: {e:?}");
    }

    if let Err(e) = env::setup_env() {
        eprintln!("failed to setup env, Err: {e:?}");
    }

    if let Err(e) = setup_i18n() {
        eprintln!("failed to setup i18n, Err: {e:?}");
    }

    let app = RnApp::new();
    app.run()
}

fn setup_logging() -> anyhow::Result<()> {
    pretty_env_logger::try_init_timed()?;
    log::debug!("... env_logger initialized");
    Ok(())
}

fn setup_i18n() -> anyhow::Result<()> {
    let locale_dir = env::locale_dir()?;

    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(config::GETTEXT_PACKAGE, locale_dir)?;
    gettextrs::bind_textdomain_codeset(config::GETTEXT_PACKAGE, "UTF-8")?;
    gettextrs::textdomain(config::GETTEXT_PACKAGE)?;
    Ok(())
}
