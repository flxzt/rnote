use std::path::PathBuf;

use gtk4::{glib, glib::clone, subclass::prelude::ObjectSubclassIsExt, traits::ButtonExt, Button};

use crate::{RnoteAppWindow, WorkspaceBrowser};

pub fn setup_add_workspace_button(
    workspacebrowser: &WorkspaceBrowser,
    appwindow: &RnoteAppWindow,
) -> Button {
    let add_workspace_button = workspacebrowser.imp().add_workspace_button.get();

    add_workspace_button.connect_clicked(
        clone!(@weak workspacebrowser as wsb, @weak appwindow => move |_add_workspace_button| {
            let dir = wsb.selected_workspace_dir().unwrap_or(PathBuf::from("./"));
            wsb.add_workspace(dir);

            // Popup the edit dialog after creation
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "edit-workspace", None);
        }),
    );

    add_workspace_button
}
