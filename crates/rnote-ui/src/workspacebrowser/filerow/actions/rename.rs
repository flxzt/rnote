// Imports
use crate::workspacebrowser::{widgethelper, RnFileRow};
use crate::RnAppWindow;
use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, pango, prelude::*, Align, Entry, Label};
use std::path::Path;

/// Create a new `rename` action.
pub(crate) fn rename(filerow: &RnFileRow, appwindow: &RnAppWindow) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("rename-file", None);
    action.connect_activate(clone!(@weak filerow, @weak appwindow => move |_, _| {
        let Some(current_file) = filerow.current_file() else {
            return;
        };
        let Some(current_file_path) = current_file.path() else {
            return;
        };
        let Some(parent_path) = current_file_path.parent().map(|p| p.to_path_buf()) else {
            return;
        };
        let entry = create_entry(&current_file_path);
        let label = create_label();
        let (apply_button, popover) = widgethelper::create_entry_dialog(&entry, &label);
        filerow.menubutton_box().append(&popover);

        // Initially the file name is set to the same file name, so set the apply button insensitive first.
        apply_button.set_sensitive(false);

        entry.connect_text_notify(clone!(@strong parent_path, @weak apply_button => move |entry2| {
            let new_file_path = parent_path.join(&entry2.text());
            // Disable apply button to prevent overwrites when file already exists
            apply_button.set_sensitive(!new_file_path.exists());
        }));

        apply_button.connect_clicked(clone!(@weak popover, @weak entry, @weak appwindow => move |_| {
            let new_file_path = parent_path.join(&entry.text());

            if new_file_path.exists() {
                appwindow.overlays().dispatch_toast_error(&gettext("Renaming file failed, target file already exists"));
                tracing::debug!("Renaming file with path '{}' failed, target file already exists", new_file_path.display());
            } else {
                glib::spawn_future_local(clone!(@strong current_file_path, @weak appwindow => async move {
                    appwindow.overlays().progressbar_start_pulsing();
                    if let Err(e) = async_fs::rename(&current_file_path, &new_file_path).await {
                        tracing::error!("Renaming file with path `{}` failed, Err: {e:?}", new_file_path.display());
                        appwindow.overlays().dispatch_toast_error(&gettext("Renaming file failed"));
                        appwindow.overlays().progressbar_abort();
                    } else {
                        appwindow.overlays().progressbar_finish();
                    }
                }));
            }
            popover.popdown();
        }));

        popover.popup();
        entry_text_select_stem(&entry);
    }));
    action
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
