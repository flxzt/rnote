use crate::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Image, ListBox,
    MenuButton, Popover, SpinButton, Switch,
};
use rnote_compose::builders::{ConstraintRatio, ShapeBuilderType};
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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shape width, Err `{}`", e);
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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper color, Err `{}`", e);
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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper fill color, Err `{}`", e);
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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing rough shape roughness, Err `{}`", e);
                }
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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing rough shape bowing, Err `{}`", e);
                }
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

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing rough shape curve stepcount, Err `{}`", e);
                }
            }),
        );

        // Multistroke
        self.imp().roughconfig_multistroke_switch.get().connect_state_notify(clone!(@weak appwindow => move |roughconfig_multistroke_switch| {
            appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.disable_multistroke = !roughconfig_multistroke_switch.state();
        }));

        // Shaper style
        self.shaperstyle_listbox().connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_shaperstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();

                        engine.penholder.shaper.style = ShaperStyle::try_from(selected_row.index() as u32).unwrap_or_default();
                        engine.penholder.shaper.smooth_options.stroke_width = shaperpage.width_spinbutton().value();
                        engine.penholder.shaper.smooth_options.stroke_color = Some(shaperpage.stroke_colorpicker().current_color());
                        engine.penholder.shaper.smooth_options.fill_color = Some(shaperpage.fill_colorpicker().current_color());
                        engine.penholder.shaper.rough_options.stroke_width = shaperpage.width_spinbutton().value();
                        engine.penholder.shaper.rough_options.stroke_color = Some(shaperpage.stroke_colorpicker().current_color());
                        engine.penholder.shaper.rough_options.fill_color = Some(shaperpage.fill_colorpicker().current_color());
                    }

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing shaper style, Err `{}`", e);
                    }
                    // Need to refresh the whole page, because changing the style affects multiple widgets
                    shaperpage.refresh_ui(&appwindow);
                }
            }),
        );

        // Constraints
        self.imp()
            .constraint_enabled_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.enabled = switch.state();

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper constraint enabled, Err `{}`", e);
                }
            }));

        self.imp()
            .constraint_one_to_one_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.insert(ConstraintRatio::OneToOne);
                } else {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.remove(&ConstraintRatio::OneToOne);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper one to one constraint, Err `{}`", e);
                }
            }));

        self.imp()
            .constraint_three_to_two_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.insert(ConstraintRatio::ThreeToTwo);
                } else {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.remove(&ConstraintRatio::ThreeToTwo);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper three to two constraint, Err `{}`", e);
                }
            }));

        self.imp()
            .constraint_golden_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.insert(ConstraintRatio::Golden);
                } else {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.remove(&ConstraintRatio::Golden);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper golden ratio constraint, Err `{}`", e);
                }
            }));

        // shape builder type
        self.shapebuildertype_listbox().connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_shapetype_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::try_from(selected_row.index() as u32).unwrap_or_default();

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing shape builder type, Err `{}`", e);
                    }
                    // Need to refresh the whole page, because changing the builder type affects multiple widgets
                    shaperpage.refresh_ui(&appwindow);
                }
            }),
        );
    }

    pub fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        // Avoiding borrow errors by cloning. Unfortunately we can't clone the entire shaper struct, because it holds the builder trait object
        let builder_type = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .shaper
            .builder_type
            .clone();
        let style = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .shaper
            .style
            .clone();
        let rough_options = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .shaper
            .rough_options
            .clone();
        let smooth_options = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .shaper
            .smooth_options
            .clone();
        let constraints = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .shaper
            .constraints
            .clone();

        // style config
        self.roughconfig_roughness_spinbutton()
            .set_value(rough_options.roughness);
        self.roughconfig_bowing_spinbutton()
            .set_value(rough_options.bowing);
        self.roughconfig_curvestepcount_spinbutton()
            .set_value(rough_options.curve_stepcount);
        self.roughconfig_multistroke_switch()
            .set_active(!rough_options.disable_multistroke);

        // constraints
        self.imp()
            .constraint_enabled_switch
            .set_state(constraints.enabled);
        self.imp()
            .constraint_one_to_one_switch
            .set_state(constraints.ratios.get(&ConstraintRatio::OneToOne).is_some());
        self.imp().constraint_three_to_two_switch.set_state(
            constraints
                .ratios
                .get(&ConstraintRatio::ThreeToTwo)
                .is_some(),
        );
        self.imp()
            .constraint_golden_switch
            .set_state(constraints.ratios.get(&ConstraintRatio::Golden).is_some());

        // builder type
        match builder_type {
            ShapeBuilderType::Line => {
                self.shapebuildertype_listbox().select_row(Some(
                    &appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapebuildertype_line_row(),
                ));
                self.shapebuildertype_image()
                    .set_icon_name(Some("shape-line-symbolic"));
            }
            ShapeBuilderType::Rectangle => {
                self.shapebuildertype_listbox().select_row(Some(
                    &appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapebuildertype_rectangle_row(),
                ));
                self.shapebuildertype_image()
                    .set_icon_name(Some("shape-rectangle-symbolic"));
            }
            ShapeBuilderType::Ellipse => {
                self.shapebuildertype_listbox()
                    .select_row(Some(&self.shapebuildertype_ellipse_row()));
                self.shapebuildertype_image()
                    .set_icon_name(Some("shape-ellipse-symbolic"));
            }
            ShapeBuilderType::FociEllipse => {
                self.shapebuildertype_listbox()
                    .select_row(Some(&self.shapebuildertype_fociellipse_row()));
                self.shapebuildertype_image()
                    .set_icon_name(Some("shape-fociellipse-symbolic"));
            }
            ShapeBuilderType::QuadBez => {
                self.shapebuildertype_listbox().select_row(Some(
                    &appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapebuildertype_quadbez_row(),
                ));
                self.shapebuildertype_image()
                    .set_icon_name(Some("shape-quadbez-symbolic"));
            }
            ShapeBuilderType::CubBez => {
                self.shapebuildertype_listbox().select_row(Some(
                    &appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapebuildertype_cubbez_row(),
                ));
                self.shapebuildertype_image()
                    .set_icon_name(Some("shape-cubbez-symbolic"));
            }
        }

        match style {
            ShaperStyle::Smooth => {
                self.shaperstyle_listbox()
                    .select_row(Some(&self.shaperstyle_smooth_row()));
                self.width_spinbutton()
                    .set_value(smooth_options.stroke_width);
                self.stroke_colorpicker()
                    .set_current_color(smooth_options.stroke_color);
                self.fill_colorpicker()
                    .set_current_color(smooth_options.fill_color);
                self.shaperstyle_image()
                    .set_icon_name(Some("pen-shaper-style-smooth-symbolic"));
            }
            ShaperStyle::Rough => {
                self.shaperstyle_listbox()
                    .select_row(Some(&self.shaperstyle_rough_row()));
                self.width_spinbutton()
                    .set_value(rough_options.stroke_width);
                self.stroke_colorpicker()
                    .set_current_color(rough_options.stroke_color);
                self.fill_colorpicker()
                    .set_current_color(rough_options.fill_color);
                self.shaperstyle_image()
                    .set_icon_name(Some("pen-shaper-style-rough-symbolic"));
            }
        }
    }
}
