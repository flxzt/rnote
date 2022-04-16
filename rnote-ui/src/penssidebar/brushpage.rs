use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Image, ListBox,
    MenuButton, Popover, SpinButton,
};

use crate::{appwindow::RnoteAppWindow, ColorPicker};
use adw::prelude::*;
use rnote_compose::style::textured::{TexturedDotsDistribution, TexturedOptions};
use rnote_engine::pens::brush::BrushStyle;
use rnote_engine::utils::GdkRGBAHelpers;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/brushpage.ui")]
    pub struct BrushPage {
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
        pub brushstyle_marker_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub brushstyle_solid_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub brushstyle_textured_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub brushconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub brushconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub texturedstyle_density_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub texturedstyle_radius_x_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub texturedstyle_radius_y_spinbutton: TemplateChild<SpinButton>,
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

glib::wrapper! {
    pub struct BrushPage(ObjectSubclass<imp::BrushPage>)
        @extends gtk4::Widget;
}

impl Default for BrushPage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrushPage {
    /// The default width
    pub const WIDTH_DEFAULT: f64 = 3.6;
    /// The min width
    pub const WIDTH_MIN: f64 = 0.1;
    /// The max width
    pub const WIDTH_MAX: f64 = 1000.0;

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create BrushPage")
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        self.imp().width_spinbutton.get()
    }

    pub fn colorpicker(&self) -> ColorPicker {
        self.imp().colorpicker.get()
    }

    pub fn brushstyle_menubutton(&self) -> MenuButton {
        self.imp().brushstyle_menubutton.get()
    }

    pub fn brushstyle_image(&self) -> Image {
        self.imp().brushstyle_image.get()
    }

    pub fn brushstyle_listbox(&self) -> ListBox {
        self.imp().brushstyle_listbox.get()
    }

    pub fn brushstyle_marker_row(&self) -> adw::ActionRow {
        self.imp().brushstyle_marker_row.get()
    }

    pub fn brushstyle_solid_row(&self) -> adw::ActionRow {
        self.imp().brushstyle_solid_row.get()
    }

    pub fn brushstyle_textured_row(&self) -> adw::ActionRow {
        self.imp().brushstyle_textured_row.get()
    }

    pub fn brushconfig_menubutton(&self) -> MenuButton {
        self.imp().brushconfig_menubutton.get()
    }

    pub fn brushconfig_popover(&self) -> Popover {
        self.imp().brushconfig_popover.get()
    }

    pub fn texturedstyle_distribution_row(&self) -> adw::ComboRow {
        self.imp().texturedstyle_distribution_row.clone()
    }

    pub fn texturedstyle_density_spinbutton(&self) -> SpinButton {
        self.imp().texturedstyle_density_spinbutton.clone()
    }

    pub fn texturedstyle_radius_x_spinbutton(&self) -> SpinButton {
        self.imp().texturedstyle_radius_x_spinbutton.clone()
    }

    pub fn texturedstyle_radius_y_spinbutton(&self) -> SpinButton {
        self.imp().texturedstyle_radius_y_spinbutton.clone()
    }

    pub fn set_texturedstyle_distribution_variant(&self, distribution: TexturedDotsDistribution) {
        let position = match distribution {
            TexturedDotsDistribution::Uniform => 0,
            TexturedDotsDistribution::Normal => 1,
            TexturedDotsDistribution::Exponential => 2,
            TexturedDotsDistribution::ReverseExponential => 3,
        };

        self.imp()
            .texturedstyle_distribution_row
            .get()
            .set_selected(position);
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.width_spinbutton().set_increments(0.1, 2.0);
        self.width_spinbutton()
            .set_range(Self::WIDTH_MIN, Self::WIDTH_MAX);
        // Must be after set_range() !
        self.width_spinbutton().set_value(Self::WIDTH_DEFAULT);

        self.colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |colorpicker, _paramspec| {
                let color = colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();
                let brush_style = appwindow.canvas().engine().borrow_mut().penholder.brush.style;

                match brush_style {
                    BrushStyle::Marker => appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_color = Some(color),
                    BrushStyle::Solid => appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_color = Some(color),
                    BrushStyle::Textured => appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.stroke_color = Some(color),
                }
            }),
        );

        self.width_spinbutton().connect_value_changed(
            clone!(@weak appwindow => move |brush_widthscale_spinbutton| {
                let brush_style = appwindow.canvas().engine().borrow_mut().penholder.brush.style;

                match brush_style {
                    BrushStyle::Marker => appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_width = brush_widthscale_spinbutton.value(),
                    BrushStyle::Solid => appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_width = brush_widthscale_spinbutton.value(),
                    BrushStyle::Textured => appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.stroke_width = brush_widthscale_spinbutton.value(),
                }
            }),
        );

        self.brushstyle_listbox().connect_row_selected(
            clone!(@weak self as brushpage, @weak appwindow => move |_brushstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    match selected_row.index() {
                        // Solid
                        0 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"marker".to_variant()));
                        }
                        // Solid
                        1 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"solid".to_variant()));
                        }
                        // Textured
                        2 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"textured".to_variant()));
                        }
                        _ => {}
                    }
                }
            }),
        );

        // Textured style
        // Density
        self.imp()
            .texturedstyle_density_spinbutton
            .get()
            .set_increments(0.1, 2.0);
        self.imp()
            .texturedstyle_density_spinbutton
            .get()
            .set_range(0.0, f64::MAX);
        self.imp()
            .texturedstyle_density_spinbutton
            .get()
            .set_value(TexturedOptions::DENSITY_DEFAULT);

        self.imp().texturedstyle_density_spinbutton.get().connect_value_changed(
            clone!(@weak appwindow => move |texturedstyle_density_adj| {
                appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.density = texturedstyle_density_adj.value();
            }),
        );

        // Radius X
        self.imp()
            .texturedstyle_radius_x_spinbutton
            .get()
            .set_increments(0.1, 2.0);
        self.imp()
            .texturedstyle_radius_x_spinbutton
            .get()
            .set_range(0.0, f64::MAX);
        self.imp()
            .texturedstyle_radius_x_spinbutton
            .get()
            .set_value(TexturedOptions::RADII_DEFAULT[0]);

        self.imp()
            .texturedstyle_radius_x_spinbutton
            .get()
            .connect_value_changed(
                clone!(@weak appwindow => move |texturedstyle_radius_x_adj| {
                    let mut radii = appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.radii;
                    radii[0] = texturedstyle_radius_x_adj.value();
                    appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.radii = radii;
                }),
            );

        // Radius Y
        self.imp()
            .texturedstyle_radius_y_spinbutton
            .get()
            .set_increments(0.1, 2.0);
        self.imp()
            .texturedstyle_radius_y_spinbutton
            .get()
            .set_range(0.0, f64::MAX);
        self.imp()
            .texturedstyle_radius_y_spinbutton
            .get()
            .set_value(TexturedOptions::RADII_DEFAULT[1]);

        self.imp()
            .texturedstyle_radius_y_spinbutton
            .get()
            .connect_value_changed(
                clone!(@weak appwindow => move |texturedstyle_radius_y_adj| {
                    let mut radii = appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.radii;
                    radii[1] = texturedstyle_radius_y_adj.value();
                    appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.radii = radii;
                }),
            );

        // Distribution
        self.set_texturedstyle_distribution_variant(
            appwindow
                .canvas()
                .engine()
                .borrow()
                .penholder
                .brush
                .textured_options
                .distribution,
        );

        self.imp().texturedstyle_distribution_row.get().connect_selected_notify(clone!(@weak self as brushpage, @weak appwindow => move |texturedstyle_distribution_row| {
            match texturedstyle_distribution_row.selected() {
                    0 => {
                        appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.distribution = TexturedDotsDistribution::Uniform;
                    },
                    1 => {
                        appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.distribution = TexturedDotsDistribution::Normal;
                    },
                    2 => {
                        appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.distribution = TexturedDotsDistribution::Exponential;
                    },
                    3 => {
                        appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.distribution = TexturedDotsDistribution::ReverseExponential;
                    },
                    _ => {
                        log::error!(
                            "invalid position when selecting a distribution in texturedstyle_distribution_row"
                        );
                    }
            }
        }));
    }
}
