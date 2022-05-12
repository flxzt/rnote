use crate::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Image, ListBox,
    MenuButton, Popover, SpinButton, Switch,
};
use rnote_compose::builders::ConstraintRatio;
use rnote_compose::style::rough::RoughOptions;
use rnote_engine::pens::shaper::ShaperStyle;
use rnote_engine::pens::Shaper;
use rnote_engine::utils::GdkRGBAHelpers;

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
        pub constraint_enabled_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_one_to_one_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_three_to_two_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_golden_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_horizontal_switch: TemplateChild<Switch>,
        #[template_child]
        pub constraint_vertical_switch: TemplateChild<Switch>,
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

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        // Width
        self.width_spinbutton().set_increments(0.1, 2.0);
        self.width_spinbutton()
            .set_range(Shaper::STROKE_WIDTH_MIN, Shaper::STROKE_WIDTH_MAX);
        // Must be set after set_range()
        self.width_spinbutton()
            .set_value(Shaper::STROKE_WIDTH_DEFAULT);

        self.width_spinbutton().connect_value_changed(
            clone!(@weak appwindow => move |width_spinbutton| {
                let shaper_style = appwindow.canvas().engine().borrow_mut().penholder.shaper.style;

                match shaper_style {
                    ShaperStyle::Smooth => appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.stroke_width = width_spinbutton.value(),
                    ShaperStyle::Rough => appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.stroke_width = width_spinbutton.value(),
                }
            }),
        );

        // Stroke color
        self.stroke_colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
                let color = stroke_colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();
                let shaper_style = appwindow.canvas().engine().borrow_mut().penholder.shaper.style;

                match shaper_style {
                    ShaperStyle::Smooth => appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.stroke_color = Some(color),
                    ShaperStyle::Rough => appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.stroke_color= Some(color),
                }
            }),
        );

        // Fill color
        self.fill_colorpicker().connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |fill_colorpicker, _paramspec| {
                let color = fill_colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();
                let shaper_style = appwindow.canvas().engine().borrow_mut().penholder.shaper.style;

                match shaper_style {
                    ShaperStyle::Smooth => appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.fill_color = Some(color),
                    ShaperStyle::Rough => appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.fill_color= Some(color),
                }
            }),
        );

        // Roughness
        self.imp()
            .roughconfig_roughness_spinbutton
            .get()
            .set_increments(0.1, 2.0);
        self.imp()
            .roughconfig_roughness_spinbutton
            .get()
            .set_range(RoughOptions::ROUGHNESS_MIN, RoughOptions::ROUGHNESS_MAX);
        self.imp()
            .roughconfig_roughness_spinbutton
            .get()
            .set_value(RoughOptions::ROUGHNESS_DEFAULT);

        self.imp().roughconfig_roughness_spinbutton.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_roughness_spinbutton| {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.roughness = roughconfig_roughness_spinbutton.value();
            }),
        );

        // Bowing
        self.imp()
            .roughconfig_bowing_spinbutton
            .get()
            .set_increments(0.1, 2.0);
        self.imp()
            .roughconfig_bowing_spinbutton
            .get()
            .set_range(RoughOptions::BOWING_MIN, RoughOptions::BOWING_MAX);
        self.imp()
            .roughconfig_bowing_spinbutton
            .get()
            .set_value(RoughOptions::BOWING_DEFAULT);

        self.imp().roughconfig_bowing_spinbutton.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_bowing_spinbutton| {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.bowing = roughconfig_bowing_spinbutton.value();
            }),
        );

        // Curve stepcount
        self.imp()
            .roughconfig_curvestepcount_spinbutton
            .get()
            .set_increments(1.0, 2.0);
        self.imp()
            .roughconfig_curvestepcount_spinbutton
            .get()
            .set_range(
                RoughOptions::CURVESTEPCOUNT_MIN,
                RoughOptions::CURVESTEPCOUNT_MAX,
            );
        self.imp()
            .roughconfig_curvestepcount_spinbutton
            .get()
            .set_value(RoughOptions::CURVESTEPCOUNT_DEFAULT);

        self.imp().roughconfig_curvestepcount_spinbutton.get().connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_curvestepcount_spinbutton| {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.curve_stepcount = roughconfig_curvestepcount_spinbutton.value();
            }),
        );

        // Multistroke
        self.imp().roughconfig_multistroke_switch.get().connect_state_notify(clone!(@weak appwindow => move |roughconfig_multistroke_switch| {
            appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.disable_multistroke = !roughconfig_multistroke_switch.state();
        }));

        // Smooth / Rough shaper style
        self.shaperstyle_listbox().connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_shaperstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    match selected_row.index() {
                        // Smooth
                        0 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shaper-style", Some(&"smooth".to_variant()));
                        }
                        // Rough
                        1 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shaper-style", Some(&"rough".to_variant()));
                        }
                        _ => {}
                    }
                }
            }),
        );

        // Constraints
        self.imp()
            .constraint_enabled_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.constraint.enabled = switch.state();
            }));

        self.imp()
            .constraint_one_to_one_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.constraint.ratio.insert(ConstraintRatio::OneToOne, switch.state());
            }));

        self.imp()
            .constraint_three_to_two_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.constraint.ratio.insert(ConstraintRatio::ThreeToTwo, switch.state());
            }));

        self.imp()
            .constraint_golden_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.constraint.ratio.insert(ConstraintRatio::Golden, switch.state());
            }));

        self.imp()
            .constraint_horizontal_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.constraint.ratio.insert(ConstraintRatio::Horizontal, switch.state());
            }));

        self.imp()
            .constraint_vertical_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.constraint.ratio.insert(ConstraintRatio::Vertical, switch.state());
            }));

        //self.constraint_ratio_combo().connect_selected_item_notify(
        //    clone!(@weak appwindow => move |combo| {
        //        let ratio = match combo.selected() {
        //            0 => ConstraintRatio::Disabled,
        //            1 => ConstraintRatio::OneToOne,
        //            2 => ConstraintRatio::ThreeToTwo,
        //            3 => ConstraintRatio::Golden,
        //            _ => unreachable!()
        //        }
        //        appwindow.canvas().engine().borrow_mut().penholder.shaper.ratio = ratio;
        //    }),
        //);

        // shape builder type
        self.shapebuildertype_listbox().connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_shapetype_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    match selected_row.index() {
                        // Line
                        0 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shape-buildertype", Some(&"line".to_variant()));
                        }
                        // Rectangle
                        1 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shape-buildertype", Some(&"rectangle".to_variant()));
                        }
                        // Ellipse
                        2 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shape-buildertype", Some(&"ellipse".to_variant()));
                        }
                        // Foci ellipse
                        3 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shape-buildertype", Some(&"fociellipse".to_variant()));
                        }
                        // Quadbez
                        4 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shape-buildertype", Some(&"quadbez".to_variant()));
                        }
                        // Cubbez
                        5 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "shape-buildertype", Some(&"cubbez".to_variant()));
                        }
                        _ => {}
                    }
                }
            }),
        );
    }
}
