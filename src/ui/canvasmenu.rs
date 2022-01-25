mod imp {
    use gtk4::{
        gio::MenuModel, glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate,
        MenuButton, PopoverMenu, ToggleButton,
    };

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/canvasmenu.ui")]
    pub struct CanvasMenu {
        #[template_child]
        pub menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub popovermenu: TemplateChild<PopoverMenu>,
        #[template_child]
        pub menu_model: TemplateChild<MenuModel>,
        #[template_child]
        pub zoom_in_button: TemplateChild<Button>,
        #[template_child]
        pub zoom_out_button: TemplateChild<Button>,
        #[template_child]
        pub zoom_reset_button: TemplateChild<Button>,
        #[template_child]
        pub zoom_fit_width_button: TemplateChild<Button>,
        #[template_child]
        pub lefthanded_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub righthanded_toggle: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasMenu {
        const NAME: &'static str = "CanvasMenu";
        type Type = super::CanvasMenu;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CanvasMenu {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.menubutton
                .get()
                .set_popover(Some(&self.popovermenu.get()));
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for CanvasMenu {
        fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(widget, width, height, baseline);
            self.popovermenu.get().present();
        }
    }
}

use crate::ui::appwindow::RnoteAppWindow;

use gtk4::{gio, MenuButton, PopoverMenu, Widget};
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, Button, ToggleButton};

glib::wrapper! {
    pub struct CanvasMenu(ObjectSubclass<imp::CanvasMenu>)
    @extends Widget;
}

impl Default for CanvasMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasMenu {
    pub fn new() -> Self {
        let canvasmenu: CanvasMenu = glib::Object::new(&[]).expect("Failed to create CanvasMenu");
        canvasmenu
    }

    pub fn menubutton(&self) -> MenuButton {
        imp::CanvasMenu::from_instance(self).menubutton.get()
    }

    pub fn popovermenu(&self) -> PopoverMenu {
        imp::CanvasMenu::from_instance(self).popovermenu.get()
    }

    pub fn menu_model(&self) -> gio::MenuModel {
        imp::CanvasMenu::from_instance(self).menu_model.get()
    }

    pub fn zoomin_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self).zoom_in_button.get()
    }

    pub fn zoomout_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self).zoom_out_button.get()
    }

    pub fn zoomreset_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self).zoom_reset_button.get()
    }

    pub fn zoom_fit_width_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self)
            .zoom_fit_width_button
            .get()
    }

    pub fn lefthanded_toggle(&self) -> ToggleButton {
        imp::CanvasMenu::from_instance(self).lefthanded_toggle.get()
    }

    pub fn righthanded_toggle(&self) -> ToggleButton {
        imp::CanvasMenu::from_instance(self)
            .righthanded_toggle
            .get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let zoomreset_button = self.imp().zoom_reset_button.get();

        self.imp().righthanded_toggle.connect_toggled(clone!(@weak appwindow => move |righthanded_toggle| {
            appwindow.application().unwrap().change_action_state("righthanded", &righthanded_toggle.is_active().to_variant());
        }));

        self.imp().zoom_fit_width_button.connect_clicked(
            clone!(@weak appwindow => move |_zoom_fit_width_button| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-fit-width", None);
            }),
        );

        self.imp().zoom_reset_button.connect_clicked(
            clone!(@weak appwindow => move |_zoomreset_button| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-reset", None);
            }),
        );

        self.imp().zoom_in_button.connect_clicked(
            clone!(@weak appwindow, @weak zoomreset_button => move |_zoom_in_button| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-in", None);
            }),
        );

        self.imp().zoom_out_button.connect_clicked(
            clone!(@weak appwindow, @weak zoomreset_button => move |_zoom_out_button| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-out", None);
            }),
        );
    }
}
