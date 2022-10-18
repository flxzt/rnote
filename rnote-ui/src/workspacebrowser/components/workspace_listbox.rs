use gtk4::{gio, glib, glib::clone, subclass::prelude::ObjectSubclassIsExt};

use crate::{RnoteAppWindow, WorkspaceBrowser};

pub fn setup_workspace_listbox(workspacebrowser: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    workspacebrowser.imp().workspace_listbox.connect_selected_rows_changed(
        clone!(@weak appwindow, @weak workspacebrowser as wsb => move |_| {
            if let Some(dir) = wsb.current_selected_workspace_row().map(|row| row.entry().dir()) {
                wsb.imp().files_dirlist.set_file(Some(&gio::File::for_path(dir)));
                wsb.save_to_settings(&appwindow.app_settings());
            }
        })
    );
}
