use crate::{RnAppWindow, RnWorkspaceBrowser};
use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, prelude::*};

pub(crate) fn open_folder(
    workspacebrowser: &RnWorkspaceBrowser,
    appwindow: &RnAppWindow,
) -> gio::SimpleAction {
    let open_folder_action = gio::SimpleAction::new("open-folder", None);

    open_folder_action.connect_activate(clone!(
        #[weak]
        workspacebrowser,
        #[weak]
        appwindow,
        move |_, _| {
        if let Some(parent_path) = workspacebrowser.dir_list_file().and_then(|f| f.path()) {
            if let Err(e) = open::that(&parent_path) {
                let path_string =   &parent_path.into_os_string().into_string().ok().unwrap_or(String::from("Failed to get the path of the workspace folder"));
                tracing::error!("Opening the parent folder '{path_string}' in the file manager failed, Err: {e:?}");
                appwindow.overlays().dispatch_toast_error(&gettext("Failed to open the file in the file manager"));
            }
        } else {
            tracing::warn!("No path found");
            appwindow.overlays().dispatch_toast_error(&gettext("Failed to open the file in the file manager"));
        }
    }
    ));

    open_folder_action
}
