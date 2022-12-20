use crate::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{
    gdk, glib, glib::clone, CompositeTemplate, Image, ListBox, MenuButton, Popover, SpinButton,
    Switch,
};
use num_traits::cast::ToPrimitive;

use rnote_compose::builders::{ConstraintRatio, ShapeBuilderType};
use rnote_compose::style::rough::roughoptions::FillStyle;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_compose::Color;
use rnote_engine::pens::pensconfig::shaperconfig::ShaperStyle;
use rnote_engine::pens::pensconfig::ShaperConfig;
use rnote_engine::utils::GdkRGBAHelpers;

mod imp {

    use super::*;
    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage.ui")]
    pub(crate) struct ShaperPage {
        #[template_child]
        pub(crate) shaperstyle_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) shaperstyle_image: TemplateChild<Image>,
        #[template_child]
        pub(crate) shaperstyle_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) shaperstyle_smooth_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shaperstyle_rough_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapeconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) shapeconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) roughstyle_fillstyle_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) roughstyle_hachure_angle_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) stroke_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub(crate) fill_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub(crate) shapebuildertype_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) shapebuildertype_image: TemplateChild<Image>,
        #[template_child]
        pub(crate) shapebuildertype_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) shapebuildertype_line_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_rectangle_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_grid_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_coordsystem2d_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_coordsystem3d_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_quadrantcoordsystem2d_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_ellipse_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_fociellipse_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_quadbez_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shapebuildertype_cubbez_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) constraint_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) constraint_enabled_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) constraint_one_to_one_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) constraint_three_to_two_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) constraint_golden_switch: TemplateChild<Switch>,
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
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ShaperPage {}
}

glib::wrapper! {
    pub(crate) struct ShaperPage(ObjectSubclass<imp::ShaperPage>)
        @extends gtk4::Widget;
}

impl Default for ShaperPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaperPage {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn shaperstyle_menubutton(&self) -> MenuButton {
        self.imp().shaperstyle_menubutton.get()
    }

    pub(crate) fn shapeconfig_menubutton(&self) -> MenuButton {
        self.imp().shapeconfig_menubutton.get()
    }

    pub(crate) fn stroke_colorpicker(&self) -> ColorPicker {
        self.imp().stroke_colorpicker.get()
    }

    pub(crate) fn fill_colorpicker(&self) -> ColorPicker {
        self.imp().fill_colorpicker.get()
    }

    pub(crate) fn shapebuildertype_menubutton(&self) -> MenuButton {
        self.imp().shapebuildertype_menubutton.get()
    }

    pub(crate) fn constraint_menubutton(&self) -> MenuButton {
        self.imp().shapebuildertype_menubutton.get()
    }

    pub(crate) fn roughstyle_fillstyle(&self) -> FillStyle {
        FillStyle::try_from(self.imp().roughstyle_fillstyle_row.get().selected()).unwrap()
    }

    pub(crate) fn set_roughstyle_fillstyle(&self, fill_style: FillStyle) {
        let position = fill_style.to_u32().unwrap();

        self.imp()
            .roughstyle_fillstyle_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        // Stroke width
        imp.width_spinbutton.set_increments(0.1, 2.0);
        imp.width_spinbutton.set_range(
            ShaperConfig::STROKE_WIDTH_MIN,
            ShaperConfig::STROKE_WIDTH_MAX,
        );
        // set value after the range!
        imp.width_spinbutton
            .get()
            .set_value(SmoothOptions::default().stroke_width);

        imp.width_spinbutton.connect_value_changed(
            clone!(@weak appwindow => move |width_spinbutton| {
                let engine = appwindow.canvas().engine();
                let mut engine = engine.borrow_mut();

                match engine.pens_config.shaper_config.style {
                    ShaperStyle::Smooth => engine.pens_config.shaper_config.smooth_options.stroke_width = width_spinbutton.value(),
                    ShaperStyle::Rough => engine.pens_config.shaper_config.rough_options.stroke_width = width_spinbutton.value(),
                }
            }),
        );

        // Stroke color
        imp.stroke_colorpicker.connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |stroke_colorpicker, _paramspec| {
                let color = stroke_colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();
                let shaper_style = appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.style;

                match shaper_style {
                    ShaperStyle::Smooth => appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.smooth_options.stroke_color = Some(color),
                    ShaperStyle::Rough => appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.rough_options.stroke_color= Some(color),
                }
            }),
        );

        // Fill color
        imp.fill_colorpicker.connect_notify_local(
            Some("current-color"),
            clone!(@weak appwindow => move |fill_colorpicker, _paramspec| {
                let color = fill_colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();

                let engine = appwindow.canvas().engine();
                let engine = &mut *engine.borrow_mut();

                match engine.pens_config.shaper_config.style {
                    ShaperStyle::Smooth => engine.pens_config.shaper_config.smooth_options.fill_color = Some(color),
                    ShaperStyle::Rough => engine.pens_config.shaper_config.rough_options.fill_color= Some(color),
                }
            }),
        );

        // Shaper style
        imp.shaperstyle_listbox.connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_shaperstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();

                        engine.pens_config.shaper_config.style = ShaperStyle::try_from(selected_row.index() as u32).unwrap_or_default();

                        // Overwrite the colors, but not the width
                        match engine.pens_config.shaper_config.style {
                            ShaperStyle::Smooth => {
                                engine.pens_config.shaper_config.smooth_options.stroke_color = Some(shaperpage.stroke_colorpicker().current_color().into_compose_color());
                                engine.pens_config.shaper_config.smooth_options.fill_color = Some(shaperpage.fill_colorpicker().current_color().into_compose_color());
                            },
                            ShaperStyle::Rough => {
                                engine.pens_config.shaper_config.rough_options.stroke_color = Some(shaperpage.stroke_colorpicker().current_color().into_compose_color());
                                engine.pens_config.shaper_config.rough_options.fill_color = Some(shaperpage.fill_colorpicker().current_color().into_compose_color());
                            },
                        }
                    }

                    // Need to refresh the whole page, because changing the style affects multiple widgets
                    shaperpage.refresh_ui(&appwindow);
                }
            }),
        );

        // Rough style
        // Fill style
        imp.roughstyle_fillstyle_row.get().connect_selected_notify(clone!(@weak self as shaperpage, @weak appwindow => move |_roughstyle_fillstyle_row| {
            appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.rough_options.fill_style = shaperpage.roughstyle_fillstyle();
        }));

        // Hachure angle
        imp.roughstyle_hachure_angle_spinbutton.get().connect_value_changed(clone!(@weak self as shaperpage, @weak appwindow => move |spinbutton| {
            appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.rough_options.hachure_angle = spinbutton.value().round().to_radians().clamp(-std::f64::consts::PI, std::f64::consts::PI);
        }));

        // Constraints
        imp
            .constraint_enabled_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.constraints.enabled = switch.state();
            }));

        imp
            .constraint_one_to_one_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.insert(ConstraintRatio::OneToOne);
                } else {
                    appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.remove(&ConstraintRatio::OneToOne);
                }
            }));

        imp
            .constraint_three_to_two_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.insert(ConstraintRatio::ThreeToTwo);
                } else {
                    appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.remove(&ConstraintRatio::ThreeToTwo);
                }
            }));

        imp
            .constraint_golden_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.insert(ConstraintRatio::Golden);
                } else {
                    appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.remove(&ConstraintRatio::Golden);
                }
            }));

        // shape builder type
        imp.shapebuildertype_listbox.connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_shapetype_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    appwindow.canvas().engine().borrow_mut().pens_config.shaper_config.builder_type = ShapeBuilderType::try_from(selected_row.index() as u32).unwrap_or_default();

                    // Need to refresh the whole page, because changing the builder type affects multiple widgets
                    shaperpage.refresh_ui(&appwindow);
                }
            }),
        );
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        let shaper_config = appwindow
            .canvas()
            .engine()
            .borrow()
            .pens_config
            .shaper_config
            .clone();

        match shaper_config.style {
            ShaperStyle::Smooth => {
                imp.shaperstyle_listbox
                    .select_row(Some(&*imp.shaperstyle_smooth_row));
                imp.width_spinbutton
                    .set_value(shaper_config.smooth_options.stroke_width);
                imp.stroke_colorpicker
                    .set_current_color(gdk::RGBA::from_compose_color(
                        shaper_config
                            .smooth_options
                            .stroke_color
                            .unwrap_or(Color::TRANSPARENT),
                    ));
                imp.fill_colorpicker
                    .set_current_color(gdk::RGBA::from_compose_color(
                        shaper_config
                            .smooth_options
                            .fill_color
                            .unwrap_or(Color::TRANSPARENT),
                    ));
                imp.shaperstyle_image
                    .set_icon_name(Some("pen-shaper-style-smooth-symbolic"));
            }
            ShaperStyle::Rough => {
                imp.shaperstyle_listbox
                    .select_row(Some(&*imp.shaperstyle_rough_row));
                imp.width_spinbutton
                    .set_value(shaper_config.rough_options.stroke_width);
                imp.stroke_colorpicker
                    .set_current_color(gdk::RGBA::from_compose_color(
                        shaper_config
                            .rough_options
                            .stroke_color
                            .unwrap_or(Color::TRANSPARENT),
                    ));
                imp.fill_colorpicker
                    .set_current_color(gdk::RGBA::from_compose_color(
                        shaper_config
                            .rough_options
                            .fill_color
                            .unwrap_or(Color::TRANSPARENT),
                    ));
                imp.shaperstyle_image
                    .set_icon_name(Some("pen-shaper-style-rough-symbolic"));
            }
        }

        // Rough style
        self.set_roughstyle_fillstyle(shaper_config.rough_options.fill_style);
        imp.roughstyle_hachure_angle_spinbutton
            .set_value(shaper_config.rough_options.hachure_angle.to_degrees());

        // constraints
        imp.constraint_enabled_switch
            .set_state(shaper_config.constraints.enabled);
        imp.constraint_one_to_one_switch.set_state(
            shaper_config
                .constraints
                .ratios
                .get(&ConstraintRatio::OneToOne)
                .is_some(),
        );
        imp.constraint_three_to_two_switch.set_state(
            shaper_config
                .constraints
                .ratios
                .get(&ConstraintRatio::ThreeToTwo)
                .is_some(),
        );
        imp.constraint_golden_switch.set_state(
            shaper_config
                .constraints
                .ratios
                .get(&ConstraintRatio::Golden)
                .is_some(),
        );

        // builder type
        match shaper_config.builder_type {
            ShapeBuilderType::Line => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_line_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-line-symbolic"));
            }
            ShapeBuilderType::Rectangle => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_rectangle_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-rectangle-symbolic"));
            }
            ShapeBuilderType::Grid => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_grid_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-grid-symbolic"));
            }
            ShapeBuilderType::CoordSystem2D => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_coordsystem2d_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-coordsystem2d-symbolic"));
            }
            ShapeBuilderType::CoordSystem3D => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_coordsystem3d_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-coordsystem3d-symbolic"));
            }
            ShapeBuilderType::QuadrantCoordSystem2D => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_quadrantcoordsystem2d_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-quadrantcoordsystem2d-symbolic"));
            }
            ShapeBuilderType::Ellipse => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_ellipse_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-ellipse-symbolic"));
            }
            ShapeBuilderType::FociEllipse => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_fociellipse_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-fociellipse-symbolic"));
            }
            ShapeBuilderType::QuadBez => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_quadbez_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-quadbez-symbolic"));
            }
            ShapeBuilderType::CubBez => {
                imp.shapebuildertype_listbox
                    .select_row(Some(&*imp.shapebuildertype_cubbez_row));
                imp.shapebuildertype_image
                    .set_icon_name(Some("shapebuilder-cubbez-symbolic"));
            }
        }
    }
}
