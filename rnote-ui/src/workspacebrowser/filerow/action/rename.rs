use std::path::{Path, PathBuf};

use gtk4::{
    gio, glib,
    glib::clone,
    pango,
    prelude::FileExt,
    traits::{BoxExt, ButtonExt, EditableExt, PopoverExt, StyleContextExt, WidgetExt},
    Align, Button, Entry, Label, Popover,
};

use gettextrs::gettext;

use crate::{
    workspacebrowser::{widget_helper, FileRow},
    RnoteAppWindow,
};

/// Creates a new `rename` action
pub fn rename(filerow: &FileRow, appwindow: &RnoteAppWindow) -> gio::SimpleAction {
    let rename_action = gio::SimpleAction::new("rename-file", None);

    rename_action.connect_activate(clone!(@weak filerow as filerow, @weak appwindow => move |_action_rename_file, _| {
        if let Some(current_file) = filerow.current_file() {
            if let Some(current_path) = current_file.path() {
                if let Some(parent_path) = current_path.parent().map(|parent_path| parent_path.to_path_buf()) {
                    let entry = create_entry(&current_path);
                    let label = create_label();
                    let (apply_button, popover) = widget_helper::entry_dialog::create_entry_dialog(&entry, &label);

                    filerow.menubutton_box().append(&popover);

                    connect_entry(&entry, &apply_button, parent_path.clone());
                    connect_apply_button(&apply_button, &popover, &entry, parent_path, current_path,
                        current_file, &appwindow);

                    popover.popup();
                }
            }
        }
    }));

    rename_action
}

fn create_entry(current_path: impl AsRef<Path>) -> Entry {
    let entry_name = current_path
        .as_ref()
        .file_name()
        .map(|current_file_name| current_file_name.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from(""));

    Entry::builder().text(entry_name.as_ref()).build()
}

fn create_label() -> Label {
    let label = Label::builder()
        .margin_bottom(12)
        .halign(Align::Center)
        .label(&gettext("Rename"))
        .width_chars(24)
        .ellipsize(pango::EllipsizeMode::End)
        .build();
    label.style_context().add_class("title-4");

    label
}

fn connect_entry(entry: &Entry, apply_button: &Button, parent_path: PathBuf) {
    entry.connect_text_notify(clone!(@weak apply_button => move |entry2| {
        let new_file_path = parent_path.join(&entry2.text());
        let new_file = gio::File::for_path(new_file_path);

        // Disable apply button to prevent overwrites when file already exists
        apply_button.set_sensitive(!new_file.query_exists(None::<&gio::Cancellable>));
    }));
    log::debug!("Connected entry");
}

fn connect_apply_button(
    apply_button: &Button,
    popover: &Popover,
    entry: &Entry,
    parent_path: PathBuf,
    current_path: PathBuf,
    current_file: gio::File,
    appwindow: &RnoteAppWindow,
) {
    apply_button.connect_clicked(clone!(@weak popover, @weak entry, @weak appwindow => move |_| {
        let new_path = parent_path.join(&entry.text());
        let new_file = gio::File::for_path(&new_path);

        if new_file.query_exists(None::<&gio::Cancellable>) {
            // Should have been caught earlier, but making sure
            log::error!("file already exists");
        } else {
            // directory check must happen before moving the file or directory
            let is_directory = current_path.is_dir();

            if let Err(e) = current_file.move_(&new_file, gio::FileCopyFlags::NONE, None::<&gio::Cancellable>, None) {
                log::error!("rename file failed with Err {}", e);
            } else if let Some(current_output_file) = appwindow.canvas().output_file() {
                if is_directory {
                    // if the output file shares a sub-tree with the renamed directory, rename the directory in the output file's path too
                    if let Some(current_output_path) = current_output_file.path() {
                        if current_output_path.starts_with(&current_path) {
                            let directory_index = parent_path.components().count();
                            let new_output_path = new_path.join(current_output_path.components().skip(directory_index + 1).collect::<PathBuf>());

                            appwindow.canvas().set_output_file(Some(gio::File::for_path(new_output_path)));
                        }
                    }
                } else if current_output_file.equal(&current_file) {
                    // if the output file is the current file, change the output file to the renamed file
                    appwindow.canvas().set_output_file(Some(new_file));
                }
            }

            popover.popdown();
        }
    }));

    log::debug!("Connected apply button");
}
