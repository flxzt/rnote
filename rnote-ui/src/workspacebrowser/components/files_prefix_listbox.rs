use gtk4::{glib, glib::clone, subclass::prelude::ObjectSubclassIsExt};

use crate::{RnoteAppWindow, WorkspaceBrowser};

pub fn setup_files_prefix_listbox(workspacebrowser: &WorkspaceBrowser, appwindow: &RnoteAppWindow) {
    workspacebrowser.imp().files_prefix_listbox.connect_row_activated(
        clone!(@weak workspacebrowser, @weak appwindow => move |_, row| {
            if row == &workspacebrowser.imp().dir_up_row.get() {
                if let Some(parent_dir) = workspacebrowser.selected_workspace_dir().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
                    workspacebrowser.set_current_workspace_dir(parent_dir.to_path_buf());
                }

            }
        })
    );
}
