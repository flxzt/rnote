use adw::prelude::*;
use gtk4::{
    gdk, glib, glib::clone, subclass::prelude::*, CompositeTemplate, Image, ListBox, MenuButton,
    Popover, SpinButton,
};
use num_traits::cast::ToPrimitive;

use rnote_compose::style::PressureCurve;
use rnote_engine::pens::Brush;

use crate::{appwindow::RnoteAppWindow, ColorPicker};
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
        pub solidstyle_pressure_curves_row: TemplateChild<adw::ComboRow>,
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

    pub fn solidstyle_pressure_curve(&self) -> PressureCurve {
        PressureCurve::try_from(self.imp().solidstyle_pressure_curves_row.get().selected()).unwrap()
    }

    pub fn set_solidstyle_pressure_curve(&self, pressure_curve: PressureCurve) {
        let position = pressure_curve.to_u32().unwrap();

        self.imp()
            .solidstyle_pressure_curves_row
            .get()
            .set_selected(position);
    }

    pub fn texturedstyle_dots_distribution(&self) -> TexturedDotsDistribution {
        TexturedDotsDistribution::try_from(
            self.imp().texturedstyle_distribution_row.get().selected(),
        )
        .unwrap()
    }

    pub fn set_texturedstyle_distribution_variant(&self, distribution: TexturedDotsDistribution) {
        let position = distribution.to_u32().unwrap();

        self.imp()
            .texturedstyle_distribution_row
            .get()
            .set_selected(position);
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.width_spinbutton().set_increments(0.1, 2.0);
        self.width_spinbutton()
            .set_range(Brush::STROKE_WIDTH_MIN, Brush::STROKE_WIDTH_MAX);
        // Must be after set_range() !
        self.width_spinbutton()
            .set_value(Brush::STROKE_WIDTH_DEFAULT);

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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after selecting brush color, Err `{}`", e);
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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing brush width, Err `{}`", e);
                }
            }),
        );

        self.brushstyle_listbox().connect_row_selected(
            clone!(@weak self as brushpage, @weak appwindow => move |_brushstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();

                        engine.penholder.brush.style = BrushStyle::try_from(selected_row.index() as u32).unwrap_or_default();
                        engine.penholder.brush.smooth_options.stroke_width = brushpage.width_spinbutton().value();
                        engine.penholder.brush.smooth_options.stroke_color = Some(brushpage.colorpicker().current_color());
                        engine.penholder.brush.textured_options.stroke_width = brushpage.width_spinbutton().value();
                        engine.penholder.brush.textured_options.stroke_color = Some(brushpage.colorpicker().current_color());
                    }

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing brush style, Err `{}`", e);
                    }
                    // Need to refresh the whole page, because changing the style affects multiple widgets
                    brushpage.refresh_ui(&appwindow);
                }
            }),
        );

        // Solid style
        // Pressure curve
        self.imp().solidstyle_pressure_curves_row.get().connect_selected_notify(clone!(@weak self as brushpage, @weak appwindow => move |_smoothstyle_pressure_curves_row| {
            appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.pressure_curve = brushpage.solidstyle_pressure_curve();

            if let Err(e) = appwindow.save_engine_config() {
                log::error!("saving engine config failed after changing brush pressure curve, Err `{}`", e);
            }
        }));

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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing brush textured density, Err `{}`", e);
                }
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

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing brush textured x radius, Err `{}`", e);
                    }
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

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing brush textured y radius, Err `{}`", e);
                    }
                }),
            );

        // dots distribution
        self.imp().texturedstyle_distribution_row.get().connect_selected_notify(clone!(@weak self as brushpage, @weak appwindow => move |_texturedstyle_distribution_row| {
            appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.distribution = brushpage.texturedstyle_dots_distribution();

            if let Err(e) = appwindow.save_engine_config() {
                log::error!("saving engine config failed after changing brush textured dots distribution, Err `{}`", e);
            }
        }));
    }

    pub fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let brush = appwindow.canvas().engine().borrow().penholder.brush.clone();

        self.set_solidstyle_pressure_curve(brush.smooth_options.pressure_curve);
        self.texturedstyle_density_spinbutton()
            .set_value(brush.textured_options.density);
        self.texturedstyle_radius_x_spinbutton()
            .set_value(brush.textured_options.radii[0]);
        self.texturedstyle_radius_y_spinbutton()
            .set_value(brush.textured_options.radii[1]);
        self.set_texturedstyle_distribution_variant(brush.textured_options.distribution);
        match brush.style {
            BrushStyle::Marker => {
                self.brushstyle_listbox()
                    .select_row(Some(&self.brushstyle_marker_row()));
                self.width_spinbutton()
                    .set_value(brush.smooth_options.stroke_width);
                self.colorpicker()
                    .set_current_color(brush.smooth_options.stroke_color);
                self.brushstyle_image()
                    .set_icon_name(Some("pen-brush-style-marker-symbolic"));
            }
            BrushStyle::Solid => {
                self.brushstyle_listbox()
                    .select_row(Some(&self.brushstyle_solid_row()));
                self.width_spinbutton()
                    .set_value(brush.smooth_options.stroke_width);
                self.colorpicker()
                    .set_current_color(brush.smooth_options.stroke_color);
                self.brushstyle_image()
                    .set_icon_name(Some("pen-brush-style-solid-symbolic"));
            }
            BrushStyle::Textured => {
                self.brushstyle_listbox()
                    .select_row(Some(&self.brushstyle_textured_row()));
                self.width_spinbutton()
                    .set_value(brush.textured_options.stroke_width);
                self.colorpicker()
                    .set_current_color(brush.textured_options.stroke_color);
                self.brushstyle_image()
                    .set_icon_name(Some("pen-brush-style-textured-symbolic"));
            }
        }
    }
}
