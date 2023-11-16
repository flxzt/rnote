// Imports
use crate::appwindow::RnAppWindow;
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{gio, glib, CompositeTemplate, MenuButton, PopoverMenu, ToggleButton, Widget};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/appmenu.ui")]
    pub(crate) struct RnAppMenu {
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
    impl ObjectSubclass for RnAppMenu {
        const NAME: &'static str = "RnAppMenu";
        type Type = super::RnAppMenu;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnAppMenu {
        fn constructed(&self) {
            self.parent_constructed();

            self.menubutton
                .get()
                .set_popover(Some(&self.popovermenu.get()));
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnAppMenu {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.popovermenu.get().present();
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnAppMenu(ObjectSubclass<imp::RnAppMenu>)
    @extends Widget;
}

impl Default for RnAppMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl RnAppMenu {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn popovermenu(&self) -> PopoverMenu {
        self.imp().popovermenu.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
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
