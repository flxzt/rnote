use gtk4::{gio, glib, glib::clone};

use crate::workspacebrowser::FileRow;
use crate::RnoteAppWindow;

impl FileRow {
    pub fn open_action(&self, appwindow: &RnoteAppWindow) -> gio::SimpleAction {

        let action_open_file = gio::SimpleAction::new("open-file", None);
        action_open_file.connect_activate(
            clone!(@weak self as filerow, @weak appwindow => move |_action_open_file, _| {
                    if let Some(current_file) = filerow.current_file() {
                        appwindow.open_file_w_dialogs(&current_file, None);
                    }
            }),
        );

        action_open_file
    }
}
