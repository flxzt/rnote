// Imports
use crate::{
    RnAppWindow, RnCanvasWrapper, RnGroupedIconPicker, RnStrokeWidthPicker,
    groupediconpicker::GroupedIconPickerGroupData,
};
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk4::{
    Button, CompositeTemplate, ListBox, MenuButton, Popover, StringList, glib, glib::clone,
};
use num_traits::cast::ToPrimitive;
use rnote_compose::builders::ShapeBuilderType;
use rnote_compose::constraints::ConstraintRatio;
use rnote_compose::style::rough::roughoptions::FillStyle;
use rnote_compose::style::smooth::{LineCap, LineStyle, SmoothOptions};
use rnote_engine::pens::pensconfig::ShaperConfig;
use rnote_engine::pens::pensconfig::shaperconfig::{ShaperBrushType, ShaperStyle};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/shaperpage.ui")]
    pub(crate) struct RnShaperPage {
        #[template_child]
        pub(crate) shapebuildertype_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) shapebuildertype_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) shapebuildertype_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) shapebuildertype_picker: TemplateChild<RnGroupedIconPicker>,

        #[template_child]
        pub(crate) shapeconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) shapeconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) shapeconfig_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) brushtype_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) brushtype_solid_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brushtype_highlighter_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) smoothstyle_line_cap_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) smoothstyle_line_style_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) roughstyle_fillstyle_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) roughstyle_hachure_angle_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) constraint_enabled_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) constraint_one_to_one_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) constraint_three_to_two_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) constraint_golden_row: TemplateChild<adw::SwitchRow>,

        #[template_child]
        pub(crate) stroke_width_picker: TemplateChild<RnStrokeWidthPicker>,

        #[template_child]
        pub(crate) shaperstyle_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) shaperstyle_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) shaperstyle_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) shaperstyle_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) shaperstyle_smooth_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) shaperstyle_rough_row: TemplateChild<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnShaperPage {
        const NAME: &'static str = "RnShaperPage";
        type Type = super::RnShaperPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnShaperPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnShaperPage {}
}

glib::wrapper! {
    pub(crate) struct RnShaperPage(ObjectSubclass<imp::RnShaperPage>)
        @extends gtk4::Widget;
}

impl Default for RnShaperPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnShaperPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
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

    pub(crate) fn shaper_style(&self) -> Option<ShaperStyle> {
        ShaperStyle::try_from(self.imp().shaperstyle_listbox.selected_row()?.index() as u32).ok()
    }

    pub(crate) fn shaper_brush_type(&self) -> Option<ShaperBrushType> {
        ShaperBrushType::try_from(self.imp().brushtype_listbox.selected_row()?.index() as u32).ok()
    }

    pub(crate) fn set_shaper_brush_type(&self, shaper_brush_type: ShaperBrushType) {
        match shaper_brush_type {
            ShaperBrushType::Solid => self
                .imp()
                .brushtype_listbox
                .select_row(Some(&*self.imp().brushtype_solid_row)),
            ShaperBrushType::Highlighter => self
                .imp()
                .brushtype_listbox
                .select_row(Some(&*self.imp().brushtype_highlighter_row)),
        }
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
        let icon_name = self.imp().shapebuildertype_picker.picked()?;
        ShapeBuilderType::from_icon_name(icon_name.as_str())
    }

    pub(crate) fn set_shapebuildertype(&self, builder_type: ShapeBuilderType) {
        self.imp()
            .shapebuildertype_picker
            .set_picked(Some(builder_type.to_icon_name()));
    }

    pub(crate) fn smoothstyle_line_cap(&self) -> LineCap {
        LineCap::try_from(self.imp().smoothstyle_line_cap_row.get().selected()).unwrap()
    }

    pub(crate) fn smoothstyle_line_style(&self) -> LineStyle {
        LineStyle::try_from(self.imp().smoothstyle_line_style_row.get().selected()).unwrap()
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

    pub(crate) fn stroke_width_picker(&self) -> RnStrokeWidthPicker {
        self.imp().stroke_width_picker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let shaperstyle_popover = imp.shaperstyle_popover.get();
        let shapeconfig_popover = imp.shapeconfig_popover.get();
        let shapebuildertype_popover = imp.shapebuildertype_popover.get();

        // Popovers
        imp.shaperstyle_popover_close_button.connect_clicked(clone!(
            #[weak]
            shaperstyle_popover,
            move |_| {
                shaperstyle_popover.popdown();
            }
        ));

        imp.shapeconfig_popover_close_button.connect_clicked(clone!(
            #[weak]
            shapeconfig_popover,
            move |_| {
                shapeconfig_popover.popdown();
            }
        ));

        imp.shapebuildertype_popover_close_button
            .connect_clicked(clone!(
                #[weak]
                shapebuildertype_popover,
                move |_| {
                    shapebuildertype_popover.popdown();
                }
            ));

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
            clone!(
                #[weak]
                appwindow,
                move |picker, _| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let stroke_width = picker.stroke_width();
                    let shaper_style = canvas.engine_ref().pens_config.shaper_config.style;

                    match shaper_style {
                        ShaperStyle::Smooth => {
                            let smooth_options: &mut SmoothOptions =
                                &mut canvas.engine_mut().pens_config.shaper_config.smooth_options;
                            smooth_options.stroke_width = stroke_width;
                            smooth_options.update_piet_stroke_style();
                        }
                        ShaperStyle::Rough => {
                            canvas
                                .engine_mut()
                                .pens_config
                                .shaper_config
                                .rough_options
                                .stroke_width = stroke_width;
                        }
                    }
                }
            ),
        );

        // Shaper style
        imp.shaperstyle_listbox.connect_row_selected(clone!(
            #[weak(rename_to=shaperpage)]
            self,
            #[weak]
            appwindow,
            move |_, _| {
                if let Some(shaper_style) = shaperpage.shaper_style() {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    canvas.engine_mut().pens_config.shaper_config.style = shaper_style;
                    shaperpage.stroke_width_picker().deselect_setters();

                    match shaper_style {
                        ShaperStyle::Smooth => {
                            let stroke_width = canvas
                                .engine_mut()
                                .pens_config
                                .shaper_config
                                .smooth_options
                                .stroke_width;
                            shaperpage
                                .imp()
                                .stroke_width_picker
                                .set_stroke_width(stroke_width);
                            shaperpage
                                .imp()
                                .shaperstyle_menubutton
                                .set_icon_name("pen-shaper-style-smooth-symbolic");
                        }
                        ShaperStyle::Rough => {
                            let Some(canvas) = appwindow.active_tab_canvas() else {
                                return;
                            };
                            let stroke_width = canvas
                                .engine_mut()
                                .pens_config
                                .shaper_config
                                .rough_options
                                .stroke_width;

                            shaperpage
                                .imp()
                                .stroke_width_picker
                                .set_stroke_width(stroke_width);
                            shaperpage
                                .imp()
                                .shaperstyle_menubutton
                                .set_icon_name("pen-shaper-style-rough-symbolic");
                        }
                    }
                }
            }
        ));
        // Brush type
        imp.brushtype_listbox.connect_row_selected(clone!(
            #[weak(rename_to=shaperpage)]
            self,
            #[weak]
            appwindow,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                if let Some(brush_type) = shaperpage.shaper_brush_type() {
                    canvas.engine_mut().pens_config.shaper_config.brush_type = brush_type;
                }
            }
        ));

        imp.brushtype_highlighter_row.connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                canvas
                    .engine_mut()
                    .pens_config
                    .shaper_config
                    .highlighter_opacity = row.value().round() / 100.0;
            }
        ));

        // Smooth style
        // Line cap
        imp.smoothstyle_line_cap_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=shaperpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let line_cap = shaperpage.smoothstyle_line_cap();

                    // If the user has selected a straight line cap while the line style was dotted, then we update the line style to be straight
                    if line_cap == LineCap::Straight
                        && shaperpage.smoothstyle_line_style().is_dotted()
                    {
                        shaperpage
                            .imp()
                            .smoothstyle_line_style_row
                            .set_selected(LineStyle::Solid.to_u32().unwrap())
                    }
                    canvas
                        .engine_mut()
                        .pens_config
                        .shaper_config
                        .smooth_options
                        .update_line_cap(line_cap);
                }
            ));

        // Line style
        imp.smoothstyle_line_style_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=shaperpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let line_style = shaperpage.smoothstyle_line_style();

                    // If the user has selected a dotted line style, then we update the line cap to be rounded
                    if line_style.is_dotted() {
                        shaperpage
                            .imp()
                            .smoothstyle_line_cap_row
                            .set_selected(LineCap::Rounded.to_u32().unwrap());
                    }
                    canvas
                        .engine_mut()
                        .pens_config
                        .shaper_config
                        .smooth_options
                        .update_line_style(line_style);
                }
            ));

        // Rough style
        // Fill style
        imp.roughstyle_fillstyle_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=shaperpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    canvas
                        .engine_mut()
                        .pens_config
                        .shaper_config
                        .rough_options
                        .fill_style = shaperpage.roughstyle_fillstyle();
                }
            ));

        // Hachure angle
        imp.roughstyle_hachure_angle_row
            .get()
            .connect_changed(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    canvas
                        .engine_mut()
                        .pens_config
                        .shaper_config
                        .rough_options
                        .hachure_angle = row
                        .value()
                        .round()
                        .to_radians()
                        .clamp(-std::f64::consts::PI, std::f64::consts::PI);
                }
            ));

        // shape builder type
        imp.shapebuildertype_picker.set_groups(
            shape_builder_type_icons_get_groups(),
            shape_builder_type_icons_to_display_name,
        );

        imp.shapebuildertype_picker.connect_notify_local(
            Some("picked"),
            clone!(
                #[weak(rename_to=shaperpage)]
                self,
                #[weak]
                appwindow,
                move |picker, _| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    if let (Some(buildertype), Some(icon_name)) =
                        (shaperpage.shapebuildertype(), picker.picked())
                    {
                        canvas.engine_mut().pens_config.shaper_config.builder_type = buildertype;
                        shaperpage
                            .imp()
                            .shapebuildertype_menubutton
                            .set_icon_name(&icon_name);
                    }
                }
            ),
        );

        // Constraints
        imp.constraint_enabled_row
            .get()
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    canvas
                        .engine_mut()
                        .pens_config
                        .shaper_config
                        .constraints
                        .enabled = row.is_active();
                }
            ));

        imp.constraint_one_to_one_row
            .get()
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    if row.is_active() {
                        canvas
                            .engine_mut()
                            .pens_config
                            .shaper_config
                            .constraints
                            .ratios
                            .insert(ConstraintRatio::OneToOne);
                    } else {
                        canvas
                            .engine_mut()
                            .pens_config
                            .shaper_config
                            .constraints
                            .ratios
                            .remove(&ConstraintRatio::OneToOne);
                    }
                }
            ));

        imp.constraint_three_to_two_row
            .get()
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    if row.is_active() {
                        canvas
                            .engine_mut()
                            .pens_config
                            .shaper_config
                            .constraints
                            .ratios
                            .insert(ConstraintRatio::ThreeToTwo);
                    } else {
                        canvas
                            .engine_mut()
                            .pens_config
                            .shaper_config
                            .constraints
                            .ratios
                            .remove(&ConstraintRatio::ThreeToTwo);
                    }
                }
            ));

        imp.constraint_golden_row
            .get()
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    if row.is_active() {
                        canvas
                            .engine_mut()
                            .pens_config
                            .shaper_config
                            .constraints
                            .ratios
                            .insert(ConstraintRatio::Golden);
                    } else {
                        canvas
                            .engine_mut()
                            .pens_config
                            .shaper_config
                            .constraints
                            .ratios
                            .remove(&ConstraintRatio::Golden);
                    }
                }
            ));
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();

        let shaper_config = active_tab
            .canvas()
            .engine_ref()
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

        // Brush type
        self.set_shaper_brush_type(shaper_config.brush_type);
        imp.brushtype_highlighter_row
            .set_value((shaper_config.highlighter_opacity * 100.0).round());

        // Smooth style
        imp.smoothstyle_line_cap_row
            .set_selected(shaper_config.smooth_options.line_cap.to_u32().unwrap());
        imp.smoothstyle_line_style_row
            .set_selected(shaper_config.smooth_options.line_style.to_u32().unwrap());

        // Rough style
        self.set_roughstyle_fillstyle(shaper_config.rough_options.fill_style);
        imp.roughstyle_hachure_angle_row
            .set_value(shaper_config.rough_options.hachure_angle.to_degrees());

        // constraints
        imp.constraint_enabled_row
            .set_active(shaper_config.constraints.enabled);
        imp.constraint_one_to_one_row.set_active(
            shaper_config
                .constraints
                .ratios
                .contains(&ConstraintRatio::OneToOne),
        );
        imp.constraint_three_to_two_row.set_active(
            shaper_config
                .constraints
                .ratios
                .contains(&ConstraintRatio::ThreeToTwo),
        );
        imp.constraint_golden_row.set_active(
            shaper_config
                .constraints
                .ratios
                .contains(&ConstraintRatio::Golden),
        );
    }
}

fn shape_builder_type_icons_get_groups() -> Vec<GroupedIconPickerGroupData> {
    vec![
        GroupedIconPickerGroupData {
            name: gettext("Miscellaneous"),
            icons: StringList::new(&[
                "shapebuilder-line-symbolic",
                "shapebuilder-arrow-symbolic",
                "shapebuilder-rectangle-symbolic",
                "shapebuilder-grid-symbolic",
            ]),
        },
        GroupedIconPickerGroupData {
            name: gettext("Coordinate Systems"),
            icons: StringList::new(&[
                "shapebuilder-coordsystem2d-symbolic",
                "shapebuilder-coordsystem3d-symbolic",
                "shapebuilder-quadrantcoordsystem2d-symbolic",
            ]),
        },
        GroupedIconPickerGroupData {
            name: gettext("Ellipses"),
            icons: StringList::new(&[
                "shapebuilder-ellipse-symbolic",
                "shapebuilder-fociellipse-symbolic",
            ]),
        },
        GroupedIconPickerGroupData {
            name: gettext("Curves & Paths"),
            icons: StringList::new(&[
                "shapebuilder-quadbez-symbolic",
                "shapebuilder-cubbez-symbolic",
                "shapebuilder-polyline-symbolic",
                "shapebuilder-polygon-symbolic",
            ]),
        },
    ]
}

fn shape_builder_type_icons_to_display_name(icon_name: &str) -> String {
    match ShapeBuilderType::from_icon_name(icon_name)
        .expect("ShapeBuilderTypePicker failed, display name of unknown icon name requested")
    {
        ShapeBuilderType::Arrow => gettext("Arrow"),
        ShapeBuilderType::Line => gettext("Line"),
        ShapeBuilderType::Rectangle => gettext("Rectangle"),
        ShapeBuilderType::Grid => gettext("Grid"),
        ShapeBuilderType::CoordSystem2D => gettext("2D coordinate system"),
        ShapeBuilderType::CoordSystem3D => gettext("3D coordinate system"),
        ShapeBuilderType::QuadrantCoordSystem2D => gettext("2D single quadrant coordinate system"),
        ShapeBuilderType::Ellipse => gettext("Ellipse"),
        ShapeBuilderType::FociEllipse => gettext("Ellipse with foci"),
        ShapeBuilderType::QuadBez => gettext("Quadratic bezier curve"),
        ShapeBuilderType::CubBez => gettext("Cubic bezier curve"),
        ShapeBuilderType::Polyline => gettext("Polyline"),
        ShapeBuilderType::Polygon => gettext("Polygon"),
    }
}
