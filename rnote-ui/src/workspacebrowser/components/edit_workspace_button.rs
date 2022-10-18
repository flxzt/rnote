use gtk4::{glib, glib::clone, subclass::prelude::ObjectSubclassIsExt, traits::ButtonExt, Button};

use crate::{RnoteAppWindow, WorkspaceBrowser};

pub fn setup_edit_workspace_button(
    workspacebrowser: &WorkspaceBrowser,
    appwindow: &RnoteAppWindow,
) -> Button {
    let edit_workspace_button = workspacebrowser.imp().edit_workspace_button.get();

    edit_workspace_button.connect_clicked(clone!(@weak appwindow => move |_| {
        adw::prelude::ActionGroupExt::activate_action(&appwindow, "edit-workspace", None);
    }));

    edit_workspace_button
}
