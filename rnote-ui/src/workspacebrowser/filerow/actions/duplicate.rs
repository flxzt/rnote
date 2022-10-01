use gtk4::prelude::FileExt;
use gtk4::{gio, glib, glib::clone};

use crate::workspacebrowser::FileRow;

impl FileRow {
    pub fn duplicate_action(&self) -> gio::SimpleAction {
        let action = gio::SimpleAction::new("duplicate-file", None);

        action.connect_activate(
            clone!(@weak self as filerow => move |_action_duplicate_file, _| {
                if let Some(current_file) = filerow.current_file() {
                    if let Some(current_path) = current_file.path() {
                        let path = current_path.into_boxed_path();

                        if path.is_dir() {
                            duplicate_file();
                        } else {
                            duplicate_dir();
                        }
                    }
                }
            }),
        );

        action
    }
}

fn duplicate_file() {}

fn duplicate_dir() {}
