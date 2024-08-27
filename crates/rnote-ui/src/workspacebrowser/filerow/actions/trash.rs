// Imports
use crate::RnAppWindow;
use crate::{dialogs, workspacebrowser::RnFileRow};
use gtk4::{gio, glib, glib::clone};

/// Create a new `trash` action.
pub(crate) fn trash(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("trash-file", None);
    action.connect_activate(
        clone!(@weak filerow, @weak appwindow => move |_action_trash_file, _| {
            glib::spawn_future_local(clone!(@weak appwindow => async move {
                dialogs::dialog_trash_file(&appwindow, &filerow).await;
            }));
        }),
    );
    action
}
