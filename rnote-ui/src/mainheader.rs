use crate::{appmenu::AppMenu, appwindow::RnoteAppWindow, canvasmenu::CanvasMenu};
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Label, ToggleButton,
    Widget,
};
use rnote_engine::pens::PenStyle;

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
        pub(crate) menus_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) pens_toggles_squeezer: TemplateChild<adw::Squeezer>,
        #[template_child]
        pub(crate) pens_toggles_clamp: TemplateChild<adw::Clamp>,
        #[template_child]
        pub(crate) pens_toggles_placeholderbox: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) brush_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) shaper_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) typewriter_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) eraser_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selector_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) tools_toggle: TemplateChild<ToggleButton>,
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

    pub(crate) fn headerbar(&self) -> adw::HeaderBar {
        self.imp().headerbar.get()
    }

    pub(crate) fn main_title(&self) -> adw::WindowTitle {
        self.imp().main_title.get()
    }

    pub(crate) fn main_title_unsaved_indicator(&self) -> Label {
        self.imp().main_title_unsaved_indicator.get()
    }

    pub(crate) fn menus_box(&self) -> gtk4::Box {
        self.imp().menus_box.get()
    }

    pub(crate) fn pens_toggles_squeezer(&self) -> adw::Squeezer {
        self.imp().pens_toggles_squeezer.get()
    }

    pub(crate) fn brush_toggle(&self) -> ToggleButton {
        self.imp().brush_toggle.get()
    }

    pub(crate) fn shaper_toggle(&self) -> ToggleButton {
        self.imp().shaper_toggle.get()
    }

    pub(crate) fn typewriter_toggle(&self) -> ToggleButton {
        self.imp().typewriter_toggle.get()
    }

    pub(crate) fn eraser_toggle(&self) -> ToggleButton {
        self.imp().eraser_toggle.get()
    }

    pub(crate) fn selector_toggle(&self) -> ToggleButton {
        self.imp().selector_toggle.get()
    }

    pub(crate) fn tools_toggle(&self) -> ToggleButton {
        self.imp().tools_toggle.get()
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
            .flags(
                glib::BindingFlags::SYNC_CREATE
                    | glib::BindingFlags::BIDIRECTIONAL
                    | glib::BindingFlags::INVERT_BOOLEAN,
            )
            .build();

        self.imp()
            .headerbar
            .get()
            .bind_property(
                "show-start-title-buttons",
                &appwindow.flap_header(),
                "show-start-title-buttons",
            )
            .flags(
                glib::BindingFlags::SYNC_CREATE
                    | glib::BindingFlags::BIDIRECTIONAL
                    | glib::BindingFlags::INVERT_BOOLEAN,
            )
            .build();

        self.pens_toggles_squeezer().connect_visible_child_notify(
            clone!(@weak self as mainheader, @weak appwindow => move |pens_toggles_squeezer| {
                if let Some(visible_child) = pens_toggles_squeezer.visible_child() {
                    if visible_child == mainheader.imp().pens_toggles_placeholderbox.get() {
                        appwindow.narrow_pens_toggles_revealer().set_reveal_child(true);
                    } else if visible_child == mainheader.imp().pens_toggles_clamp.get() {
                        appwindow.narrow_pens_toggles_revealer().set_reveal_child(false);
                    }
                }
            }),
        );

        self.imp().brush_toggle.get().connect_toggled(clone!(@weak appwindow => move |brush_toggle| {
            if brush_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Brush.nick().to_variant()));
            }
        }));

        self.imp().shaper_toggle.get().connect_toggled(clone!(@weak appwindow => move |shaper_toggle| {
            if shaper_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Shaper.nick().to_variant()));
            }
        }));

        self.imp().typewriter_toggle.get().connect_toggled(clone!(@weak appwindow => move |typewriter_toggle| {
            if typewriter_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Typewriter.nick().to_variant()));
            }
        }));

        self.imp().eraser_toggle.get().connect_toggled(clone!(@weak appwindow => move |eraser_toggle| {
            if eraser_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Eraser.nick().to_variant()));
            }
        }));

        self.imp().selector_toggle.get().connect_toggled(clone!(@weak appwindow => move |selector_toggle| {
            if selector_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Selector.nick().to_variant()));
            }
        }));

        self.imp().tools_toggle.get().connect_toggled(clone!(@weak appwindow => move |tools_toggle| {
            if tools_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Tools.nick().to_variant()));
            }
        }));
    }
}
