mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };
    use gtk4::{Image, ListBox, MenuButton, Popover};

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/brushpage.ui")]
    pub struct BrushPage {
        #[template_child]
        pub width_resetbutton: TemplateChild<Button>,
        #[template_child]
        pub width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub brushstyle_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub brushstyle_image: TemplateChild<Image>,
        #[template_child]
        pub brushstyle_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub brushstyle_solid_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub brushstyle_textured_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub brushstyle_experimental_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub styleconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub styleconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub texturedstyle_density_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub texturedstyle_radius_x_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub texturedstyle_radius_y_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub texturedstyle_distribution_row: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BrushPage {
        const NAME: &'static str = "BrushPage";
        type Type = super::BrushPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BrushPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for BrushPage {}
}

use crate::compose::color::Color;
use crate::compose::textured::{TexturedDotsDistribution, TexturedOptions};
use crate::pens::brush::Brush;
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use adw::prelude::*;
use gtk4::{
    gdk, Accessible, Actionable, Buildable, ConstraintTarget, Image, ListBox, MenuButton, Popover,
};
use gtk4::{
    glib, glib::clone, subclass::prelude::*, Adjustment, Button, Orientable, SpinButton, Widget,
};

glib::wrapper! {
    pub struct BrushPage(ObjectSubclass<imp::BrushPage>)
        @extends Widget,
        @implements Orientable, Accessible, Actionable, Buildable, ConstraintTarget;
}

impl Default for BrushPage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrushPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create BrushPage")
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::BrushPage::from_instance(self).width_resetbutton.get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::BrushPage::from_instance(self).width_adj.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::BrushPage::from_instance(self).width_spinbutton.get()
    }

    pub fn colorpicker(&self) -> ColorPicker {
        imp::BrushPage::from_instance(self).colorpicker.get()
    }

    pub fn brushstyle_menubutton(&self) -> MenuButton {
        imp::BrushPage::from_instance(self)
            .brushstyle_menubutton
            .get()
    }

    pub fn brushstyle_image(&self) -> Image {
        imp::BrushPage::from_instance(self).brushstyle_image.get()
    }

    pub fn brushstyle_listbox(&self) -> ListBox {
        imp::BrushPage::from_instance(self).brushstyle_listbox.get()
    }

    pub fn brushstyle_solid_row(&self) -> adw::ActionRow {
        imp::BrushPage::from_instance(self)
            .brushstyle_solid_row
            .get()
    }

    pub fn brushstyle_textured_row(&self) -> adw::ActionRow {
        imp::BrushPage::from_instance(self)
            .brushstyle_textured_row
            .get()
    }

    pub fn brushstyle_experimental_row(&self) -> adw::ActionRow {
        imp::BrushPage::from_instance(self)
            .brushstyle_experimental_row
            .get()
    }

    pub fn styleconfig_menubutton(&self) -> MenuButton {
        imp::BrushPage::from_instance(self)
            .styleconfig_menubutton
            .get()
    }

    pub fn styleconfigonfig_popover(&self) -> Popover {
        imp::BrushPage::from_instance(self)
            .styleconfig_popover
            .get()
    }

    pub fn texturedstyle_distribution_row(&self) -> adw::ComboRow {
        imp::BrushPage::from_instance(self)
            .texturedstyle_distribution_row
            .clone()
    }

    pub fn set_texturedstyle_distribution_variant(&self, distribution: TexturedDotsDistribution) {
        let texturedstyle_distribution_listmodel = self
            .imp()
            .texturedstyle_distribution_row
            .get()
            .model()
            .unwrap()
            .downcast::<adw::EnumListModel>()
            .unwrap();
        self.imp()
            .texturedstyle_distribution_row
            .get()
            .set_selected(texturedstyle_distribution_listmodel.find_position(distribution as i32));
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let width_adj = self.width_adj();

        self.width_adj().set_lower(Brush::WIDTH_MIN);
        self.width_adj().set_upper(Brush::WIDTH_MAX);
        self.width_adj().set_value(Brush::WIDTH_DEFAULT);

        self.colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |colorpicker, _paramspec| {
                let color = colorpicker.property::<gdk::RGBA>("current-color");
                appwindow.canvas().pens().borrow_mut().brush.set_color(Color::from(color));
            }),
        );

        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().brush.set_width(Brush::WIDTH_DEFAULT);
                width_adj.set_value(Brush::WIDTH_DEFAULT);
            }),
        );

        self.width_adj().connect_value_changed(
            clone!(@weak appwindow => move |brush_widthscale_adj| {
                appwindow.canvas().pens().borrow_mut().brush.set_width(brush_widthscale_adj.value());
            }),
        );

        self.brushstyle_listbox().connect_row_selected(
            clone!(@weak self as brushpage, @weak appwindow => move |_brushstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    match selected_row.index() {
                        // Solid
                        0 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"solid".to_variant()));
                            brushpage.brushstyle_image().set_icon_name(Some("pen-brush-style-solid-symbolic"));
                            brushpage.styleconfig_menubutton().set_sensitive(false);
                        }
                        // Textured
                        1 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"textured".to_variant()));
                            brushpage.brushstyle_image().set_icon_name(Some("pen-brush-style-textured-symbolic"));
                            brushpage.styleconfig_menubutton().set_sensitive(true);
                        }
                        // Experimental
                        2 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"experimental".to_variant()));
                            brushpage.brushstyle_image().set_icon_name(Some("pen-brush-style-experimental-symbolic"));
                            brushpage.styleconfig_menubutton().set_sensitive(false);
                        }
                        _ => {}
                    }
                }
            }),
        );

        // Textured style
        // Density
        self.imp()
            .texturedstyle_density_adj
            .get()
            .set_lower(TexturedOptions::DENSITY_MIN);
        self.imp()
            .texturedstyle_density_adj
            .get()
            .set_upper(TexturedOptions::DENSITY_MAX);
        self.imp()
            .texturedstyle_density_adj
            .get()
            .set_value(TexturedOptions::DENSITY_DEFAULT);

        self.imp().texturedstyle_density_adj.get().connect_value_changed(
            clone!(@weak appwindow => move |texturedstyle_density_adj| {
                appwindow.canvas().pens().borrow_mut().brush.textured_config.set_density(texturedstyle_density_adj.value());
            }),
        );

        // Radius X
        self.imp()
            .texturedstyle_radius_x_adj
            .get()
            .set_lower(TexturedOptions::RADII_MIN[0]);
        self.imp()
            .texturedstyle_radius_x_adj
            .get()
            .set_upper(TexturedOptions::RADII_MAX[0]);
        self.imp()
            .texturedstyle_radius_x_adj
            .get()
            .set_value(TexturedOptions::RADII_DEFAULT[0]);

        self.imp()
            .texturedstyle_radius_x_adj
            .get()
            .connect_value_changed(
                clone!(@weak appwindow => move |texturedstyle_radius_x_adj| {
                    let mut radii = appwindow.canvas().pens().borrow().brush.textured_config.radii();
                    radii[0] = texturedstyle_radius_x_adj.value();
                    appwindow.canvas().pens().borrow_mut().brush.textured_config.set_radii(radii);
                }),
            );

        // Radius Y
        self.imp()
            .texturedstyle_radius_y_adj
            .get()
            .set_lower(TexturedOptions::RADII_MIN[1]);
        self.imp()
            .texturedstyle_radius_y_adj
            .get()
            .set_upper(TexturedOptions::RADII_MAX[1]);
        self.imp()
            .texturedstyle_radius_y_adj
            .get()
            .set_value(TexturedOptions::RADII_DEFAULT[1]);

        self.imp()
            .texturedstyle_radius_y_adj
            .get()
            .connect_value_changed(
                clone!(@weak appwindow => move |texturedstyle_radius_y_adj| {
                    let mut radii = appwindow.canvas().pens().borrow().brush.textured_config.radii();
                    radii[1] = texturedstyle_radius_y_adj.value();
                    appwindow.canvas().pens().borrow_mut().brush.textured_config.set_radii(radii);
                }),
            );

        // Distribution
        self.set_texturedstyle_distribution_variant(
            appwindow
                .canvas()
                .pens()
                .borrow()
                .brush
                .textured_config
                .distribution(),
        );

        self.imp().texturedstyle_distribution_row.get().connect_selected_item_notify(clone!(@weak self as brushpage, @weak appwindow => move |texturedstyle_distribution_row| {
            if let Some(selected_item) = texturedstyle_distribution_row.selected_item() {
                match selected_item
                    .downcast::<adw::EnumListItem>()
                    .unwrap()
                    .nick()
                    .as_str()
                {
                    "uniform" => {
                        appwindow.canvas().pens().borrow_mut().brush.textured_config.set_distribution(TexturedDotsDistribution::Uniform);
                    },
                    "normal" => {
                        appwindow.canvas().pens().borrow_mut().brush.textured_config.set_distribution(TexturedDotsDistribution::Normal);
                    },
                    "exponential" => {
                        appwindow.canvas().pens().borrow_mut().brush.textured_config.set_distribution(TexturedDotsDistribution::Exponential);
                    },
                    "reverse-exponential" => {
                        appwindow.canvas().pens().borrow_mut().brush.textured_config.set_distribution(TexturedDotsDistribution::ReverseExponential);
                    },
                    _ => {
                        log::error!(
                            "invalid nick string when selecting a distribution in texturedstyle_distribution_row"
                        );
                    }
                };

                appwindow.canvas().regenerate_background(true);
            }
        }));
    }
}
