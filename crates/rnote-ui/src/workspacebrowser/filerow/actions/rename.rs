// Imports
use crate::workspacebrowser::{widgethelper, RnFileRow};
use gettextrs::gettext;
use gtk4::{
    gio, glib,
    glib::clone,
    pango,
    prelude::FileExt,
    traits::{BoxExt, ButtonExt, EditableExt, PopoverExt, WidgetExt},
    Align, Button, Entry, Label, Popover,
};
use std::path::{Path, PathBuf};

/// Create a new `rename` action.
pub(crate) fn rename(filerow: &RnFileRow) -> gio::SimpleAction {
    let rename_action = gio::SimpleAction::new("rename-file", None);

    rename_action.connect_activate(clone!(@weak filerow => move |_action_rename_file, _| {
        let Some(current_file) = filerow.current_file() else {
            return;
        };
        let Some(current_path) = current_file.path() else {
            return;
        };
        if let Some(parent_path) = current_path.parent().map(|parent_path| parent_path.to_path_buf()) {
            let entry = create_entry(&current_path);
            let label = create_label();
            let (apply_button, popover) = widgethelper::create_entry_dialog(&entry, &label);

            filerow.menubutton_box().append(&popover);

            connect_entry(&entry, &apply_button, parent_path.clone());
            connect_apply_button(&apply_button, &popover, &entry, parent_path, current_file);

            popover.popup();
            entry_text_select_stem(&entry);
        }
    }));

    rename_action
}

fn create_entry(current_path: impl AsRef<Path>) -> Entry {
    let entry_text = current_path
        .as_ref()
        .file_name()
        .map(|current_file_name| current_file_name.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from(""));

    Entry::builder()
        .text(glib::GString::from(entry_text))
        .build()
}

fn entry_text_select_stem(entry: &Entry) {
    let entry_text = entry.text();
    let stem_end = entry_text.match_indices('.').map(|(i, _)| i).last();

    // Select entire text first
    entry.grab_focus();
    if let Some(end) = stem_end {
        // Select only the file stem
        tracing::debug!("file name select end position: {end}");
        entry.select_region(0, end as i32);
    }
}

fn create_label() -> Label {
    let label = Label::builder()
        .margin_bottom(12)
        .halign(Align::Center)
        .label(gettext("Rename"))
        .width_chars(24)
        .ellipsize(pango::EllipsizeMode::End)
        .build();
    label.add_css_class("title-4");
    label
}

fn connect_entry(entry: &Entry, apply_button: &Button, parent_path: PathBuf) {
    entry.connect_text_notify(clone!(@weak apply_button => move |entry2| {
        let new_file_path = parent_path.join(&entry2.text());
        let new_file = gio::File::for_path(new_file_path);

        // Disable apply button to prevent overwrites when file already exists
        apply_button.set_sensitive(!new_file.query_exists(None::<&gio::Cancellable>));
    }));
}

fn connect_apply_button(
    apply_button: &Button,
    popover: &Popover,
    entry: &Entry,
    parent_path: PathBuf,
    current_file: gio::File,
) {
    apply_button.connect_clicked(clone!(@weak popover, @weak entry => move |_| {
        let new_path = parent_path.join(&entry.text());
        let new_file = gio::File::for_path(new_path);

        if new_file.query_exists(None::<&gio::Cancellable>) {
            // Should have been caught earlier, but making sure
            tracing::error!("Renaming file `{new_file:?}` failed, file already exists");
        } else {
            if let Err(e) = current_file.move_(&new_file, gio::FileCopyFlags::NONE, None::<&gio::Cancellable>, None) {
                tracing::error!("Renaming file `{new_file:?}` failed, Err: {e:?}");
            }

            popover.popdown();
        }
    }));
}
