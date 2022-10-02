use gtk4::{gio, glib, glib::clone, Entry, Label, Align, traits::{WidgetExt, StyleContextExt}};
use crate::workspacebrowser::FileRow;
use gettextrs::gettext;

impl FileRow {
    pub fn add_dir_action(&self) -> gio::SimpleAction {
        let new_file = gio::SimpleAction::new("add-dir", None);

        new_file.connect_activate(clone!(@weak self as filerow => move |_action_rename_file, _| {
            let entry = get_entry();
            let label = get_label();
        }));

        new_file
    }
}

fn get_entry() -> Entry {
    Entry::builder()
        .build()
}

fn get_label() -> Label {
    let label = Label::builder()
        .margin_bottom(12)
        .halign(Align::Center)
        .label(&gettext("Enter Directory name:"))
        .build();

    label.style_context().add_class("title-4");
    label
}
