// Imports
use crate::{RnAppMenu, RnAppWindow, RnSettingsPanel, RnWorkspaceBrowser};
use gettextrs::gettext;
use gtk4::{
    Align, Button, CompositeTemplate, Orientation, Widget, glib, glib::clone, prelude::*,
    subclass::prelude::*,
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

        for layer in layers.iter().rev().cloned() {
            let row = gtk4::ListBoxRow::new();
            let row_box = gtk4::Box::new(Orientation::Horizontal, 6);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);
            row_box.set_margin_start(6);
            row_box.set_margin_end(6);

            let active_button = Button::with_label(if layer.id == active_layer_id {
                "*"
            } else {
                " "
            });
            active_button.set_tooltip_text(Some(&gettext("Set as active layer")));
            active_button.set_valign(Align::Center);
            active_button.connect_clicked(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |_| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas.engine_mut().set_active_layer(layer.id);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&active_button);

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

            let visible_switch = gtk4::Switch::builder()
                .active(layer.visible)
                .valign(Align::Center)
                .build();
            visible_switch.set_tooltip_text(Some(&gettext("Layer visible")));
            visible_switch.connect_active_notify(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |switch| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas
                            .engine_mut()
                            .set_layer_visible(layer.id, switch.is_active());
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&visible_switch);

            let lock_switch = gtk4::Switch::builder()
                .active(layer.locked)
                .valign(Align::Center)
                .build();
            lock_switch.set_tooltip_text(Some(&gettext("Layer locked")));
            lock_switch.connect_active_notify(clone!(
                #[weak]
                appwindow,
                #[strong]
                layer,
                move |switch| {
                    if let Some(canvas) = appwindow.active_tab_canvas() {
                        let widget_flags = canvas
                            .engine_mut()
                            .set_layer_locked(layer.id, switch.is_active());
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));
            row_box.append(&lock_switch);

            let down_button = Button::with_label("v");
            down_button.set_tooltip_text(Some(&gettext("Move layer down")));
            down_button.set_valign(Align::Center);
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

            let up_button = Button::with_label("^");
            up_button.set_tooltip_text(Some(&gettext("Move layer up")));
            up_button.set_valign(Align::Center);
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

            let delete_button = Button::with_label("-");
            delete_button.set_sensitive(can_delete);
            delete_button.set_tooltip_text(Some(&gettext("Delete layer")));
            delete_button.set_valign(Align::Center);
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
    }
}
