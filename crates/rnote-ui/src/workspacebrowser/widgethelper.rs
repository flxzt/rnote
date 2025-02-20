// Imports
use gettextrs::gettext;
use gtk4::{
    Align, Button, Entry, Grid, Label, Popover, PositionType, glib, glib::clone, prelude::*,
};

/// A template-function to create a simple dialog widget for an action:
///
/// ```text
/// -------------------------
/// |         <Label>       |
/// | <        Entry      > |
/// | <Cancel>      <Apply> |
/// -------------------------
/// ```
///
/// Just create the `apply` button and the label.
/// Everything else is done in this function.
///
/// Only `ApplyButton` and `Popover` are returned because you likely want to
/// apply a connection to them.
pub(crate) fn create_entry_dialog(entry: &Entry, label: &Label) -> (Button, Popover) {
    let grid = Grid::builder()
        .margin_top(6)
        .margin_bottom(6)
        .column_spacing(18)
        .row_spacing(6)
        .build();
    let cancel_button = Button::builder()
        .halign(Align::Start)
        .label(gettext("Cancel"))
        .build();

    let apply_button = Button::builder()
        .halign(Align::End)
        .label(gettext("Apply"))
        .build();
    apply_button.add_css_class("suggested-action");

    grid.attach(label, 0, 0, 2, 1);
    grid.attach(entry, 0, 1, 2, 1);
    grid.attach(&cancel_button, 0, 2, 1, 1);
    grid.attach(&apply_button, 1, 2, 1, 1);

    let popover = Popover::builder()
        .autohide(true)
        .has_arrow(true)
        .position(PositionType::Bottom)
        .build();
    popover.set_child(Some(&grid));

    cancel_button.connect_clicked(clone!(
        #[weak]
        popover,
        move |_| {
            popover.popdown();
        }
    ));

    (apply_button, popover)
}
