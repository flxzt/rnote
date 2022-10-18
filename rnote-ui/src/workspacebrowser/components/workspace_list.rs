use gtk4::{
    glib, glib::clone, prelude::ListModelExt, subclass::prelude::ObjectSubclassIsExt,
    traits::WidgetExt, Button,
};

use crate::{RnoteAppWindow, WorkspaceBrowser};

pub fn setup_workspace_list(
    workspacebrowser: &WorkspaceBrowser,
    appwindow: &RnoteAppWindow,
    remove_workspace_button: &Button,
    edit_workspace_button: &Button,
) {
    workspacebrowser.imp().workspace_list.connect_items_changed(
        clone!(@weak workspacebrowser as wsb, @weak appwindow, @weak remove_workspace_button as rwb, @weak edit_workspace_button as ewb => move |folders_model, _, _, _| {
            rwb.set_sensitive(folders_model.n_items() > 1);
            ewb.set_sensitive(folders_model.n_items() > 0);
            wsb.save_to_settings(&appwindow.app_settings());
        }),
    );
}
