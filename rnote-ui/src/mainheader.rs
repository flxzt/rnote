use crate::{appmenu::AppMenu, appwindow::RnoteAppWindow, canvasmenu::CanvasMenu};
use gtk4::{glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate, Label, Widget};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/mainheader.ui")]
    pub(crate) struct MainHeader {
        #[template_child]
        pub(crate) headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(crate) main_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(crate) main_title_unsaved_indicator: TemplateChild<Label>,
        #[template_child]
        pub(crate) quickactions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) fixedsize_quickactions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) undo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) redo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) menus_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) canvasmenu: TemplateChild<CanvasMenu>,
        #[template_child]
        pub(crate) appmenu: TemplateChild<AppMenu>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainHeader {
        const NAME: &'static str = "MainHeader";
        type Type = super::MainHeader;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainHeader {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for MainHeader {}
}

glib::wrapper! {
    pub(crate) struct MainHeader(ObjectSubclass<imp::MainHeader>)
        @extends Widget;
}

impl Default for MainHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl MainHeader {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn main_title(&self) -> adw::WindowTitle {
        self.imp().main_title.get()
    }

    pub(crate) fn main_title_unsaved_indicator(&self) -> Label {
        self.imp().main_title_unsaved_indicator.get()
    }

    pub(crate) fn quickactions_box(&self) -> gtk4::Box {
        self.imp().quickactions_box.get()
    }

    pub(crate) fn fixedsize_quickactions_box(&self) -> gtk4::Box {
        self.imp().fixedsize_quickactions_box.get()
    }

    pub(crate) fn undo_button(&self) -> Button {
        self.imp().undo_button.get()
    }

    pub(crate) fn redo_button(&self) -> Button {
        self.imp().redo_button.get()
    }

    pub(crate) fn menus_box(&self) -> gtk4::Box {
        self.imp().menus_box.get()
    }

    pub(crate) fn canvasmenu(&self) -> CanvasMenu {
        self.imp().canvasmenu.get()
    }

    pub(crate) fn appmenu(&self) -> AppMenu {
        self.imp().appmenu.get()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
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
