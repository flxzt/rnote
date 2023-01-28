use adw::{prelude::*, subclass::prelude::*};
use gtk4::{
    glib, glib::clone, CompositeTemplate, ListBox, MenuButton, Popover, SpinButton, Switch,
};
use num_traits::cast::ToPrimitive;
use rnote_compose::builders::{ConstraintRatio, ShapeBuilderType};
use rnote_compose::style::rough::roughoptions::FillStyle;
use rnote_compose::style::smooth::SmoothOptions;
use rnote_engine::pens::pensconfig::shaperconfig::ShaperStyle;
use rnote_engine::pens::pensconfig::ShaperConfig;

use crate::{RnoteAppWindow, RnoteCanvasWrapper, StrokeWidthPicker};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage.ui")]
    pub(crate) struct ShaperPage {
        #[template_child]
        pub(crate) shaperstyle_menubutton: TemplateChild<MenuButton>,
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
        pub(crate) stroke_width_picker: TemplateChild<StrokeWidthPicker>,
        #[template_child]
        pub(crate) shapebuildertype_menubutton: TemplateChild<MenuButton>,
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

    pub(crate) fn shapebuildertype_menubutton(&self) -> MenuButton {
        self.imp().shapebuildertype_menubutton.get()
    }

    pub(crate) fn constraint_menubutton(&self) -> MenuButton {
        self.imp().shapebuildertype_menubutton.get()
    }

    pub(crate) fn shaper_style(&self) -> Option<ShaperStyle> {
        ShaperStyle::try_from(self.imp().shaperstyle_listbox.selected_row()?.index() as u32).ok()
    }

    pub(crate) fn set_shaper_style(&self, style: ShaperStyle) {
        match style {
            ShaperStyle::Smooth => self
                .imp()
                .shaperstyle_listbox
                .select_row(Some(&*self.imp().shaperstyle_smooth_row)),
            ShaperStyle::Rough => self
                .imp()
                .shaperstyle_listbox
                .select_row(Some(&*self.imp().shaperstyle_rough_row)),
        }
    }

    pub(crate) fn shapebuildertype(&self) -> Option<ShapeBuilderType> {
        ShapeBuilderType::try_from(
            self.imp().shapebuildertype_listbox.selected_row()?.index() as u32
        )
        .ok()
    }

    pub(crate) fn set_shapebuildertype(&self, buildertype: ShapeBuilderType) {
        match buildertype {
            ShapeBuilderType::Line => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_line_row)),
            ShapeBuilderType::Rectangle => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_rectangle_row)),
            ShapeBuilderType::Grid => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_grid_row)),
            ShapeBuilderType::CoordSystem2D => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_coordsystem2d_row)),
            ShapeBuilderType::CoordSystem3D => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_coordsystem3d_row)),
            ShapeBuilderType::QuadrantCoordSystem2D => {
                self.imp().shapebuildertype_listbox.select_row(Some(
                    &*self.imp().shapebuildertype_quadrantcoordsystem2d_row,
                ))
            }
            ShapeBuilderType::Ellipse => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_ellipse_row)),
            ShapeBuilderType::FociEllipse => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_fociellipse_row)),
            ShapeBuilderType::QuadBez => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_quadbez_row)),
            ShapeBuilderType::CubBez => self
                .imp()
                .shapebuildertype_listbox
                .select_row(Some(&*self.imp().shapebuildertype_cubbez_row)),
        }
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

    pub(crate) fn stroke_width_picker(&self) -> StrokeWidthPicker {
        self.imp().stroke_width_picker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        // Stroke width
        imp.stroke_width_picker.spinbutton().set_range(
            ShaperConfig::STROKE_WIDTH_MIN,
            ShaperConfig::STROKE_WIDTH_MAX,
        );
        // set value after the range!
        imp.stroke_width_picker
            .set_stroke_width(SmoothOptions::default().stroke_width);

        imp.stroke_width_picker.connect_notify_local(
            Some("stroke-width"),
            clone!(@weak self as shaperpage, @weak appwindow => move |picker, _| {
                let stroke_width = picker.stroke_width();
                let engine = appwindow.active_tab().canvas().engine();
                let mut engine = engine.borrow_mut();

                match engine.pens_config.shaper_config.style {
                    ShaperStyle::Smooth => {
                        engine.pens_config.shaper_config.smooth_options.stroke_width = stroke_width;
                    },
                    ShaperStyle::Rough => {
                        engine.pens_config.shaper_config.rough_options.stroke_width = stroke_width;
                    },
                }
            }),
        );

        // Shaper style
        imp.shaperstyle_listbox.connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_, _| {
                if let Some(shaper_style) = shaperpage.shaper_style() {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.style = shaper_style;
                    shaperpage.stroke_width_picker().deselect_setters();

                    match shaper_style {
                        ShaperStyle::Smooth => {
                            let stroke_width = appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.smooth_options.stroke_width;
                            shaperpage.imp().stroke_width_picker.set_stroke_width(stroke_width);
                            shaperpage.imp().shaperstyle_menubutton.set_icon_name("pen-shaper-style-smooth-symbolic");
                        },
                        ShaperStyle::Rough => {
                            let stroke_width = appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.rough_options.stroke_width;
                            shaperpage.imp().stroke_width_picker.set_stroke_width(stroke_width);
                            shaperpage.imp().shaperstyle_menubutton.set_icon_name("pen-shaper-style-rough-symbolic");
                        },
                    }
                }
            }),
        );

        // Rough style
        // Fill style
        imp.roughstyle_fillstyle_row.get().connect_selected_notify(clone!(@weak self as shaperpage, @weak appwindow => move |_roughstyle_fillstyle_row| {
            appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.rough_options.fill_style = shaperpage.roughstyle_fillstyle();
        }));

        // Hachure angle
        imp.roughstyle_hachure_angle_spinbutton.get().connect_value_changed(clone!(@weak self as shaperpage, @weak appwindow => move |spinbutton| {
            appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.rough_options.hachure_angle = spinbutton.value().round().to_radians().clamp(-std::f64::consts::PI, std::f64::consts::PI);
        }));

        // shape builder type
        imp.shapebuildertype_listbox.connect_row_selected(
            clone!(@weak self as shaperpage, @weak appwindow => move |_, _| {
                if let Some(buildertype) = shaperpage.shapebuildertype() {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.builder_type = buildertype;

                    match buildertype {
                        ShapeBuilderType::Line => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-line-symbolic"),
                        ShapeBuilderType::Rectangle => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-rectangle-symbolic"),
                        ShapeBuilderType::Grid => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-grid-symbolic"),
                        ShapeBuilderType::CoordSystem2D => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-coordsystem2d-symbolic"),
                        ShapeBuilderType::CoordSystem3D => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-coordsystem3d-symbolic"),
                        ShapeBuilderType::QuadrantCoordSystem2D => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-quadrantcoordsystem2d-symbolic"),
                        ShapeBuilderType::Ellipse => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-ellipse-symbolic"),
                        ShapeBuilderType::FociEllipse => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-fociellipse-symbolic"),
                        ShapeBuilderType::QuadBez => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-quadbez-symbolic"),
                        ShapeBuilderType::CubBez => shaperpage.imp().shapebuildertype_menubutton.set_icon_name("shapebuilder-cubbez-symbolic"),
                    }
                }
            }),
        );

        // Constraints
        imp
            .constraint_enabled_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.constraints.enabled = switch.state();
            }));

        imp
            .constraint_one_to_one_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.insert(ConstraintRatio::OneToOne);
                } else {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.remove(&ConstraintRatio::OneToOne);
                }
            }));

        imp
            .constraint_three_to_two_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.insert(ConstraintRatio::ThreeToTwo);
                } else {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.remove(&ConstraintRatio::ThreeToTwo);
                }
            }));

        imp
            .constraint_golden_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.insert(ConstraintRatio::Golden);
                } else {
                    appwindow.active_tab().canvas().engine().borrow_mut().pens_config.shaper_config.constraints.ratios.remove(&ConstraintRatio::Golden);
                }
            }));
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnoteCanvasWrapper) {
        let imp = self.imp();

        let shaper_config = active_tab
            .canvas()
            .engine()
            .borrow()
            .pens_config
            .shaper_config
            .clone();

        self.set_shaper_style(shaper_config.style);

        match shaper_config.style {
            ShaperStyle::Smooth => {
                imp.stroke_width_picker
                    .set_stroke_width(shaper_config.smooth_options.stroke_width);
            }
            ShaperStyle::Rough => {
                imp.stroke_width_picker
                    .set_stroke_width(shaper_config.rough_options.stroke_width);
            }
        }

        // builder type
        self.set_shapebuildertype(shaper_config.builder_type);

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
                .contains(&ConstraintRatio::OneToOne),
        );
        imp.constraint_three_to_two_switch.set_state(
            shaper_config
                .constraints
                .ratios
                .contains(&ConstraintRatio::ThreeToTwo),
        );
        imp.constraint_golden_switch.set_state(
            shaper_config
                .constraints
                .ratios
                .contains(&ConstraintRatio::Golden),
        );
    }
}
