// Imports
use crate::workspacebrowser::RnFileRow;
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gio, gio::prelude::FileExt, glib, glib::clone};

/// Create a new `open-in-default` action.
pub(crate) fn open_in_default(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action_open_in_default = gio::SimpleAction::new("open-in-default", None);
    action_open_in_default.connect_activate(
        clone!(@weak filerow, @weak appwindow => move |_action_open_in_default, _| {
            if let Some(current_file) = filerow.current_file() {
                if let Err(e) =  open::that(current_file.uri()) {
                    appwindow.overlays().dispatch_toast_error(&gettext("Failed to open the file in the default app"));
                    log::debug!("opening file {} with default app failed: {e:?}", current_file.uri());
                }
            }
        }),
    );

    action_open_in_default
}
