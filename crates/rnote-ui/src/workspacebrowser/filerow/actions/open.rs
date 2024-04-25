// Imports
use crate::workspacebrowser::RnFileRow;
use crate::RnAppWindow;
use gtk4::{gio, glib, glib::clone};

/// Create a new `open` action.
pub(crate) fn open(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("open-file", None);
    action.connect_activate(clone!(@weak filerow, @weak appwindow => move |_, _| {
        let Some(current_file) = filerow.current_file() else {
            return
        };
        glib::spawn_future_local(clone!(@weak appwindow => async move {
            tracing::debug!("open action in the workspace bar");
            appwindow.open_file_w_dialogs(current_file, None, true, appwindow.active_tab_wrapper().respect_borders()).await;
        }));
    }));
    action
}
