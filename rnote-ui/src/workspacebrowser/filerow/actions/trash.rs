use gtk4::{gio, glib, glib::clone, prelude::FileExt};

use crate::workspacebrowser::FileRow;

impl FileRow {
    pub fn trash_action(&self) -> gio::SimpleAction {
        let action_trash_file = gio::SimpleAction::new("trash-file", None);
        action_trash_file.connect_activate(clone!(@weak self as filerow => move |_action_trash_file, _| {
            if let Some(current_file) = filerow.current_file() {
                current_file.trash_async(glib::PRIORITY_DEFAULT, None::<&gio::Cancellable>, clone!(@weak filerow => move |res| {
                    if let Err(e) = res {
                        log::error!("filerow trash file failed with Err {}", e);
                    } else {
                        filerow.set_current_file(None);
                    }
                }));
            }
        }));

        action_trash_file
    }
}
