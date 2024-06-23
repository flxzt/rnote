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
        pub(crate) left_sidebar_reveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) right_sidebar_reveal_toggle: TemplateChild<ToggleButton>,
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
            klass.bind_template();
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

    pub(crate) fn headerbar(&self) -> adw::HeaderBar {
        self.imp().headerbar.get()
    }

    pub(crate) fn main_title(&self) -> adw::WindowTitle {
        self.imp().main_title.get()
    }

    pub(crate) fn main_title_unsaved_indicator(&self) -> Label {
        self.imp().main_title_unsaved_indicator.get()
    }

    pub(crate) fn left_sidebar_reveal_toggle(&self) -> ToggleButton {
        self.imp().left_sidebar_reveal_toggle.get()
    }

    pub(crate) fn right_sidebar_reveal_toggle(&self) -> ToggleButton {
        self.imp().right_sidebar_reveal_toggle.get()
    }

    pub(crate) fn canvasmenu(&self) -> RnCanvasMenu {
        self.imp().canvasmenu.get()
    }

    pub(crate) fn appmenu(&self) -> RnAppMenu {
        self.imp().appmenu.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.canvasmenu.get().init(appwindow);
        imp.appmenu.get().init(appwindow);
    }
}
