use crate::workspacebrowser::FileRow;
use gettextrs::gettext;
use gtk4::{
    gio, glib,
    glib::clone,
    traits::{StyleContextExt, WidgetExt, EditableExt},
    Align, Entry, Label, pango, Button,
};

impl FileRow {
    pub fn add_dir_action(&self) -> gio::SimpleAction {
        let new_file = gio::SimpleAction::new("add-dir", None);

        new_file.connect_activate(clone!(@weak self as filerow => move |_action_rename_file, _| {
            let entry = get_entry();
            let label = get_label();

            let (cancel_button, apply_button, popover) = self.get_entry_box_dialog_action(entry, label);
        }));

        new_file
    }
}

fn get_entry() -> Entry {
    Entry::new()
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

fn connect_entry(entry: &Entry, apply_button: &Button) {
    entry.connect_text_notify(clone!(@weak apply_button => move |entry| {
        println!("Connect entry");
    }));
}
