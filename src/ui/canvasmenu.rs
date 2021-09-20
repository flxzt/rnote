mod imp {
    use gtk4::{
        gio::MenuModel, glib, glib::clone, prelude::*, subclass::prelude::*, Button,
        CompositeTemplate, Entry, MenuButton, PopoverMenu, ToggleButton,
    };

    use crate::sheet::format::Format;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/felixzwettler/rnote/ui/canvasmenu.ui")]
    pub struct CanvasMenu {
        #[template_child]
        pub menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub popovermenu: TemplateChild<PopoverMenu>,
        #[template_child]
        pub menu_model: TemplateChild<MenuModel>,
        #[template_child]
        pub zoomin_button: TemplateChild<Button>,
        #[template_child]
        pub zoomout_button: TemplateChild<Button>,
        #[template_child]
        pub zoomreset_button: TemplateChild<Button>,
        #[template_child]
        pub zoom_fit_width_button: TemplateChild<Button>,
        #[template_child]
        pub lefthanded_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub righthanded_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub custom_format_width_entry: TemplateChild<Entry>,
        #[template_child]
        pub custom_format_height_entry: TemplateChild<Entry>,
        #[template_child]
        pub custom_format_dpi_entry: TemplateChild<Entry>,
        #[template_child]
        pub custom_format_apply: TemplateChild<Button>,
    }

    impl Default for CanvasMenu {
        fn default() -> Self {
            Self {
                menubutton: TemplateChild::<MenuButton>::default(),
                popovermenu: TemplateChild::<PopoverMenu>::default(),
                menu_model: TemplateChild::<MenuModel>::default(),
                zoomin_button: TemplateChild::<Button>::default(),
                zoomout_button: TemplateChild::<Button>::default(),
                zoomreset_button: TemplateChild::<Button>::default(),
                zoom_fit_width_button: TemplateChild::<Button>::default(),
                lefthanded_toggle: TemplateChild::<ToggleButton>::default(),
                righthanded_toggle: TemplateChild::<ToggleButton>::default(),
                custom_format_width_entry: TemplateChild::<Entry>::default(),
                custom_format_height_entry: TemplateChild::<Entry>::default(),
                custom_format_dpi_entry: TemplateChild::<Entry>::default(),
                custom_format_apply: TemplateChild::<Button>::default(),
            }
        }
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            let custom_format_width_entry = self.custom_format_width_entry.get();
            let custom_format_height_entry = self.custom_format_height_entry.get();
            let custom_format_dpi_entry = self.custom_format_dpi_entry.get();

            self.menubutton
                .get()
                .set_popover(Some(&self.popovermenu.get()));

            self.custom_format_width_entry.get().set_tooltip_text(Some(
                format!(
                    "Width must be between `{}` and `{}`",
                    Format::WIDTH_MIN,
                    Format::WIDTH_MAX
                )
                .as_str(),
            ));
            self.custom_format_width_entry
                .get()
                .buffer()
                .connect_text_notify(clone!(@weak custom_format_width_entry => move |buffer| {
                    if Format::try_parse_width(buffer.text().as_str()).is_some() {
                        custom_format_width_entry.style_context().remove_class("error");
                        custom_format_width_entry.style_context().add_class("plain");
                    } else {
                        custom_format_width_entry.style_context().remove_class("plain");
                        custom_format_width_entry.style_context().add_class("error");
                    }
                }));

            self.custom_format_height_entry.get().set_tooltip_text(Some(
                format!(
                    "Height must be between `{}` and `{}`",
                    Format::HEIGHT_MIN,
                    Format::HEIGHT_MAX
                )
                .as_str(),
            ));
            self.custom_format_height_entry
                .get()
                .buffer()
                .connect_text_notify(clone!(@weak custom_format_height_entry => move |buffer| {
                    if Format::try_parse_height(buffer.text().as_str()).is_some() {
                        custom_format_height_entry.style_context().remove_class("error");
                        custom_format_height_entry.style_context().add_class("plain");
                    } else {
                        custom_format_height_entry.style_context().remove_class("plain");
                        custom_format_height_entry.style_context().add_class("error");
                    }
                }));

            self.custom_format_dpi_entry.get().set_tooltip_text(Some(
                format!(
                    "DPI must be between `{}` and `{}`",
                    Format::DPI_MIN,
                    Format::DPI_MAX
                )
                .as_str(),
            ));
            self.custom_format_dpi_entry
                .get()
                .buffer()
                .connect_text_notify(clone!(@weak custom_format_dpi_entry => move |buffer| {
                    if Format::try_parse_height(buffer.text().as_str()).is_some() {
                        custom_format_dpi_entry.style_context().remove_class("error");
                        custom_format_dpi_entry.style_context().add_class("plain");
                    } else {
                        custom_format_dpi_entry.style_context().remove_class("plain");
                        custom_format_dpi_entry.style_context().add_class("error");
                    }
                }));
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for CanvasMenu {
        fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(widget, width, height, baseline);
            self.popovermenu.get().present();
        }
    }
}

use crate::sheet::format::Format;
use crate::ui::{appwindow::RnoteAppWindow, canvas};

use gtk4::{gio, Entry, MenuButton, PopoverMenu, Widget};
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, Button, ToggleButton};

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
        let canvasmenu: CanvasMenu = glib::Object::new(&[]).expect("Failed to create CanvasMenu");
        canvasmenu
    }

    pub fn menubutton(&self) -> MenuButton {
        imp::CanvasMenu::from_instance(self)
            .menubutton
            .get()
            .clone()
    }

    pub fn popovermenu(&self) -> PopoverMenu {
        imp::CanvasMenu::from_instance(self)
            .popovermenu
            .get()
            .clone()
    }

    pub fn menu_model(&self) -> gio::MenuModel {
        imp::CanvasMenu::from_instance(self)
            .menu_model
            .get()
            .clone()
    }

    pub fn zoomin_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self)
            .zoomin_button
            .get()
            .clone()
    }

    pub fn zoomout_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self)
            .zoomout_button
            .get()
            .clone()
    }

    pub fn zoomreset_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self)
            .zoomreset_button
            .get()
            .clone()
    }

    pub fn zoom_fit_width_button(&self) -> Button {
        imp::CanvasMenu::from_instance(self)
            .zoom_fit_width_button
            .get()
            .clone()
    }

    pub fn lefthanded_toggle(&self) -> ToggleButton {
        imp::CanvasMenu::from_instance(self)
            .lefthanded_toggle
            .get()
            .clone()
    }

    pub fn righthanded_toggle(&self) -> ToggleButton {
        imp::CanvasMenu::from_instance(self)
            .righthanded_toggle
            .get()
            .clone()
    }

    pub fn custom_format_width_entry(&self) -> Entry {
        imp::CanvasMenu::from_instance(self)
            .custom_format_width_entry
            .get()
            .clone()
    }

    pub fn custom_format_height_entry(&self) -> Entry {
        imp::CanvasMenu::from_instance(self)
            .custom_format_height_entry
            .get()
            .clone()
    }

    pub fn custom_format_dpi_entry(&self) -> Entry {
        imp::CanvasMenu::from_instance(self)
            .custom_format_dpi_entry
            .get()
            .clone()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::CanvasMenu::from_instance(self);
        let zoomreset_button = priv_.zoomreset_button.get();
        let custom_format_width_entry = priv_.custom_format_width_entry.get();
        let custom_format_height_entry = priv_.custom_format_height_entry.get();
        let custom_format_dpi_entry = priv_.custom_format_dpi_entry.get();

        priv_.custom_format_apply.get().connect_clicked(clone!(@weak appwindow, @weak custom_format_width_entry, @weak custom_format_height_entry, @weak custom_format_dpi_entry => move |_custom_format_apply| {
                if let (Some(width), Some(height), Some(dpi)) = (Format::try_parse_width(custom_format_width_entry.buffer().text().as_str()),
                    Format::try_parse_height(custom_format_height_entry.buffer().text().as_str()), Format::try_parse_dpi(custom_format_dpi_entry.buffer().text().as_str())) {
                        appwindow.application().unwrap().activate_action("sheet-format", Some(&(width, height, dpi).to_variant()));
                        appwindow.application().unwrap().activate_action("predefined-format", Some(&"custom".to_variant()));
                    };
            }));
        priv_
            .righthanded_toggle
            .set_group(Some(&priv_.lefthanded_toggle.get()));

        priv_.righthanded_toggle.connect_active_notify(clone!(@weak appwindow => move |_righthanded_toggle| {
            appwindow.application().unwrap().change_action_state("righthanded", &true.to_variant());
        }));

        priv_.lefthanded_toggle.connect_active_notify(clone!(@weak appwindow => move |_lefthanded_toggle| {
            appwindow.application().unwrap().change_action_state("righthanded", &false.to_variant());
        }));

        priv_.zoom_fit_width_button.connect_clicked(
            clone!(@weak appwindow => move |_zoom_fit_width_button| {
                appwindow.application().unwrap().activate_action("zoom-fit-width", None);
            }),
        );

        priv_.zoomreset_button.connect_clicked(clone!(@weak appwindow => move |_zoomreset_button| {
            appwindow.canvas().set_property("scalefactor", &canvas::Canvas::SCALE_DEFAULT).unwrap();
        }));

        priv_.zoomin_button.connect_clicked(
            clone!(@weak appwindow, @weak zoomreset_button => move |_| {
                let scalefactor = ((appwindow.canvas().property("scalefactor").unwrap().get::<f64>().unwrap() * 10. ).floor() + 1. ) / 10.;
                appwindow.canvas().set_property("scalefactor", &scalefactor).unwrap();
            }),
        );

        priv_.zoomout_button.connect_clicked(
            clone!(@weak appwindow, @weak zoomreset_button => move |_| {
                let scalefactor = ((appwindow.canvas().property("scalefactor").unwrap().get::<f64>().unwrap() * 10.).ceil() - 1.) / 10.;
                appwindow.canvas().set_property("scalefactor", &scalefactor).unwrap();
            }),
        );
    }
}
