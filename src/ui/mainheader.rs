mod imp {
    use crate::ui::{appmenu::AppMenu, canvasmenu::CanvasMenu};
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Box, Button, CompositeTemplate, Image, Revealer,
        ToggleButton, Widget,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/felixzwettler/rnote/ui/mainheader.ui")]
    pub struct MainHeader {
        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub header_icon_image: TemplateChild<Image>,
        #[template_child]
        pub menus_box: TemplateChild<Box>,
        #[template_child]
        pub quickactions_box: TemplateChild<Box>,
        #[template_child]
        pub pageedit_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub add_page_button: TemplateChild<Button>,
        #[template_child]
        pub fit_to_format_button: TemplateChild<Button>,
        #[template_child]
        pub undo_button: TemplateChild<Button>,
        #[template_child]
        pub redo_button: TemplateChild<Button>,
        #[template_child]
        pub pens_togglebox: TemplateChild<Box>,
        #[template_child]
        pub marker_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub brush_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub eraser_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub selector_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub canvasmenu: TemplateChild<CanvasMenu>,
        #[template_child]
        pub appmenu: TemplateChild<AppMenu>,
    }

    impl Default for MainHeader {
        fn default() -> Self {
            Self {
                headerbar: TemplateChild::<adw::HeaderBar>::default(),
                header_icon_image: TemplateChild::<Image>::default(),
                menus_box: TemplateChild::<Box>::default(),
                quickactions_box: TemplateChild::<Box>::default(),
                pageedit_revealer: TemplateChild::<Revealer>::default(),
                add_page_button: TemplateChild::<Button>::default(),
                fit_to_format_button: TemplateChild::<Button>::default(),
                undo_button: TemplateChild::<Button>::default(),
                redo_button: TemplateChild::<Button>::default(),
                pens_togglebox: TemplateChild::<Box>::default(),
                marker_toggle: TemplateChild::<ToggleButton>::default(),
                brush_toggle: TemplateChild::<ToggleButton>::default(),
                eraser_toggle: TemplateChild::<ToggleButton>::default(),
                selector_toggle: TemplateChild::<ToggleButton>::default(),
                canvasmenu: TemplateChild::<CanvasMenu>::default(),
                appmenu: TemplateChild::<AppMenu>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainHeader {
        const NAME: &'static str = "MainHeader";
        type Type = super::MainHeader;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            CanvasMenu::static_type();
            AppMenu::static_type();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainHeader {
        fn constructed(&self, _obj: &Self::Type) {}

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for MainHeader {}
}

use crate::{
    config, ui::appmenu::AppMenu, ui::appwindow::RnoteAppWindow, ui::canvasmenu::CanvasMenu,
};

use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Box, Button, Image, Revealer,
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

    pub fn header_icon_image(&self) -> Image {
        imp::MainHeader::from_instance(self).header_icon_image.get()
    }

    pub fn menus_box(&self) -> Box {
        imp::MainHeader::from_instance(self).menus_box.get()
    }

    pub fn quickactions_box(&self) -> Box {
        imp::MainHeader::from_instance(self).quickactions_box.get()
    }

    pub fn pageedit_revealer(&self) -> Revealer {
        imp::MainHeader::from_instance(self).pageedit_revealer.get()
    }

    pub fn add_page_button(&self) -> Button {
        imp::MainHeader::from_instance(self).add_page_button.get()
    }

    pub fn fit_to_format_button(&self) -> Button {
        imp::MainHeader::from_instance(self)
            .fit_to_format_button
            .get()
    }

    pub fn undo_button(&self) -> Button {
        imp::MainHeader::from_instance(self).undo_button.get()
    }

    pub fn redo_button(&self) -> Button {
        imp::MainHeader::from_instance(self).redo_button.get()
    }

    pub fn pens_togglebox(&self) -> Box {
        imp::MainHeader::from_instance(self).pens_togglebox.get()
    }

    pub fn marker_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).marker_toggle.get()
    }

    pub fn brush_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).brush_toggle.get()
    }

    pub fn eraser_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).eraser_toggle.get()
    }

    pub fn selector_toggle(&self) -> ToggleButton {
        imp::MainHeader::from_instance(self).selector_toggle.get()
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
            .header_icon_image
            .get()
            .set_icon_name(Some(config::APP_ID));

        priv_
            .brush_toggle
            .get()
            .set_group(Some(&priv_.marker_toggle.get()));
        priv_
            .eraser_toggle
            .get()
            .set_group(Some(&priv_.marker_toggle.get()));
        priv_
            .selector_toggle
            .get()
            .set_group(Some(&priv_.marker_toggle.get()));

        priv_
            .headerbar
            .get()
            .bind_property(
                "show-end-title-buttons",
                &appwindow.workspace_headerbar(),
                "show-end-title-buttons",
            )
            .flags(
                glib::BindingFlags::SYNC_CREATE
                    | glib::BindingFlags::BIDIRECTIONAL
                    | glib::BindingFlags::INVERT_BOOLEAN,
            )
            .build();

        priv_.add_page_button.get().connect_clicked(
            clone!(@weak appwindow => move |_add_page_button| {
                let format_height = appwindow.canvas().sheet().format().borrow().height;
                appwindow.canvas().sheet().set_height(appwindow.canvas().sheet().height() + format_height);
                appwindow.canvas().queue_resize();
            }),
        );

        priv_.fit_to_format_button.get().connect_clicked(
            clone!(@weak appwindow => move |_fit_to_format_button| {
                appwindow.canvas().sheet().fit_to_format();
                appwindow.canvas().queue_resize();
            }),
        );

        priv_.marker_toggle.get().connect_active_notify(clone!(@weak appwindow => move |marker_toggle| {
            if marker_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-pen", Some(&"marker".to_variant()));
            }
        }));

        priv_.brush_toggle.get().connect_active_notify(clone!(@weak appwindow => move |brush_toggle| {
            if brush_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-pen", Some(&"brush".to_variant()));
            }
        }));

        priv_.eraser_toggle.get().connect_active_notify(clone!(@weak appwindow => move |eraser_toggle| {
            if eraser_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-pen", Some(&"eraser".to_variant()));
            }
        }));

        priv_.selector_toggle.get().connect_active_notify(clone!(@weak appwindow => move |selector_toggle| {
            if selector_toggle.is_active() {
                appwindow.application().unwrap().activate_action("current-pen", Some(&"selector".to_variant()));
            }
        }));

        priv_
            .undo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                if appwindow.canvas().sheet().undo_last_stroke() {
                    appwindow.canvas().queue_resize();
                }
                appwindow.canvas().queue_draw();
            }));

        priv_
            .redo_button
            .get()
            .connect_clicked(clone!(@weak appwindow => move |_| {
                if appwindow.canvas().sheet().redo_last_stroke() {
                    appwindow.canvas().queue_resize();
                }
                appwindow.canvas().queue_draw();
            }));
    }
}
