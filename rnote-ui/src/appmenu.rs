use crate::appwindow::RnoteAppWindow;
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{gio, glib, CompositeTemplate, MenuButton, PopoverMenu, ToggleButton, Widget};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/appmenu.ui")]
    pub(crate) struct AppMenu {
        #[template_child]
        pub(crate) menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) popovermenu: TemplateChild<PopoverMenu>,
        #[template_child]
        pub(crate) menu_model: TemplateChild<gio::MenuModel>,
        #[template_child]
        pub(crate) lefthanded_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) righthanded_toggle: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppMenu {
        const NAME: &'static str = "AppMenu";
        type Type = super::AppMenu;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AppMenu {
        fn constructed(&self) {
            self.parent_constructed();

            self.menubutton
                .get()
                .set_popover(Some(&self.popovermenu.get()));
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for AppMenu {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.popovermenu.get().present();
        }
    }
}

glib::wrapper! {
    pub(crate) struct AppMenu(ObjectSubclass<imp::AppMenu>)
    @extends Widget;
}

impl Default for AppMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl AppMenu {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn popovermenu(&self) -> PopoverMenu {
        self.imp().popovermenu.get()
    }

    pub(crate) fn lefthanded_toggle(&self) -> ToggleButton {
        self.imp().lefthanded_toggle.get()
    }

    pub(crate) fn righthanded_toggle(&self) -> ToggleButton {
        self.imp().righthanded_toggle.get()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        self.imp()
            .lefthanded_toggle
            .bind_property("active", appwindow, "righthanded")
            .sync_create()
            .bidirectional()
            .invert_boolean()
            .build();
        self.imp()
            .righthanded_toggle
            .bind_property("active", appwindow, "righthanded")
            .sync_create()
            .bidirectional()
            .build();
    }
}
