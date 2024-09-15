// Imports
use crate::{RnAppWindow, RnCanvasWrapper, RnStrokeWidthPicker};
use adw::prelude::*;
use gtk4::{
    glib, glib::clone, subclass::prelude::*, Button, CompositeTemplate, ListBox, MenuButton,
    Popover,
};
use num_traits::cast::ToPrimitive;
use rnote_compose::builders::PenPathBuilderType;
use rnote_compose::style::textured::{TexturedDotsDistribution, TexturedOptions};
use rnote_compose::style::PressureCurve;
use rnote_engine::pens::pensconfig::brushconfig::{BrushStyle, SolidOptions};
use rnote_engine::pens::pensconfig::BrushConfig;

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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnBrushPage {
        const NAME: &'static str = "RnBrushPage";
        type Type = super::RnBrushPage;
        type ParentType = gtk4::Widget;

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
        @extends gtk4::Widget;
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
                    let canvas = appwindow.active_tab_wrapper().canvas();
                    let brush_style = canvas.engine_ref().pens_config.brush_config.style;

                    match brush_style {
                        BrushStyle::Marker => {
                            canvas
                                .engine_mut()
                                .pens_config
                                .brush_config
                                .marker_options
                                .stroke_width = stroke_width;
                        }
                        BrushStyle::Solid => {
                            canvas
                                .engine_mut()
                                .pens_config
                                .brush_config
                                .solid_options
                                .stroke_width = stroke_width;
                        }
                        BrushStyle::Textured => {
                            canvas
                                .engine_mut()
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
                if let Some(brush_style) = brushpage.brush_style() {
                    appwindow
                        .active_tab_wrapper()
                        .canvas()
                        .engine_mut()
                        .pens_config
                        .brush_config
                        .style = brush_style;
                    brushpage.stroke_width_picker().deselect_setters();

                    match brush_style {
                        BrushStyle::Marker => {
                            let stroke_width = appwindow
                                .active_tab_wrapper()
                                .canvas()
                                .engine_mut()
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
                                .active_tab_wrapper()
                                .canvas()
                                .engine_mut()
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
                                .active_tab_wrapper()
                                .canvas()
                                .engine_mut()
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
            }
        ));

        // Builder type
        imp.brush_buildertype_listbox.connect_row_selected(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |_, _| {
                if let Some(buildertype) = brushpage.buildertype() {
                    appwindow
                        .active_tab_wrapper()
                        .canvas()
                        .engine_mut()
                        .pens_config
                        .brush_config
                        .builder_type = buildertype;
                }
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
                move |_smoothstyle_pressure_curves_row| {
                    appwindow
                        .active_tab_wrapper()
                        .canvas()
                        .engine_mut()
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
                    .active_tab_wrapper()
                    .canvas()
                    .engine_mut()
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
                move |_texturedstyle_distribution_row| {
                    appwindow
                        .active_tab_wrapper()
                        .canvas()
                        .engine_mut()
                        .pens_config
                        .brush_config
                        .textured_options
                        .distribution = brushpage.texturedstyle_dots_distribution();
                }
            ));
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();
        let brush_config = active_tab
            .canvas()
            .engine_ref()
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
    }
}
