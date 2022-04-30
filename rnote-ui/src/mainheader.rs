use crate::{appmenu::AppMenu, appwindow::RnoteAppWindow, canvasmenu::CanvasMenu};
use gtk4::{
    gio, glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate, Label,
    Revealer, ToggleButton, Widget,
};
use rnote_engine::pens::penholder::PenStyle;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/mainheader.ui")]
    pub struct MainHeader {
        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub main_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub main_title_unsaved_indicator: TemplateChild<Label>,
        #[template_child]
        pub menus_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub quickactions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub pageedit_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub add_page_button: TemplateChild<Button>,
        #[template_child]
        pub resize_to_fit_strokes_button: TemplateChild<Button>,
        #[template_child]
        pub undo_button: TemplateChild<Button>,
        #[template_child]
        pub redo_button: TemplateChild<Button>,
        #[template_child]
        pub pens_toggles_squeezer: TemplateChild<adw::Squeezer>,
        #[template_child]
        pub pens_toggles_clamp: TemplateChild<adw::Clamp>,
        #[template_child]
        pub pens_toggles_placeholderbox: TemplateChild<gtk4::Box>,
        #[template_child]
        pub brush_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub shaper_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub eraser_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub selector_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub tools_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub canvasmenu: TemplateChild<CanvasMenu>,
        #[template_child]
        pub appmenu: TemplateChild<AppMenu>,
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for MainHeader {}
}

glib::wrapper! {
    pub struct MainHeader(ObjectSubclass<imp::MainHeader>)
        @extends Widget;
}

impl Default for MainHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl MainHeader {
    pub fn new() -> Self {
        let mainheader: MainHeader = glib::Object::new(&[]).expect("Failed to create MainHeader");
        mainheader
    }

    pub fn headerbar(&self) -> adw::HeaderBar {
        self.imp().headerbar.get()
    }

    pub fn main_title(&self) -> adw::WindowTitle {
        self.imp().main_title.get()
    }

    pub fn main_title_unsaved_indicator(&self) -> Label {
        self.imp().main_title_unsaved_indicator.get()
    }

    pub fn menus_box(&self) -> gtk4::Box {
        self.imp().menus_box.get()
    }

    pub fn quickactions_box(&self) -> gtk4::Box {
        self.imp().quickactions_box.get()
    }

    pub fn pageedit_revealer(&self) -> Revealer {
        self.imp().pageedit_revealer.get()
    }

    pub fn add_page_button(&self) -> Button {
        self.imp().add_page_button.get()
    }

    pub fn resize_to_fit_strokes_button(&self) -> Button {
        self.imp().resize_to_fit_strokes_button.get()
    }

    pub fn undo_button(&self) -> Button {
        self.imp().undo_button.get()
    }

    pub fn redo_button(&self) -> Button {
        self.imp().redo_button.get()
    }

    pub fn pens_toggles_placeholderbox(&self) -> gtk4::Box {
        self.imp().pens_toggles_placeholderbox.get()
    }

    pub fn pens_toggles_squeezer(&self) -> adw::Squeezer {
        self.imp().pens_toggles_squeezer.get()
    }

    pub fn brush_toggle(&self) -> ToggleButton {
        self.imp().brush_toggle.get()
    }

    pub fn shaper_toggle(&self) -> ToggleButton {
        self.imp().shaper_toggle.get()
    }

    pub fn eraser_toggle(&self) -> ToggleButton {
        self.imp().eraser_toggle.get()
    }

    pub fn selector_toggle(&self) -> ToggleButton {
        self.imp().selector_toggle.get()
    }

    pub fn tools_toggle(&self) -> ToggleButton {
        self.imp().tools_toggle.get()
    }

    pub fn canvasmenu(&self) -> CanvasMenu {
        self.imp().canvasmenu.get()
    }

    pub fn appmenu(&self) -> AppMenu {
        self.imp().appmenu.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
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

        self.imp().add_page_button.get().connect_clicked(
            clone!(@weak appwindow => move |_add_page_button| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "add-page-to-sheet", None);
            }),
        );

        self.imp().resize_to_fit_strokes_button.get().connect_clicked(
            clone!(@weak appwindow => move |_resize_to_format_button| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "resize-to-fit-strokes", None);
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

        self.imp()
            .undo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "undo", None);
            }));

        self.imp()
            .redo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "redo", None);
            }));
    }

    pub fn set_title_for_file(&self, file: Option<&gio::File>) {
        if let Some(file) = file {
            match file.query_info(
                "standard::*",
                gio::FileQueryInfoFlags::NONE,
                None::<&gio::Cancellable>,
            ) {
                Ok(fileinfo) => {
                    let title = fileinfo.name().with_extension("");

                    self.main_title().set_title(&title.to_string_lossy());

                    if let Some(mut path) = file.path() {
                        if path.pop() {
                            self.main_title()
                                .set_subtitle(&String::from(path.to_string_lossy() + "/"));
                        }
                    }
                }
                Err(e) => {
                    log::warn!("failed to query fileinfo for file {:?}, {}", file, e);
                }
            }
        } else {
            self.main_title().set_title("New Document");
            self.main_title().set_subtitle("Draft");
        }
    }
}
