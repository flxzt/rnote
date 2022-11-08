mod border_width_widget;
mod constraints_widget;
mod fill_colorpicker_widget;
mod shapeconfiguration_widget;
mod shaperbuilder_widget;
mod shaperstyle_widget;
mod stroke_colorpicker_widget;

use crate::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use gtk4::{
    glib, prelude::*, subclass::prelude::*, CompositeTemplate, Image, ListBox, MenuButton, Popover,
    SpinButton, Switch,
};
use rnote_engine::pens::shaper::ShaperStyle;

mod imp {

    use super::*;
    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage.ui")]
    pub struct ShaperPage {
        #[template_child]
        pub shaperstyle_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub shaperstyle_image: TemplateChild<Image>,
        #[template_child]
        pub shaperstyle_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub shaperstyle_smooth_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub shaperstyle_rough_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub shapeconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub shapeconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub roughconfig_roughness_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_bowing_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_curvestepcount_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub roughconfig_multistroke_switch: TemplateChild<Switch>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub stroke_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub fill_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub shapebuildertype_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub shapebuildertype_image: TemplateChild<Image>,
        #[template_child]
        pub shapebuildertype_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub shapebuildertype_line_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub shapebuildertype_rectangle_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub shapebuildertype_ellipse_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub shapebuildertype_fociellipse_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub shapebuildertype_quadbez_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub shapebuildertype_cubbez_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub constraint_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub constraint_enabled_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_one_to_one_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_three_to_two_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_golden_switch: TemplateChild<Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShaperPage {
        const NAME: &'static str = "ShaperPage";
        type Type = super::ShaperPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ShaperPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ShaperPage {}
}

glib::wrapper! {
    pub struct ShaperPage(ObjectSubclass<imp::ShaperPage>)
        @extends gtk4::Widget;
}

impl Default for ShaperPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaperPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ShaperPage")
    }

    pub fn shaperstyle_menubutton(&self) -> MenuButton {
        self.imp().shaperstyle_menubutton.get()
    }

    pub fn shaperstyle_image(&self) -> Image {
        self.imp().shaperstyle_image.get()
    }

    pub fn shaperstyle_listbox(&self) -> ListBox {
        self.imp().shaperstyle_listbox.get()
    }

    pub fn shaperstyle_smooth_row(&self) -> adw::ActionRow {
        self.imp().shaperstyle_smooth_row.get()
    }

    pub fn shaperstyle_rough_row(&self) -> adw::ActionRow {
        self.imp().shaperstyle_rough_row.get()
    }

    pub fn shapeconfig_menubutton(&self) -> MenuButton {
        self.imp().shapeconfig_menubutton.get()
    }

    pub fn shapeconfig_popover(&self) -> Popover {
        self.imp().shapeconfig_popover.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        self.imp().width_spinbutton.get()
    }

    pub fn roughconfig_roughness_spinbutton(&self) -> SpinButton {
        self.imp().roughconfig_roughness_spinbutton.get()
    }

    pub fn roughconfig_bowing_spinbutton(&self) -> SpinButton {
        self.imp().roughconfig_bowing_spinbutton.get()
    }

    pub fn roughconfig_curvestepcount_spinbutton(&self) -> SpinButton {
        self.imp().roughconfig_curvestepcount_spinbutton.get()
    }

    pub fn roughconfig_multistroke_switch(&self) -> Switch {
        self.imp().roughconfig_multistroke_switch.get()
    }

    pub fn stroke_colorpicker(&self) -> ColorPicker {
        self.imp().stroke_colorpicker.get()
    }

    pub fn fill_colorpicker(&self) -> ColorPicker {
        self.imp().fill_colorpicker.get()
    }

    pub fn shapebuildertype_menubutton(&self) -> MenuButton {
        self.imp().shapebuildertype_menubutton.get()
    }

    pub fn shapebuildertype_image(&self) -> Image {
        self.imp().shapebuildertype_image.get()
    }

    pub fn shapebuildertype_listbox(&self) -> ListBox {
        self.imp().shapebuildertype_listbox.get()
    }

    pub fn shapebuildertype_line_row(&self) -> adw::ActionRow {
        self.imp().shapebuildertype_line_row.get()
    }

    pub fn shapebuildertype_rectangle_row(&self) -> adw::ActionRow {
        self.imp().shapebuildertype_rectangle_row.get()
    }

    pub fn shapebuildertype_ellipse_row(&self) -> adw::ActionRow {
        self.imp().shapebuildertype_ellipse_row.get()
    }

    pub fn shapebuildertype_fociellipse_row(&self) -> adw::ActionRow {
        self.imp().shapebuildertype_fociellipse_row.get()
    }

    pub fn shapebuildertype_quadbez_row(&self) -> adw::ActionRow {
        self.imp().shapebuildertype_quadbez_row.get()
    }

    pub fn shapebuildertype_cubbez_row(&self) -> adw::ActionRow {
        self.imp().shapebuildertype_cubbez_row.get()
    }

    pub fn constraint_menubutton(&self) -> MenuButton {
        self.imp().shapebuildertype_menubutton.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        shaperstyle_widget::setup(self, appwindow);
        shapeconfiguration_widget::setup(self, appwindow);
        border_width_widget::setup(self, appwindow);
        constraints_widget::setup(self, appwindow);
        stroke_colorpicker_widget::setup(self, appwindow);
        fill_colorpicker_widget::setup(self, appwindow);
        shaperbuilder_widget::setup(self, appwindow);
    }

    pub fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let shaper = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .shaper
            .clone();

        shaperstyle_widget::refresh(self, &shaper);
        constraints_widget::refresh(self, &shaper);
        shaperbuilder_widget::refresh(self, appwindow, &shaper);

        match shaper.style {
            ShaperStyle::Smooth => {
                self.shaperstyle_listbox()
                    .select_row(Some(&self.shaperstyle_smooth_row()));
                self.width_spinbutton()
                    .set_value(shaper.smooth_options.stroke_width);
                self.stroke_colorpicker()
                    .set_current_color(shaper.smooth_options.stroke_color);
                self.fill_colorpicker()
                    .set_current_color(shaper.smooth_options.fill_color);
                self.shaperstyle_image()
                    .set_icon_name(Some("pen-shaper-style-smooth-symbolic"));
            }
            ShaperStyle::Rough => {
                self.shaperstyle_listbox()
                    .select_row(Some(&self.shaperstyle_rough_row()));
                self.width_spinbutton()
                    .set_value(shaper.rough_options.stroke_width);
                self.stroke_colorpicker()
                    .set_current_color(shaper.rough_options.stroke_color);
                self.fill_colorpicker()
                    .set_current_color(shaper.rough_options.fill_color);
                self.shaperstyle_image()
                    .set_icon_name(Some("pen-shaper-style-rough-symbolic"));
            }
        }
    }
}
