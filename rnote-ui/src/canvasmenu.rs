use crate::appwindow::RnoteAppWindow;

use gtk4::{
    gio, glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate, MenuButton,
    PopoverMenu, Widget,
};
use rnote_engine::Camera;

mod imp {
    use super::*;
    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/canvasmenu.ui")]
    pub struct CanvasMenu {
        #[template_child]
        pub menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub popovermenu: TemplateChild<PopoverMenu>,
        #[template_child]
        pub menu_model: TemplateChild<gio::MenuModel>,
        #[template_child]
        pub zoom_in_button: TemplateChild<Button>,
        #[template_child]
        pub zoom_out_button: TemplateChild<Button>,
        #[template_child]
        pub zoom_reset_button: TemplateChild<Button>,
        #[template_child]
        pub zoom_fit_width_button: TemplateChild<Button>,
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

    impl WidgetImpl for CanvasMenu {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.popovermenu.get().present();
        }
    }
}

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
        glib::Object::new(&[])
    }

    pub fn menubutton(&self) -> MenuButton {
        self.imp().menubutton.get()
    }

    pub fn popovermenu(&self) -> PopoverMenu {
        self.imp().popovermenu.get()
    }

    pub fn menu_model(&self) -> gio::MenuModel {
        self.imp().menu_model.get()
    }

    pub fn zoomin_button(&self) -> Button {
        self.imp().zoom_in_button.get()
    }

    pub fn zoomout_button(&self) -> Button {
        self.imp().zoom_out_button.get()
    }

    pub fn zoomreset_button(&self) -> Button {
        self.imp().zoom_reset_button.get()
    }

    pub fn zoom_fit_width_button(&self) -> Button {
        self.imp().zoom_fit_width_button.get()
    }

    pub fn init(&self, _appwindow: &RnoteAppWindow) {
        self.zoomreset_button()
            .set_label(format!("{:.0}%", (100.0 * Camera::ZOOM_DEFAULT).round()).as_str());
    }
}
