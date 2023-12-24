// Imports
use crate::workspacebrowser::RnFileRow;
use gtk4::{gio, glib, glib::clone, prelude::FileExt};

/// Create a new `trash` action.
pub(crate) fn trash(filerow: &RnFileRow) -> gio::SimpleAction {
    let action_trash_file = gio::SimpleAction::new("trash-file", None);
    action_trash_file.connect_activate(clone!(@weak filerow => move |_action_trash_file, _| {
        if let Some(current_file) = filerow.current_file() {
            current_file.trash_async(glib::source::Priority::DEFAULT, None::<&gio::Cancellable>, clone!(@weak filerow, @weak current_file => move |res| {
                if let Err(e) = res {
                    tracing::error!("Trash filerow file `{current_file:?}` failed , Err: {e:?}");
                } else {
                    filerow.set_current_file(None);
                }
            }));
        }
    }));

    action_trash_file
}
