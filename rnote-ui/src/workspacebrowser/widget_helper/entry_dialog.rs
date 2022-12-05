use gettextrs::gettext;
use gtk4::{
    glib,
    glib::clone,
    traits::{ButtonExt, GridExt, PopoverExt, StyleContextExt, WidgetExt},
    Align, Button, Entry, Grid, Label, Popover, PositionType,
};

pub type ApplyButton = Button;

/// A template-function to create a simple dialog widget for an action:
///
/// -------------------------
/// |         <Label>       |
/// | <        Entry      > |
/// | <Cancel>      <Apply> |
/// -------------------------
///
/// Just create the `apply` button and the label.
/// Everything else is done in this function.
///
/// Only `ApplyButton` and `Popover` are returned because you likely want to
/// apply a connection to them.
pub(crate) fn create_entry_dialog(entry: &Entry, label: &Label) -> (ApplyButton, Popover) {
    let grid = create_grid();
    let cancel_button = create_cancel_button();
    let apply_button = create_apply_button();

    grid.attach(label, 0, 0, 2, 1);
    grid.attach(entry, 0, 1, 2, 1);
    grid.attach(&cancel_button, 0, 2, 1, 1);
    grid.attach(&apply_button, 1, 2, 1, 1);

    let popover = create_popover(&grid);

    connect_cancel_button(&cancel_button, &popover);

    (apply_button, popover)
}

fn create_grid() -> Grid {
    Grid::builder()
        .margin_top(6)
        .margin_bottom(6)
        .column_spacing(18)
        .row_spacing(6)
        .build()
}

fn create_cancel_button() -> Button {
    Button::builder()
        .halign(Align::Start)
        .label(&gettext("Cancel"))
        .build()
}

fn create_apply_button() -> Button {
    let apply_button = Button::builder()
        .halign(Align::End)
        .label(&gettext("Apply"))
        .build();

    apply_button.style_context().add_class("suggested-action");
    apply_button
}

fn create_popover(grid: &Grid) -> Popover {
    let popover = Popover::builder()
        .autohide(true)
        .has_arrow(true)
        .position(PositionType::Bottom)
        .build();
    popover.set_child(Some(grid));

    popover
}

fn connect_cancel_button(cancel_button: &Button, popover: &Popover) {
    cancel_button.connect_clicked(clone!(@weak popover => move |_| {
        popover.popdown();
    }));
}
