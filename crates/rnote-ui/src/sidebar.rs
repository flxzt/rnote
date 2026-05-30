// Imports
use crate::{RnAppMenu, RnAppWindow, RnSettingsPanel, RnWorkspaceBrowser};
use gettextrs::gettext;
use gtk4::{
    Align, Button, CheckButton, CompositeTemplate, Orientation, SelectionMode, ToggleButton,
    Widget, glib, glib::clone, prelude::*, subclass::prelude::*,
};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/github/flxzt/rnote/ui/sidebar.ui")]
    pub(crate) struct RnSidebar {
        #[template_child]
        pub(crate) headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(crate) left_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) right_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) appmenu: TemplateChild<RnAppMenu>,
        #[template_child]
        pub(crate) sidebar_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub(crate) workspacebrowser: TemplateChild<RnWorkspaceBrowser>,
        #[template_child]
        pub(crate) new_layer_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) layers_list: TemplateChild<gtk4::ListBox>,
        #[template_child]
        pub(crate) settings_panel: TemplateChild<RnSettingsPanel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnSidebar {
        const NAME: &'static str = "RnSidebar";
        type Type = super::RnSidebar;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnSidebar {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnSidebar {}
}

glib::wrapper! {
    pub(crate) struct RnSidebar(ObjectSubclass<imp::RnSidebar>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnSidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl RnSidebar {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn headerbar(&self) -> adw::HeaderBar {
        self.imp().headerbar.get()
    }

    pub(crate) fn left_close_button(&self) -> Button {
        self.imp().left_close_button.get()
    }

    pub(crate) fn right_close_button(&self) -> Button {
        self.imp().right_close_button.get()
    }

    pub(crate) fn appmenu(&self) -> RnAppMenu {
        self.imp().appmenu.get()
    }

    pub(crate) fn sidebar_stack(&self) -> adw::ViewStack {
        self.imp().sidebar_stack.get()
    }

    pub(crate) fn workspacebrowser(&self) -> RnWorkspaceBrowser {
        self.imp().workspacebrowser.get()
    }

    pub(crate) fn settings_panel(&self) -> RnSettingsPanel {
        self.imp().settings_panel.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.appmenu.get().init(appwindow);
        imp.workspacebrowser.get().init(appwindow);
        imp.settings_panel.get().init(appwindow);

        imp.new_layer_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                if let Some(canvas) = appwindow.active_tab_canvas() {
                    let widget_flags = canvas.engine_mut().add_layer(None);
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            }
        ));

        imp.left_close_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                appwindow.split_view().set_show_sidebar(false);
            }
        ));
        imp.right_close_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                appwindow.split_view().set_show_sidebar(false);
            }
        ));

        self.refresh_layers_panel(appwindow);
    }

    pub(crate) fn refresh_layers_panel(&self, appwindow: &RnAppWindow) {
        let list = self.imp().layers_list.get();
        list.set_selection_mode(SelectionMode::None);
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let Some(canvas) = appwindow.active_tab_canvas() else {
            return;
        };

        let (layers, active_layer_id) = {
            let engine = canvas.engine_ref();
            (engine.layers().to_vec(), engine.active_layer_id())
        };
        let can_delete = layers.len() > 1;

        let mut first_radio: Option<CheckButton> = None;

        for layer in layers.iter().rev().cloned() {
            let row = gtk4::ListBoxRow::new();
            let row_box = gtk4::Box::new(Orientation::Horizontal, 6);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);
            row_box.set_margin_start(6);
            row_box.set_margin_end(6);

            let active_radio = CheckButton::builder()
                .active(layer.id == active_layer_id)
                .valign(Align::Center)
                .build();
            active_radio.set_tooltip_text(Some(&gettext("Set as active layer")));
            active_radio.set_can_focus(false);
            active_radio.set_focus_on_click(false);
            if let Some(ref group) = first_radio {
                active_radio.set_group(Some(group));
            } else {
                first_radio = Some(active_radio.clone());
            }
            active_radio.connect_toggled(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |radio| {
                    if radio.is_active() {
                        if let Some(canvas) = appwindow.active_tab_canvas() {
                            let widget_flags = canvas.engine_mut().set_active_layer(layer.id);
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                    }
                }
            ));
            row_box.append(&active_radio);

            let name_entry = gtk4::Entry::new();
            name_entry.set_hexpand(true);
            name_entry.set_text(&layer.name);
            name_entry.set_valign(Align::Center);
            name_entry.connect_activate(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |entry| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas
                            .engine_mut()
                            .rename_layer(layer.id, entry.text().to_string());
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            name_entry.connect_has_focus_notify(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |entry| {
                    if entry.has_focus() || entry.text() == layer.name {
                        return;
                    }

                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas
                            .engine_mut()
                            .rename_layer(layer.id, entry.text().to_string());
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&name_entry);

            let visible_button = ToggleButton::builder()
                .icon_name("view-reveal-symbolic")
                .active(layer.visible)
                .valign(Align::Center)
                .build();
            visible_button.add_css_class("flat");
            visible_button.set_tooltip_text(Some(&gettext("Layer visible")));
            visible_button.set_can_focus(false);
            visible_button.set_focus_on_click(false);
            visible_button.connect_toggled(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |button| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas
                            .engine_mut()
                            .set_layer_visible(layer.id, button.is_active());
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&visible_button);

            let lock_button = ToggleButton::builder()
                .icon_name("changes-prevent-symbolic")
                .active(layer.locked)
                .valign(Align::Center)
                .build();
            lock_button.add_css_class("flat");
            lock_button.set_tooltip_text(Some(&gettext("Layer locked")));
            lock_button.set_can_focus(false);
            lock_button.set_focus_on_click(false);
            lock_button.connect_toggled(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |button| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas
                            .engine_mut()
                            .set_layer_locked(layer.id, button.is_active());
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&lock_button);

            let down_button = Button::builder()
                .icon_name("dir-down-symbolic")
                .tooltip_text(gettext("Move layer down"))
                .valign(Align::Center)
                .build();
            down_button.add_css_class("flat");
            down_button.connect_clicked(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |_| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas.engine_mut().move_layer_down(layer.id);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&down_button);

            let up_button = Button::builder()
                .icon_name("dir-up-symbolic")
                .tooltip_text(gettext("Move layer up"))
                .valign(Align::Center)
                .build();
            up_button.add_css_class("flat");
            up_button.connect_clicked(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |_| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas.engine_mut().move_layer_up(layer.id);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&up_button);

            let delete_button = Button::builder()
                .icon_name("selection-trash-symbolic")
                .sensitive(can_delete)
                .tooltip_text(gettext("Delete layer"))
                .valign(Align::Center)
                .build();
            delete_button.add_css_class("flat");
            delete_button.connect_clicked(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |_| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas.engine_mut().delete_layer(layer.id);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&delete_button);

            row.set_child(Some(&row_box));
            list.append(&row);
        }

        list.set_activate_on_single_click(true);
        list.connect_row_activated(clone!(
            #[weak]
            appwindow,
            move |_list, row| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let layers = {
                    let engine = canvas.engine_ref();
                    engine.layers().to_vec()
                };
                let row_idx = row.index() as usize;
                if row_idx < layers.len() {
                    #[allow(clippy::borrow_deref_ref)]
                    let layer_id = layers[layers.len() - 1 - row_idx].id;
                    let active_id = canvas.engine_ref().active_layer_id();
                    if layer_id != active_id {
                        let widget_flags = canvas.engine_mut().set_active_layer(layer_id);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            }
        ));
    }
}
