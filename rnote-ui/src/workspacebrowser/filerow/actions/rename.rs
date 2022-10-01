use std::path::PathBuf;

use gtk4::{
    gio, glib,
    glib::clone,
    pango,
    prelude::FileExt,
    traits::{ButtonExt, EditableExt, GridExt, PopoverExt, StyleContextExt, WidgetExt},
    Align, Button, Entry, Grid, Label, Popover, PositionType,
};

use gettextrs::gettext;

use crate::workspacebrowser::FileRow;

impl FileRow {
    pub fn rename_action(&self) -> gio::SimpleAction {
        let rename_action = gio::SimpleAction::new("rename-file", None);

        rename_action.connect_activate(clone!(@weak self as filerow => move |_action_rename_file, _| {
            if let Some(current_file) = filerow.current_file() {
                if let Some(current_path) = current_file.path() {
                    if let Some(parent_path) = current_path.parent().map(|parent_path| parent_path.to_path_buf()) {

                        let entry = get_entry(&current_path);
                        let cancel_button = get_cancel_button();
                        let apply_button = get_apply_button();
                        let label = get_label();

                        let grid = get_grid();
                        grid.attach(&label, 0, 0, 2, 1);
                        grid.attach(&entry, 0, 1, 2, 1);
                        grid.attach(&cancel_button, 0, 2, 1, 1);
                        grid.attach(&apply_button, 1, 2, 1, 1);

                        let popover = get_popover(grid);

                        connect_entry(&entry, &apply_button, parent_path.clone());
                        connect_cancel_button(&cancel_button, &popover);
                        connect_apply_button(&apply_button, &popover, &entry, parent_path.clone(),
                            current_file.clone());

                        popover.popup();
                    }
                }
            }
        }));

        rename_action
    }
}

fn get_entry(current_path: &PathBuf) -> Entry {
    let entry_name = current_path
        .file_name()
        .map(|current_file_name| current_file_name.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from(""));

    Entry::builder().text(entry_name.as_ref()).build()
}

fn get_cancel_button() -> Button {
    Button::builder()
        .halign(Align::Start)
        .label(&gettext("Cancel"))
        .build()
}

fn get_apply_button() -> Button {
    let apply_button = Button::builder()
        .halign(Align::End)
        .label(&gettext("Apply"))
        .build();

    apply_button.style_context().add_class("suggested-action");
    apply_button
}

fn get_label() -> Label {
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

fn get_grid() -> Grid {
    Grid::builder()
        .margin_top(6)
        .margin_bottom(6)
        .column_spacing(18)
        .row_spacing(6)
        .build()
}

fn get_popover(rename_grid: Grid) -> Popover {
    let popover = Popover::builder()
        .autohide(true)
        .has_arrow(true)
        .position(PositionType::Bottom)
        .build();
    popover.set_child(Some(&rename_grid));

    popover
}

fn connect_entry(entry: &Entry, apply_button: &Button, parent_path: PathBuf) {
    entry.connect_text_notify(clone!(@weak apply_button => move |entry| {
        let new_file_path = parent_path.join(entry.text().to_string());
        let new_file = gio::File::for_path(new_file_path);

        // Disable apply button to prevent overwrites when file already exists
        apply_button.set_sensitive(!new_file.query_exists(None::<&gio::Cancellable>));
    }));
}

fn connect_cancel_button(cancel_button: &Button, popover: &Popover) {
    cancel_button.connect_clicked(clone!(@weak popover => move |_| {
        popover.popdown();
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
        let new_file_path = parent_path.join(entry.text().to_string());
        let new_file = gio::File::for_path(new_file_path);

        if new_file.query_exists(None::<&gio::Cancellable>) {
            // Should have been caught earlier, but making sure
            log::error!("file already exists");
        } else {
            if let Err(e) = current_file.move_(&new_file, gio::FileCopyFlags::NONE, None::<&gio::Cancellable>, None) {
                log::error!("rename file failed with Err {}", e);
            }

            popover.popdown();
        }
    }));
}
