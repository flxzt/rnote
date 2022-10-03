use std::path::PathBuf;

use gtk4::{
    gio, glib,
    glib::clone,
    pango,
    prelude::FileExt,
    traits::{ButtonExt, EditableExt, PopoverExt, StyleContextExt, WidgetExt, GridExt, BoxExt},
    Align, Button, Entry, Label, Popover, Grid, PositionType,
};

use gettextrs::gettext;

use crate::workspacebrowser::{filerow::widget_helper, FileRow};

impl FileRow {
    pub fn rename_action(&self) -> gio::SimpleAction {
        let action_rename_file = gio::SimpleAction::new("rename-file", None);

        action_rename_file.connect_activate(clone!(@weak self as filerow => move |_action_rename_file, _| {
                if let Some(current_file) = filerow.current_file() {
                    if let Some(current_path) = current_file.path() {
                        if let Some(parent_path) = current_path.parent().map(|parent_path| parent_path.to_path_buf()) {
                            let current_name = current_path.file_name().map(|current_file_name| current_file_name.to_string_lossy().to_string()).unwrap_or_else(|| String::from(""));

                            let rename_entry = Entry::builder()
                                .text(current_name.as_str())
                                .build();

                            let rename_cancel_button = Button::builder().halign(Align::Start).label(&gettext("Cancel")).build();

                            let rename_apply_button = Button::builder().halign(Align::End).label(&gettext("Apply")).build();
                            rename_apply_button.style_context().add_class("suggested-action");

                            let rename_label = Label::builder().margin_bottom(12).halign(Align::Center).label(&gettext("Rename")).width_chars(24).ellipsize(pango::EllipsizeMode::End).build();
                            rename_label.style_context().add_class("title-4");

                            let rename_grid = Grid::builder().margin_top(6).margin_bottom(6).column_spacing(18).row_spacing(6).build();
                            rename_grid.attach(&rename_label, 0, 0, 2, 1);
                            rename_grid.attach(&rename_entry, 0, 1, 2, 1);
                            rename_grid.attach(&rename_cancel_button, 0, 2, 1, 1);
                            rename_grid.attach(&rename_apply_button, 1, 2, 1, 1);

                            let rename_popover = Popover::builder().autohide(true).has_arrow(true).position(PositionType::Bottom).build();
                            rename_popover.set_child(Some(&rename_grid));
                            filerow.menubutton_box().append(&rename_popover);

                            let parent_path_1 = parent_path.clone();
                            rename_entry.connect_text_notify(clone!(@weak rename_apply_button => move |rename_entry| {
                                let new_file_path = parent_path_1.join(rename_entry.text().to_string());
                                let new_file = gio::File::for_path(new_file_path);

                                // Disable apply button to prevent overwrites when file already exists
                                rename_apply_button.set_sensitive(!new_file.query_exists(None::<&gio::Cancellable>));
                            }));

                            rename_cancel_button.connect_clicked(clone!(@weak rename_popover => move |_| {
                                rename_popover.popdown();
                            }));

                            rename_apply_button.connect_clicked(clone!(@weak rename_popover, @weak rename_entry => move |_| {
                                let new_file_path = parent_path.join(rename_entry.text().to_string());
                                let new_file = gio::File::for_path(new_file_path);

                                if new_file.query_exists(None::<&gio::Cancellable>) {
                                    // Should have been caught earlier, but making sure
                                    log::error!("file already exists");
                                } else {
                                    if let Err(e) = current_file.move_(&new_file, gio::FileCopyFlags::NONE, None::<&gio::Cancellable>, None) {
                                        log::error!("rename file failed with Err {}", e);
                                    }

                                    rename_popover.popdown();
                                }
                            }));

                            rename_popover.popup();
                        }
                    }
                }
            }));

        action_rename_file
    }
}

// impl FileRow {
//     pub fn rename_action(&self) -> gio::SimpleAction {
//         let rename_action = gio::SimpleAction::new("rename-file", None);
//
//         rename_action.connect_activate(clone!(@weak self as filerow => move |_action_rename_file, _| {
//             if let Some(current_file) = filerow.current_file() {
//                 if let Some(current_path) = current_file.path() {
//                     if let Some(parent_path) = current_path.parent().map(|parent_path| parent_path.to_path_buf()) {
//
//                         let entry = get_entry(&current_path);
//                         let label = get_label();
//
//                         let (grid, cancel_button, apply_button, popover) = widget_helper::entry_dialog::get_entry_dialog(&entry, &label);
//
//                         connect_entry(&entry, &apply_button, parent_path.clone());
//                         connect_apply_button(&apply_button, &popover, &entry, parent_path.clone(),
//                             current_file.clone());
//
//                         log::debug!("Start popup");
//                         popover.popup();
//                         log::debug!("yeet");
//                     }
//                 }
//             }
//         }));
//
//         rename_action
//     }
// }

fn get_entry(current_path: &PathBuf) -> Entry {
    let entry_name = current_path
        .file_name()
        .map(|current_file_name| current_file_name.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from(""));

    Entry::builder().text(entry_name.as_ref()).build()
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

fn connect_entry(entry: &Entry, apply_button: &Button, parent_path: PathBuf) {
    entry.connect_text_notify(clone!(@weak apply_button => move |entry2| {
        let new_file_path = parent_path.join(entry2.text().to_string());
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

    log::debug!("Connected apply button");
}
