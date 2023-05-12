// Imports
use crate::appwindow::RnAppWindow;
use gtk4::{
    gio, glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate, MenuButton,
    PopoverMenu, Widget,
};
use rnote_engine::Camera;

mod imp {
    use super::*;
    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/canvasmenu.ui")]
    pub(crate) struct RnCanvasMenu {
        #[template_child]
        pub(crate) menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) popovermenu: TemplateChild<PopoverMenu>,
        #[template_child]
        pub(crate) menu_model: TemplateChild<gio::MenuModel>,
        #[template_child]
        pub(crate) zoom_in_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) zoom_out_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) zoom_reset_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) zoom_fit_width_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnCanvasMenu {
        const NAME: &'static str = "RnCanvasMenu";
        type Type = super::RnCanvasMenu;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnCanvasMenu {
        fn constructed(&self) {
            self.parent_constructed();

            self.menubutton
                .get()
                .set_popover(Some(&self.popovermenu.get()));
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnCanvasMenu {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.popovermenu.get().present();
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnCanvasMenu(ObjectSubclass<imp::RnCanvasMenu>)
    @extends Widget;
}

impl Default for RnCanvasMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl RnCanvasMenu {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn popovermenu(&self) -> PopoverMenu {
        self.imp().popovermenu.get()
    }

    pub(crate) fn zoom_reset_button(&self) -> Button {
        self.imp().zoom_reset_button.get()
    }

    pub(crate) fn init(&self, _appwindow: &RnAppWindow) {
        self.imp()
            .zoom_reset_button
            .set_label(format!("{:.0}%", (100.0 * Camera::ZOOM_DEFAULT).round()).as_str());
    }
}
