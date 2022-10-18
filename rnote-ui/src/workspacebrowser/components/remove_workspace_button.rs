use gtk4::{glib, glib::clone, subclass::prelude::ObjectSubclassIsExt, traits::ButtonExt, Button};

use crate::{RnoteAppWindow, WorkspaceBrowser};

pub fn setup_remove_workspace_button(
    workspacebrowser: &WorkspaceBrowser,
    appwindow: &RnoteAppWindow,
) -> Button {
    let remove_workspace_button = workspacebrowser.imp().remove_workspace_button.get();

    remove_workspace_button.connect_clicked(
        clone!(@weak workspacebrowser as wsb, @weak appwindow => move |_| {
            wsb.remove_current_workspace();
        }),
    );

    remove_workspace_button
}
