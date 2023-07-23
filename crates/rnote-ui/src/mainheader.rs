// Imports
use crate::{appmenu::RnAppMenu, appwindow::RnAppWindow, canvasmenu::RnCanvasMenu};
use gtk4::{
    glib, prelude::*, subclass::prelude::*, CompositeTemplate, Label, ToggleButton, Widget,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/mainheader.ui")]
    pub(crate) struct RnMainHeader {
        #[template_child]
        pub(crate) headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(crate) main_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(crate) main_title_unsaved_indicator: TemplateChild<Label>,
        #[template_child]
        pub(crate) left_flapreveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) right_flapreveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) menus_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) canvasmenu: TemplateChild<RnCanvasMenu>,
        #[template_child]
        pub(crate) appmenu: TemplateChild<RnAppMenu>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnMainHeader {
        const NAME: &'static str = "RnMainHeader";
        type Type = super::RnMainHeader;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnMainHeader {
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
    impl WidgetImpl for RnMainHeader {}
}

glib::wrapper! {
    pub(crate) struct RnMainHeader(ObjectSubclass<imp::RnMainHeader>)
        @extends Widget;
}

impl Default for RnMainHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl RnMainHeader {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn main_title(&self) -> adw::WindowTitle {
        self.imp().main_title.get()
    }

    pub(crate) fn main_title_unsaved_indicator(&self) -> Label {
        self.imp().main_title_unsaved_indicator.get()
    }

    pub(crate) fn left_flapreveal_toggle(&self) -> ToggleButton {
        self.imp().left_flapreveal_toggle.get()
    }

    pub(crate) fn right_flapreveal_toggle(&self) -> ToggleButton {
        self.imp().right_flapreveal_toggle.get()
    }

    pub(crate) fn menus_box(&self) -> gtk4::Box {
        self.imp().menus_box.get()
    }

    pub(crate) fn canvasmenu(&self) -> RnCanvasMenu {
        self.imp().canvasmenu.get()
    }

    pub(crate) fn appmenu(&self) -> RnAppMenu {
        self.imp().appmenu.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.imp()
            .headerbar
            .get()
            .bind_property(
                "show-end-title-buttons",
                &appwindow.flap_header(),
                "show-end-title-buttons",
            )
            .sync_create()
            .bidirectional()
            .invert_boolean()
            .build();

        self.imp()
            .headerbar
            .get()
            .bind_property(
                "show-start-title-buttons",
                &appwindow.flap_header(),
                "show-start-title-buttons",
            )
            .sync_create()
            .bidirectional()
            .invert_boolean()
            .build();
    }
}
