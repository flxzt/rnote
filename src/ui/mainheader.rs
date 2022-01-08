mod imp {
    use crate::config;
    use crate::ui::{appmenu::AppMenu, canvasmenu::CanvasMenu};
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Button, CompositeTemplate, Image, Label, Revealer,
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
        pub header_icon_image: TemplateChild<Image>,
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
        pub pens_togglebox: TemplateChild<gtk4::Box>,
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

            self.header_icon_image
                .get()
                .set_icon_name(Some(config::APP_ID));
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
    gio, glib, glib::clone, prelude::*, subclass::prelude::*, Button, Image, Label, Revealer,
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

    pub fn header_icon_image(&self) -> Image {
        imp::MainHeader::from_instance(self).header_icon_image.get()
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

    pub fn pens_togglebox(&self) -> gtk4::Box {
        imp::MainHeader::from_instance(self).pens_togglebox.get()
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
        let priv_ = imp::MainHeader::from_instance(self);

        priv_
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

        priv_
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

        priv_.add_page_button.get().connect_clicked(
            clone!(@weak appwindow => move |_add_page_button| {
                let format_height = appwindow.canvas().sheet().format().height();
                appwindow.canvas().sheet().set_height(appwindow.canvas().sheet().height() + format_height);
                appwindow.canvas().update_background_rendernode(true);
            }),
        );

        priv_.resize_to_format_button.get().connect_clicked(
            clone!(@weak appwindow => move |_resize_to_format_button| {
                appwindow.canvas().sheet().resize_to_format();
                appwindow.canvas().update_background_rendernode(true);
            }),
        );

        priv_.marker_toggle.get().connect_active_notify(clone!(@weak appwindow => move |marker_toggle| {
            if marker_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"marker".to_variant()));
            }
        }));

        priv_.brush_toggle.get().connect_active_notify(clone!(@weak appwindow => move |brush_toggle| {
            if brush_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"brush".to_variant()));
            }
        }));

        priv_.shaper_toggle.get().connect_active_notify(clone!(@weak appwindow => move |shaper_toggle| {
            if shaper_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"shaper".to_variant()));
            }
        }));

        priv_.eraser_toggle.get().connect_active_notify(clone!(@weak appwindow => move |eraser_toggle| {
            if eraser_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"eraser".to_variant()));
            }
        }));

        priv_.selector_toggle.get().connect_active_notify(clone!(@weak appwindow => move |selector_toggle| {
            if selector_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"selector".to_variant()));
            }
        }));

        priv_.tools_toggle.get().connect_active_notify(clone!(@weak appwindow => move |tools_toggle| {
            if tools_toggle.is_active() {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "current-pen", Some(&"tools".to_variant()));
            }
        }));

        priv_
            .undo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "undo-stroke", None);
            }));

        priv_
            .redo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "redo-stroke", None);
            }));
    }

    pub fn set_title_for_file(&self, file: Option<&gio::File>) {
        if let Some(file) = file {
            match file.query_info::<gio::Cancellable>(
                "standard::*",
                gio::FileQueryInfoFlags::NONE,
                None,
            ) {
                Ok(fileinfo) => {
                    self.main_title()
                        .set_title(fileinfo.display_name().as_str());
                    if let Some(path) = file.path() {
                        self.main_title().set_subtitle(&String::from(
                            glib::path_get_dirname(path).to_string_lossy() + "/",
                        ));
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
