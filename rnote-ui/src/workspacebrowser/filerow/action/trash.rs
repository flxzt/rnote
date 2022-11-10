use gtk4::{gio, glib, glib::clone, prelude::FileExt};

use crate::{workspacebrowser::FileRow, RnoteAppWindow};

/// Creates a new `trash` action
pub fn trash(filerow: &FileRow, appwindow: &RnoteAppWindow) -> gio::SimpleAction {
    let action_trash_file = gio::SimpleAction::new("trash-file", None);
    action_trash_file.connect_activate(clone!(@weak filerow, @weak appwindow => move |_action_trash_file, _| {
        if let Some(current_file) = filerow.current_file() {
            // directory check must happen before deleting the file or directory
            let current_path = current_file.path().unwrap();
            let is_directory = current_path.is_dir();

            current_file.trash_async(glib::PRIORITY_DEFAULT, None::<&gio::Cancellable>, clone!(@weak filerow, @weak current_file => move |res| {
                if let Err(e) = res {
                    log::error!("filerow trash file failed with Err {}", e);
                } else {
                    filerow.set_current_file(None);

                    if let Some(current_output_file) = appwindow.canvas().output_file() {
                        if is_directory {
                            // if the output file shares a sub-tree with the deleted directory, the output file has been deleted too and gets unset
                            if let Some(current_output_path) = current_output_file.path() {
                                if current_output_path.starts_with(&current_path) {
                                    appwindow.canvas().set_output_file(None);
                                }
                            }
                        } else if current_output_file.equal(&current_file) {
                            // if the output file is the current file, unset the output file
                            appwindow.canvas().set_output_file(None);
                        }
                    }
                }
            }));
        }
    }));

    action_trash_file
}
