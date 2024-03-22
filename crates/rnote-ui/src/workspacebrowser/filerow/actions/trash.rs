// Imports
use crate::workspacebrowser::RnFileRow;
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, prelude::FileExt};

/// Create a new `trash` action.
pub(crate) fn trash(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("trash-file", None);
    action.connect_activate(
        clone!(@weak filerow, @weak appwindow => move |_action_trash_file, _| {
            let Some(current_file) = filerow.current_file() else {
                return;
            };
            current_file.trash_async(
                glib::source::Priority::DEFAULT,
                None::<&gio::Cancellable>,
                clone!(@weak filerow, @strong current_file => move |res| {
                if let Err(e) = res {
                    appwindow.overlays().dispatch_toast_error(&gettext("Trashing file failed"));
                    tracing::debug!("Trash filerow file `{current_file:?}` failed , Err: {e:?}");
                    return;
                }
                filerow.set_current_file(None);
            }));
        }),
    );
    action
}
