// Imports
use crate::{RnAppWindow, RnStrokeWidthPicker};
use adw::prelude::*;
use gtk4::{
    Button, CompositeTemplate, ListBox, MenuButton, Popover, ToggleButton, Widget, glib,
    glib::clone, subclass::prelude::*,
};
use num_traits::cast::ToPrimitive;
use rnote_compose::builders::PenPathBuilderType;
use rnote_compose::style::PressureCurve;
use rnote_compose::style::textured::{TexturedDotsDistribution, TexturedOptions};
use rnote_engine::pens::pensconfig::BrushConfig;
use rnote_engine::pens::pensconfig::brushconfig::{BrushStyle, SolidOptions};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/brushpage.ui")]
    pub(crate) struct RnBrushPage {
        #[template_child]
        pub(crate) brushstyle_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) brushstyle_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) brushstyle_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) brushstyle_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) brushstyle_marker_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brushstyle_solid_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brushstyle_textured_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brushconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) brushconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) brushconfig_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) brush_buildertype_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) brush_buildertype_simple: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brush_buildertype_curved: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brush_buildertype_modeled: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) solidstyle_pressure_curves_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) texturedstyle_density_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) texturedstyle_distribution_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) stroke_width_picker: TemplateChild<RnStrokeWidthPicker>,
        #[template_child]
        pub(crate) ruler_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) ruler_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) ruler_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) ruler_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) ruler_snap_distance_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) ruler_angle_snap_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) ruler_show_dial_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) ruler_width_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) ruler_tick_spacing_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) ruler_body_opacity_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) ruler_scroll_step_row: TemplateChild<adw::SpinRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnBrushPage {
        const NAME: &'static str = "RnBrushPage";
        type Type = super::RnBrushPage;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnBrushPage {
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

    impl WidgetImpl for RnBrushPage {}
}

glib::wrapper! {
    pub(crate) struct RnBrushPage(ObjectSubclass<imp::RnBrushPage>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnBrushPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnBrushPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn brushstyle_menubutton(&self) -> MenuButton {
        self.imp().brushstyle_menubutton.get()
    }

    pub(crate) fn brushconfig_menubutton(&self) -> MenuButton {
        self.imp().brushconfig_menubutton.get()
    }

    pub(crate) fn brush_style(&self) -> Option<BrushStyle> {
        BrushStyle::try_from(self.imp().brushstyle_listbox.selected_row()?.index() as u32).ok()
    }

    pub(crate) fn set_brush_style(&self, brush_style: BrushStyle) {
        match brush_style {
            BrushStyle::Marker => self
                .imp()
                .brushstyle_listbox
                .select_row(Some(&*self.imp().brushstyle_marker_row)),
            BrushStyle::Solid => self
                .imp()
                .brushstyle_listbox
                .select_row(Some(&*self.imp().brushstyle_solid_row)),
            BrushStyle::Textured => self
                .imp()
                .brushstyle_listbox
                .select_row(Some(&*self.imp().brushstyle_textured_row)),
        }
    }

    pub(crate) fn buildertype(&self) -> Option<PenPathBuilderType> {
        PenPathBuilderType::try_from(
            self.imp().brush_buildertype_listbox.selected_row()?.index() as u32
        )
        .ok()
    }

    pub(crate) fn set_buildertype(&self, buildertype: PenPathBuilderType) {
        match buildertype {
            PenPathBuilderType::Simple => self
                .imp()
                .brush_buildertype_listbox
                .select_row(Some(&*self.imp().brush_buildertype_simple)),
            PenPathBuilderType::Curved => self
                .imp()
                .brush_buildertype_listbox
                .select_row(Some(&*self.imp().brush_buildertype_curved)),
            PenPathBuilderType::Modeled => self
                .imp()
                .brush_buildertype_listbox
                .select_row(Some(&*self.imp().brush_buildertype_modeled)),
        }
    }

    pub(crate) fn solidstyle_pressure_curve(&self) -> PressureCurve {
        PressureCurve::try_from(self.imp().solidstyle_pressure_curves_row.get().selected()).unwrap()
    }

    pub(crate) fn set_solidstyle_pressure_curve(&self, pressure_curve: PressureCurve) {
        let position = pressure_curve.to_u32().unwrap();

        self.imp()
            .solidstyle_pressure_curves_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn texturedstyle_dots_distribution(&self) -> TexturedDotsDistribution {
        TexturedDotsDistribution::try_from(
            self.imp().texturedstyle_distribution_row.get().selected(),
        )
        .unwrap()
    }

    pub(crate) fn set_texturedstyle_distribution_variant(
        &self,
        distribution: TexturedDotsDistribution,
    ) {
        let position = distribution.to_u32().unwrap();

        self.imp()
            .texturedstyle_distribution_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn stroke_width_picker(&self) -> RnStrokeWidthPicker {
        self.imp().stroke_width_picker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let brushstyle_popover = imp.brushstyle_popover.get();
        let brushconfig_popover = imp.brushconfig_popover.get();

        // Popovers
        imp.brushstyle_popover_close_button.connect_clicked(clone!(
            #[weak]
            brushstyle_popover,
            move |_| {
                brushstyle_popover.popdown();
            }
        ));
        imp.brushconfig_popover_close_button.connect_clicked(clone!(
            #[weak]
            brushconfig_popover,
            move |_| {
                brushconfig_popover.popdown();
            }
        ));

        // Stroke width
        imp.stroke_width_picker
            .spinbutton()
            .set_range(BrushConfig::STROKE_WIDTH_MIN, BrushConfig::STROKE_WIDTH_MAX);
        // set value after the range!
        imp.stroke_width_picker
            .set_stroke_width(SolidOptions::default().stroke_width);

        imp.stroke_width_picker.connect_notify_local(
            Some("stroke-width"),
            clone!(
                #[weak]
                appwindow,
                move |picker, _| {
                    let stroke_width = picker.stroke_width();
                    let brush_style = appwindow
                        .engine_config()
                        .read()
                        .pens_config
                        .brush_config
                        .style;

                    match brush_style {
                        BrushStyle::Marker => {
                            appwindow
                                .engine_config()
                                .write()
                                .pens_config
                                .brush_config
                                .marker_options
                                .stroke_width = stroke_width;
                        }
                        BrushStyle::Solid => {
                            appwindow
                                .engine_config()
                                .write()
                                .pens_config
                                .brush_config
                                .solid_options
                                .stroke_width = stroke_width;
                        }
                        BrushStyle::Textured => {
                            appwindow
                                .engine_config()
                                .write()
                                .pens_config
                                .brush_config
                                .textured_options
                                .stroke_width = stroke_width;
                        }
                    }
                }
            ),
        );

        // Style
        imp.brushstyle_listbox.connect_row_selected(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |_, _| {
                let Some(brush_style) = brushpage.brush_style() else {
                    return;
                };

                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .style = brush_style;
                brushpage.stroke_width_picker().deselect_setters();

                match brush_style {
                    BrushStyle::Marker => {
                        let stroke_width = appwindow
                            .engine_config()
                            .read()
                            .pens_config
                            .brush_config
                            .marker_options
                            .stroke_width;
                        brushpage
                            .imp()
                            .stroke_width_picker
                            .set_stroke_width(stroke_width);
                        brushpage
                            .imp()
                            .brushstyle_menubutton
                            .set_icon_name("pen-brush-style-marker-symbolic");
                    }
                    BrushStyle::Solid => {
                        let stroke_width = appwindow
                            .engine_config()
                            .read()
                            .pens_config
                            .brush_config
                            .solid_options
                            .stroke_width;
                        brushpage
                            .imp()
                            .stroke_width_picker
                            .set_stroke_width(stroke_width);
                        brushpage
                            .imp()
                            .brushstyle_menubutton
                            .set_icon_name("pen-brush-style-solid-symbolic");
                    }
                    BrushStyle::Textured => {
                        let stroke_width = appwindow
                            .engine_config()
                            .read()
                            .pens_config
                            .brush_config
                            .textured_options
                            .stroke_width;
                        brushpage
                            .imp()
                            .stroke_width_picker
                            .set_stroke_width(stroke_width);
                        brushpage
                            .imp()
                            .brushstyle_menubutton
                            .set_icon_name("pen-brush-style-textured-symbolic");
                    }
                }
            }
        ));

        // Builder type
        imp.brush_buildertype_listbox.connect_row_selected(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |_, _| {
                let Some(buildertype) = brushpage.buildertype() else {
                    return;
                };
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .builder_type = buildertype;
            }
        ));

        // Solid style
        // Pressure curve
        imp.solidstyle_pressure_curves_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=brushpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .brush_config
                        .solid_options
                        .pressure_curve = brushpage.solidstyle_pressure_curve();
                }
            ));

        // Textured style
        // Density
        imp.texturedstyle_density_row
            .get()
            .set_range(TexturedOptions::DENSITY_MIN, TexturedOptions::DENSITY_MAX);
        // set value after the range!
        imp.texturedstyle_density_row
            .get()
            .set_value(TexturedOptions::default().density);

        imp.texturedstyle_density_row.get().connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .textured_options
                    .density = row.value();
            }
        ));

        // dots distribution
        imp.texturedstyle_distribution_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=brushpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .brush_config
                        .textured_options
                        .distribution = brushpage.texturedstyle_dots_distribution();
                }
            ));

        // Ruler toggle: enables/disables the on-canvas ruler. The first time the
        // user enables it, seed the ruler position to the current viewport center.
        imp.ruler_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                let visible = toggle.is_active();
                let needs_seed = {
                    let mut config = appwindow.engine_config().write();
                    let ruler = &mut config.pens_config.brush_config.ruler_config;
                    ruler.visible = visible;
                    visible && ruler.anchor == p2d::math::Vector2::ZERO
                };
                if let Some(canvas) = appwindow.active_tab_canvas() {
                    if needs_seed {
                        // The ruler position is in scroller (window-relative) pixels —
                        // seed it to the center of the visible viewport.
                        let center_scroller = canvas.engine_ref().camera.size() * 0.5;
                        let mut c = appwindow.engine_config().write();
                        let r = &mut c.pens_config.brush_config.ruler_config;
                        r.anchor = center_scroller;
                        r.dial_pos = center_scroller;
                    }
                    canvas.queue_draw();
                }
            }
        ));

        let ruler_popover = imp.ruler_popover.get();
        imp.ruler_popover_close_button.connect_clicked(clone!(
            #[weak]
            ruler_popover,
            move |_| {
                ruler_popover.popdown();
            }
        ));

        imp.ruler_snap_distance_row.get().connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .ruler_config
                    .snap_distance = row.value();
            }
        ));

        imp.ruler_angle_snap_row.get().connect_active_notify(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .ruler_config
                    .angle_snap_enabled = row.is_active();
            }
        ));

        imp.ruler_show_dial_row.get().connect_active_notify(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .ruler_config
                    .show_dial = row.is_active();
                if let Some(canvas) = appwindow.active_tab_canvas() {
                    canvas.queue_draw();
                }
            }
        ));

        // Ruler width: the UI exposes the full width; storage uses the half-width.
        imp.ruler_width_row.get().connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .ruler_config
                    .body_half_width = row.value() * 0.5;
                if let Some(canvas) = appwindow.active_tab_canvas() {
                    canvas.queue_draw();
                }
            }
        ));

        imp.ruler_tick_spacing_row.get().connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .ruler_config
                    .tick_spacing = row.value();
                if let Some(canvas) = appwindow.active_tab_canvas() {
                    canvas.queue_draw();
                }
            }
        ));

        imp.ruler_body_opacity_row.get().connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .ruler_config
                    .body_opacity = row.value();
                if let Some(canvas) = appwindow.active_tab_canvas() {
                    canvas.queue_draw();
                }
            }
        ));

        imp.ruler_scroll_step_row.get().connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .ruler_config
                    .scroll_rotation_step_deg = row.value();
            }
        ));

    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let brush_config = appwindow
            .engine_config()
            .read()
            .pens_config
            .brush_config
            .clone();

        self.set_solidstyle_pressure_curve(brush_config.solid_options.pressure_curve);
        imp.texturedstyle_density_row
            .set_value(brush_config.textured_options.density);
        self.set_texturedstyle_distribution_variant(brush_config.textured_options.distribution);

        self.set_brush_style(brush_config.style);
        self.set_buildertype(brush_config.builder_type);

        match brush_config.style {
            BrushStyle::Marker => {
                imp.stroke_width_picker
                    .set_stroke_width(brush_config.marker_options.stroke_width);
            }
            BrushStyle::Solid => {
                imp.stroke_width_picker
                    .set_stroke_width(brush_config.solid_options.stroke_width);
            }
            BrushStyle::Textured => {
                imp.stroke_width_picker
                    .set_stroke_width(brush_config.textured_options.stroke_width);
            }
        }

        imp.ruler_toggle
            .set_active(brush_config.ruler_config.visible);
        imp.ruler_snap_distance_row
            .set_value(brush_config.ruler_config.snap_distance);
        imp.ruler_angle_snap_row
            .set_active(brush_config.ruler_config.angle_snap_enabled);
        imp.ruler_show_dial_row
            .set_active(brush_config.ruler_config.show_dial);
        imp.ruler_width_row
            .set_value(brush_config.ruler_config.body_half_width * 2.0);
        imp.ruler_tick_spacing_row
            .set_value(brush_config.ruler_config.tick_spacing);
        imp.ruler_body_opacity_row
            .set_value(brush_config.ruler_config.body_opacity);
        imp.ruler_scroll_step_row
            .set_value(brush_config.ruler_config.scroll_rotation_step_deg);
    }
}
