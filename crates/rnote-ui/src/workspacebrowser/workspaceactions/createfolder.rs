// Imports
use crate::workspacebrowser::widgethelper;
use crate::{RnAppWindow, RnWorkspaceBrowser};
use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, pango, prelude::*, Align, Entry, Label};

/// Create a new `create_folder` action.
pub(crate) fn create_folder(
    workspacebrowser: &RnWorkspaceBrowser,
    appwindow: &RnAppWindow,
) -> gio::SimpleAction {
    let new_folder_action = gio::SimpleAction::new("create-folder", None);

    new_folder_action.connect_activate(clone!(@weak workspacebrowser, @weak appwindow => move |_, _| {
        if let Some(parent_path) = workspacebrowser.dir_list_file().and_then(|f| f.path()) {
            let folder_name_entry = create_folder_name_entry();
            let dialog_title_label = create_dialog_title_label();
            let (apply_button, popover) = widgethelper::create_entry_dialog(&folder_name_entry, &dialog_title_label);

            // at first don't allow applying, since the user did not enter any text yet.
            apply_button.set_sensitive(false);

            workspacebrowser.dir_controls_actions_box().append(&popover);

            folder_name_entry.connect_changed(clone!(@weak apply_button, @strong parent_path => move |entry| {
                let entry_text = entry.text();
                let new_folder_path = parent_path.join(&entry_text);

                if new_folder_path.exists() || entry_text.is_empty() {
                    apply_button.set_sensitive(false);
                    entry.add_css_class("error");
                } else {
                    // Only allow creating valid folder names
                    apply_button.set_sensitive(true);
                    entry.remove_css_class("error");
                }
            }));

            apply_button.connect_clicked(clone!(@weak popover, @weak folder_name_entry, @weak appwindow => move |_| {
                let new_folder_path = parent_path.join(folder_name_entry.text().as_str());

                if new_folder_path.exists() {
                    // Should have been caught earlier, but making sure
                    appwindow.overlays().dispatch_toast_error("Can't create folder that already exists.");
                    tracing::debug!("Couldn't create new folder wit name `{}`, it already exists.", folder_name_entry.text().as_str());
                } else {
                    if let Err(e) = fs_extra::dir::create(new_folder_path, false) {
                        appwindow.overlays().dispatch_toast_error("Creating new folder failed");
                        tracing::debug!("Couldn't create folder, Err: {e:?}");
                    }

                    popover.popdown();
                }
            }));

            popover.popup();
        } else {
            tracing::warn!("Can't create new folder when there currently is no workspace selected");
        }
    }));

    new_folder_action
}

fn create_folder_name_entry() -> Entry {
    Entry::builder()
        .placeholder_text(gettext("Folder Name"))
        .build()
}

fn create_dialog_title_label() -> Label {
    let label = Label::builder()
        .margin_bottom(12)
        .halign(Align::Center)
        .label(gettext("New Folder"))
        .width_chars(24)
        .ellipsize(pango::EllipsizeMode::End)
        .build();
    label.add_css_class("title-4");
    label
}
