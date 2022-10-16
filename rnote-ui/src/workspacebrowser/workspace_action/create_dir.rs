use std::path::PathBuf;

use gettextrs::gettext;
use gtk4::{
    gio, glib,
    glib::clone,
    pango,
    prelude::FileExt,
    traits::{BoxExt, ButtonExt, EditableExt, PopoverExt, StyleContextExt, WidgetExt},
    Align, Button, Entry, Label, Popover,
};

use crate::{workspacebrowser::widget_helper, WorkspaceBrowser};

pub fn create_dir(workspacebrowser: &WorkspaceBrowser) -> gio::SimpleAction {
    let new_dir_action = gio::SimpleAction::new("create-dir", None);

    new_dir_action.connect_activate(clone!(@weak workspacebrowser as workspacebrowser => move |_, _| {
        if let Some(parent_path) = workspacebrowser.selected_workspace_dir() {
            let entry = create_entry();
            let label = create_label();
            let (apply_button, popover) = widget_helper::entry_dialog::create_entry_dialog(&entry, &label);

            workspacebrowser.workspace_button_box().append(&popover);

            connect_apply_button(&apply_button, &popover, &entry, parent_path.clone());

            popover.popup();
        }
    }));

    new_dir_action
}

fn create_entry() -> Entry {
    Entry::new()
}

fn create_label() -> Label {
    let label = Label::builder()
        .margin_bottom(12)
        .halign(Align::Center)
        .label(&gettext("New Directory name"))
        .width_chars(24)
        .ellipsize(pango::EllipsizeMode::End)
        .build();
    label.style_context().add_class("title-4");

    label
}

fn connect_apply_button(
    apply_button: &Button,
    popover: &Popover,
    entry: &Entry,
    parent_path: PathBuf,
) {
    apply_button.connect_clicked(clone!(@weak popover, @weak entry => move |_| {
        let new_file_path = parent_path.join(entry.text().to_string());
        let new_file = gio::File::for_path(new_file_path.clone());

        if new_file.query_exists(None::<&gio::Cancellable>) {
            // Should have been caught earlier, but making sure
            log::error!("Directory already exists.");
        } else {
            if let Err(e) = fs_extra::dir::create(new_file_path, false) {
                log::error!("Couldn't create directory: {}", e);
            }

            popover.popdown();
        }
    }));
}
