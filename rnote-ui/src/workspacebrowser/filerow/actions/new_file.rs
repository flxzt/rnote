use gtk4::{gio, glib, glib::clone};

use crate::workspacebrowser::FileRow;

impl FileRow {
    pub fn new_file_action(&self) -> gio::SimpleAction {
        let new_file = gio::SimpleAction::new("new-file", None);
        new_file
    }
}
