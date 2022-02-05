mod imp {
    use crate::ui::{appmenu::AppMenu, canvasmenu::CanvasMenu};
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate, Label, Revealer,
        ToggleButton, Widget,
    };

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
        pub resize_to_format_button: TemplateChild<Button>,
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
        pub marker_toggle: TemplateChild<ToggleButton>,
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

use crate::{ui::appmenu::AppMenu, ui::appwindow::RnoteAppWindow, ui::canvasmenu::CanvasMenu};

use gtk4::{
    gio, glib, glib::clone, prelude::*, subclass::prelude::*, Button, Label, Revealer,
    ToggleButton, Widget,
};

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
        imp::MainHeader::from_instance(self).headerbar.get()
    }

    pub fn main_title(&self) -> adw::WindowTitle {
        imp::MainHeader::from_instance(self).main_title.get()
    }

    pub fn main_title_unsaved_indicator(&self) -> Label {
        imp::MainHeader::from_instance(self)
            .main_title_unsaved_indicator
            .get()
    }

    pub fn menus_box(&self) -> gtk4::Box {
        imp::MainHeader::from_instance(self).menus_box.get()
    }

    pub fn quickactions_box(&self) -> gtk4::Box {
        imp::MainHeader::from_instance(self).quickactions_box.get()
    }

    pub fn pageedit_revealer(&self) -> Revealer {
        imp::MainHeader::from_instance(self).pageedit_revealer.get()
    }

    pub fn add_page_button(&self) -> Button {
        imp::MainHeader::from_instance(self).add_page_button.get()
    }

    pub fn resize_to_format_button(&self) -> Button {
        imp::MainHeader::from_instance(self)
            .resize_to_format_button
            .get()
    }

    pub fn undo_button(&self) -> Button {
        imp::MainHeader::from_instance(self).undo_button.get()
    }

    pub fn redo_button(&self) -> Button {
        imp::MainHeader::from_instance(self).redo_button.get()
    }

    pub fn pens_toggles_placeholderbox(&self) -> gtk4::Box {
        imp::MainHeader::from_instance(self)
            .pens_toggles_placeholderbox
            .get()
    }

    pub fn pens_toggles_squeezer(&self) -> adw::Squeezer {
        imp::MainHeader::from_instance(self)
            .pens_toggles_squeezer
            .get()
    }

    pub fn marker_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).marker_toggle.get()
    }

    pub fn brush_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).brush_toggle.get()
    }

    pub fn shaper_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).shaper_toggle.get()
    }

    pub fn eraser_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).eraser_toggle.get()
    }

    pub fn selector_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).selector_toggle.get()
    }

    pub fn tools_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).tools_toggle.get()
    }

    pub fn canvasmenu(&self) -> CanvasMenu {
        imp::MainHeader::from_instance(self).canvasmenu.get()
    }

    pub fn appmenu(&self) -> AppMenu {
        imp::MainHeader::from_instance(self).appmenu.get()
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
                let format_height = appwindow.canvas().sheet().borrow().format.height;
                let new_sheet_height = appwindow.canvas().sheet().borrow().height + format_height;
                appwindow.canvas().sheet().borrow_mut().height = new_sheet_height;

                appwindow.canvas().update_background_rendernode(true);
            }),
        );

        self.imp().resize_to_format_button.get().connect_clicked(
            clone!(@weak appwindow => move |_resize_to_format_button| {
                appwindow.canvas().resize_to_format();
                appwindow.canvas().update_background_rendernode(true);
            }),
        );

        self.imp().marker_toggle.get().connect_toggled(clone!(@weak appwindow => move |marker_toggle| {
            if marker_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"marker_style".to_variant()));
            }
        }));

        self.imp().brush_toggle.get().connect_toggled(clone!(@weak appwindow => move |brush_toggle| {
            if brush_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"brush_style".to_variant()));
            }
        }));

        self.imp().shaper_toggle.get().connect_toggled(clone!(@weak appwindow => move |shaper_toggle| {
            if shaper_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"shaper_style".to_variant()));
            }
        }));

        self.imp().eraser_toggle.get().connect_toggled(clone!(@weak appwindow => move |eraser_toggle| {
            if eraser_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"eraser_style".to_variant()));
            }
        }));

        self.imp().selector_toggle.get().connect_toggled(clone!(@weak appwindow => move |selector_toggle| {
            if selector_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"selector_style".to_variant()));
            }
        }));

        self.imp().tools_toggle.get().connect_toggled(clone!(@weak appwindow => move |tools_toggle| {
            if tools_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"tools_style".to_variant()));
            }
        }));

        self.imp()
            .undo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "undo-stroke", None);
            }));

        self.imp()
            .redo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "redo-stroke", None);
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
                    self.main_title()
                        .set_title(fileinfo.display_name().as_str());
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
