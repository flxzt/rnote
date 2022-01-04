mod imp {
    use adw::prelude::*;
    use gtk4::{glib, glib::clone, subclass::prelude::*, CompositeTemplate};
    use gtk4::{Adjustment, Button, ColorButton, ToggleButton};

    use crate::sheet::format::{self, Format};
    use crate::ui::unitentry::UnitEntry;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/settingspanel.ui")]
    pub struct SettingsPanel {
        pub temporary_format: Format,
        #[template_child]
        pub general_sheet_margin_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub general_pdf_import_width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub general_pdf_import_as_vector_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub general_pdf_import_as_bitmap_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub format_predefined_formats_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub format_orientation_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub format_orientation_portrait_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub format_orientation_landscape_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub format_width_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub format_width_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub format_height_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub format_height_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub format_dpi_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub format_dpi_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub format_revert_button: TemplateChild<Button>,
        #[template_child]
        pub format_apply_button: TemplateChild<Button>,
        #[template_child]
        pub background_color_choosebutton: TemplateChild<ColorButton>,
        #[template_child]
        pub background_patterns_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub background_pattern_color_choosebutton: TemplateChild<ColorButton>,
        #[template_child]
        pub background_pattern_width_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub background_pattern_height_unitentry: TemplateChild<UnitEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsPanel {
        const NAME: &'static str = "SettingsPanel";
        type Type = super::SettingsPanel;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsPanel {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            /*             self.general_sheet_margin_unitentry
                .get()
                .value_adj()
                .set_lower(0.0);
            self.general_sheet_margin_unitentry
                .get()
                .value_spinner()
                .set_increments(1.0, 10.0);
            self.general_sheet_margin_unitentry
                .get()
                .value_spinner()
                .set_digits(1); */

            self.format_predefined_formats_row
                .connect_selected_item_notify(
                    clone!(@weak obj => move |_format_predefined_formats_row| {
                        obj.update_temporary_format_from_rows();
                        Self::apply_predefined_format(&obj);
                    }),
                );

            self.format_orientation_portrait_toggle.connect_toggled(
                clone!(@weak obj => move |_format_orientation_portrait_toggle| {
                    obj.update_temporary_format_from_rows();
                    Self::apply_predefined_format(&obj);
                }),
            );

            self.format_orientation_landscape_toggle.connect_toggled(
                clone!(@weak obj => move |_format_orientation_landscape_toggle| {
                    obj.update_temporary_format_from_rows();
                    Self::apply_predefined_format(&obj);
                }),
            );

            self.format_width_unitentry.get().value_adj().set_lower(1.0);
            self.format_width_unitentry
                .get()
                .value_spinner()
                .set_increments(10.0, 1000.0);
            self.format_width_unitentry
                .get()
                .value_spinner()
                .set_digits(1);

            self.format_height_unitentry
                .get()
                .value_adj()
                .set_lower(1.0);
            self.format_height_unitentry
                .get()
                .value_spinner()
                .set_increments(10.0, 1000.0);
            self.format_height_unitentry
                .get()
                .value_spinner()
                .set_digits(1);

            self.background_pattern_width_unitentry
                .get()
                .value_adj()
                .set_lower(1.0);
            self.background_pattern_width_unitentry
                .get()
                .value_spinner()
                .set_increments(1.0, 10.0);
            self.background_pattern_width_unitentry
                .get()
                .value_spinner()
                .set_digits(1);

            self.background_pattern_height_unitentry
                .get()
                .value_adj()
                .set_lower(1.0);
            self.background_pattern_height_unitentry
                .get()
                .value_spinner()
                .set_increments(1.0, 10.0);
            self.background_pattern_height_unitentry
                .get()
                .value_spinner()
                .set_digits(1);

            self.temporary_format.connect_notify_local(
                Some("dpi"),
                clone!(@weak obj as settings_panel => move |format, _pspec| {
                    settings_panel.format_width_unitentry().set_dpi(format.dpi());
                    settings_panel.format_height_unitentry().set_dpi(format.dpi());
                }),
            );

            self.format_width_unitentry
                .get()
                .connect_local(
                    "measurement-changed",
                    false,
                    clone!(@weak obj as settings_panel => @default-return None, move |_args| {
                            settings_panel.update_temporary_format_from_rows();
                            None
                    }),
                )
                .unwrap();

            self.format_height_unitentry
                .get()
                .connect_local(
                    "measurement-changed",
                    false,
                    clone!(@weak obj as settings_panel => @default-return None, move |_args| {
                            settings_panel.update_temporary_format_from_rows();
                            None
                    }),
                )
                .unwrap();

            self.format_dpi_adj.connect_value_changed(
                clone!(@weak obj as settings_panel => move |_format_dpi_adj| {
                    settings_panel.update_temporary_format_from_rows();
                }),
            );
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            &[]
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            &[]
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            _value: &glib::Value,
            _pspec: &glib::ParamSpec,
        ) {
            unimplemented!()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            unimplemented!()
        }
    }

    impl WidgetImpl for SettingsPanel {}

    impl SettingsPanel {
        fn apply_predefined_format(obj: &super::SettingsPanel) {
            let priv_ = Self::from_instance(obj);

            if let Some(selected_item) = priv_.format_predefined_formats_row.selected_item() {
                // Dimensions are in mm
                let mut preconfigured_dimensions = match selected_item
                    .downcast::<adw::EnumListItem>()
                    .unwrap()
                    .nick()
                    .unwrap()
                    .as_str()
                {
                    "a2" => {
                        priv_.format_orientation_row.set_sensitive(true);
                        priv_.format_width_row.set_sensitive(false);
                        priv_.format_height_row.set_sensitive(false);
                        Some((420.0, 594.0))
                    }
                    "a3" => {
                        priv_.format_orientation_row.set_sensitive(true);
                        priv_.format_width_row.set_sensitive(false);
                        priv_.format_height_row.set_sensitive(false);
                        Some((297.0, 420.0))
                    }
                    "a4" => {
                        priv_.format_orientation_row.set_sensitive(true);
                        priv_.format_width_row.set_sensitive(false);
                        priv_.format_height_row.set_sensitive(false);
                        Some((210.0, 297.0))
                    }
                    "a5" => {
                        priv_.format_orientation_row.set_sensitive(true);
                        priv_.format_width_row.set_sensitive(false);
                        priv_.format_height_row.set_sensitive(false);
                        Some((148.0, 210.0))
                    }
                    "a6" => {
                        priv_.format_orientation_row.set_sensitive(true);
                        priv_.format_width_row.set_sensitive(false);
                        priv_.format_height_row.set_sensitive(false);
                        Some((105.0, 148.0))
                    }
                    "us-letter" => {
                        priv_.format_orientation_row.set_sensitive(true);
                        priv_.format_width_row.set_sensitive(false);
                        priv_.format_height_row.set_sensitive(false);
                        Some((215.9, 279.4))
                    }
                    "us-legal" => {
                        priv_.format_orientation_row.set_sensitive(true);
                        priv_.format_width_row.set_sensitive(false);
                        priv_.format_height_row.set_sensitive(false);
                        Some((215.9, 355.6))
                    }
                    "custom" => {
                        priv_.format_orientation_row.set_sensitive(false);
                        priv_.format_width_row.set_sensitive(true);
                        priv_.format_height_row.set_sensitive(true);
                        priv_.format_orientation_portrait_toggle.set_active(true);
                        priv_
                            .temporary_format
                            .set_orientation(format::Orientation::Portrait);
                        None
                    }
                    _ => {
                        log::error!(
                            "invalid nick string when selecting a format in format_predefined_formats_row"
                        );
                        None
                    }
                };

                if let Some(ref mut preconfigured_dimensions) = preconfigured_dimensions {
                    if priv_.temporary_format.orientation() == format::Orientation::Landscape {
                        let tmp = preconfigured_dimensions.0;
                        preconfigured_dimensions.0 = preconfigured_dimensions.1;
                        preconfigured_dimensions.1 = tmp;
                    }

                    let converted_width_mm = format::MeasureUnit::convert_measurement(
                        preconfigured_dimensions.0,
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                    );
                    let converted_height_mm = format::MeasureUnit::convert_measurement(
                        preconfigured_dimensions.1,
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                    );

                    // Setting the unit dropdowns to Mm
                    priv_
                        .format_width_unitentry
                        .get()
                        .set_unit(format::MeasureUnit::Mm);
                    priv_
                        .format_height_unitentry
                        .get()
                        .set_unit(format::MeasureUnit::Mm);

                    // setting the values
                    priv_
                        .format_width_unitentry
                        .get()
                        .set_value(converted_width_mm);
                    priv_
                        .format_height_unitentry
                        .get()
                        .set_value(converted_height_mm);
                }
            }
        }
    }
}

use adw::prelude::*;
use gtk4::{glib, glib::clone, subclass::prelude::*, Widget};
use gtk4::{Adjustment, ColorButton, ToggleButton};

use super::appwindow::RnoteAppWindow;
use super::canvas::Canvas;
use crate::sheet::background::PatternStyle;
use crate::sheet::format::{self, Format};
use crate::sheet::Sheet;
use crate::ui::unitentry::UnitEntry;
use crate::utils;

glib::wrapper! {
    pub struct SettingsPanel(ObjectSubclass<imp::SettingsPanel>)
    @extends Widget;
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsPanel {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SettingsPanel")
    }

    pub fn temporary_format(&self) -> Format {
        imp::SettingsPanel::from_instance(self)
            .temporary_format
            .clone()
    }

    pub fn set_predefined_format_variant(&self, predefined_format: format::PredefinedFormat) {
        let priv_ = imp::SettingsPanel::from_instance(self);
        let predefined_format_listmodel = priv_
            .format_predefined_formats_row
            .get()
            .model()
            .unwrap()
            .downcast::<adw::EnumListModel>()
            .unwrap();
        priv_
            .format_predefined_formats_row
            .get()
            .set_selected(predefined_format_listmodel.find_position(predefined_format as i32));
    }

    pub fn set_background_pattern_variant(&self, pattern: PatternStyle) {
        let priv_ = imp::SettingsPanel::from_instance(self);
        let background_pattern_listmodel = priv_
            .background_patterns_row
            .get()
            .model()
            .unwrap()
            .downcast::<adw::EnumListModel>()
            .unwrap();
        priv_
            .background_patterns_row
            .get()
            .set_selected(background_pattern_listmodel.find_position(pattern as i32));
    }

    pub fn set_format_orientation(&self, orientation: format::Orientation) {
        let priv_ = imp::SettingsPanel::from_instance(self);
        if orientation == format::Orientation::Portrait {
            priv_.format_orientation_portrait_toggle.set_active(true);
        } else {
            priv_.format_orientation_landscape_toggle.set_active(true);
        }
    }

    pub fn general_sheet_margin_unitentry(&self) -> UnitEntry {
        imp::SettingsPanel::from_instance(self)
            .general_sheet_margin_unitentry
            .clone()
    }

    pub fn general_pdf_import_width_adj(&self) -> Adjustment {
        imp::SettingsPanel::from_instance(self)
            .general_pdf_import_width_adj
            .clone()
    }

    pub fn general_pdf_import_as_vector_toggle(&self) -> ToggleButton {
        imp::SettingsPanel::from_instance(self)
            .general_pdf_import_as_vector_toggle
            .clone()
    }

    pub fn general_pdf_import_as_bitmap_toggle(&self) -> ToggleButton {
        imp::SettingsPanel::from_instance(self)
            .general_pdf_import_as_bitmap_toggle
            .clone()
    }

    pub fn format_width_unitentry(&self) -> UnitEntry {
        imp::SettingsPanel::from_instance(self)
            .format_width_unitentry
            .clone()
    }

    pub fn format_height_unitentry(&self) -> UnitEntry {
        imp::SettingsPanel::from_instance(self)
            .format_height_unitentry
            .clone()
    }

    pub fn format_dpi_adj(&self) -> Adjustment {
        imp::SettingsPanel::from_instance(self)
            .format_dpi_adj
            .clone()
    }

    pub fn background_color_choosebutton(&self) -> ColorButton {
        imp::SettingsPanel::from_instance(self)
            .background_color_choosebutton
            .clone()
    }

    pub fn background_patterns_row(&self) -> adw::ComboRow {
        imp::SettingsPanel::from_instance(self)
            .background_patterns_row
            .clone()
    }

    pub fn background_pattern_color_choosebutton(&self) -> ColorButton {
        imp::SettingsPanel::from_instance(self)
            .background_pattern_color_choosebutton
            .clone()
    }

    pub fn background_pattern_width_unitentry(&self) -> UnitEntry {
        imp::SettingsPanel::from_instance(self)
            .background_pattern_width_unitentry
            .clone()
    }

    pub fn background_pattern_height_unitentry(&self) -> UnitEntry {
        imp::SettingsPanel::from_instance(self)
            .background_pattern_height_unitentry
            .clone()
    }

    pub fn load_all(&self, appwindow: &RnoteAppWindow) {
        self.load_general(&appwindow.canvas());
        self.load_format(&appwindow.canvas().sheet());
        self.load_background(&appwindow.canvas().sheet());
    }

    pub fn load_general(&self, canvas: &Canvas) {
        self.general_sheet_margin_unitentry()
            .set_dpi(canvas.sheet().format().dpi());
        self.general_sheet_margin_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.general_sheet_margin_unitentry()
            .set_value(canvas.sheet_margin());

        self.general_pdf_import_width_adj()
            .set_value(canvas.pdf_import_width());
    }

    pub fn load_format(&self, sheet: &Sheet) {
        self.set_predefined_format_variant(format::PredefinedFormat::Custom);
        self.set_format_orientation(sheet.format().orientation());
        self.format_dpi_adj().set_value(sheet.format().dpi());

        self.format_width_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.format_width_unitentry()
            .set_value(f64::from(sheet.format().width()));

        self.format_height_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.format_height_unitentry()
            .set_value(f64::from(sheet.format().height()));
    }

    pub fn load_background(&self, sheet: &Sheet) {
        // Avoid already borrowed errors
        let background = sheet.background().borrow().clone();

        self.background_color_choosebutton()
            .set_rgba(&background.color().to_gdk());

        self.set_background_pattern_variant(background.pattern());

        self.background_pattern_color_choosebutton()
            .set_rgba(&background.pattern_color().to_gdk());

        // Background pattern Unit Entries
        self.background_pattern_width_unitentry()
            .set_dpi(sheet.format().dpi());
        self.background_pattern_width_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.background_pattern_width_unitentry()
            .set_value(background.pattern_size()[0]);

        self.background_pattern_height_unitentry()
            .set_dpi(sheet.format().dpi());
        self.background_pattern_height_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.background_pattern_height_unitentry()
            .set_value(background.pattern_size()[1]);
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SettingsPanel::from_instance(self);
        let temporary_format = priv_.temporary_format.clone();

        // General
        priv_.general_sheet_margin_unitentry.get().connect_local(
            "measurement-changed",
            false,
            clone!(@weak self as settings_panel, @weak appwindow => @default-return None, move |_args| {
                    let sheet_margin = f64::from(settings_panel.general_sheet_margin_unitentry().value_in_px());

                    appwindow.canvas().set_sheet_margin(sheet_margin);

                    appwindow.canvas().queue_allocate();
                    appwindow.canvas().queue_resize();
                    appwindow.canvas().queue_draw();

                    None
            }),
        )
        .unwrap();

        priv_
            .general_pdf_import_width_adj
            .get()
            .connect_value_changed(
                clone!(@weak appwindow => move |general_pdf_import_width_adj| {
                    let percentage = general_pdf_import_width_adj.value();

                    appwindow.canvas().set_pdf_import_width(percentage);
                }),
            );

        priv_
            .general_pdf_import_as_vector_toggle
            .connect_active_notify(
                clone!(@weak appwindow => move |general_pdf_import_as_vector_toggle| {
                    if general_pdf_import_as_vector_toggle.is_active() {
                        appwindow.canvas().set_pdf_import_as_vector(true);
                    }
                }),
            );

        priv_.general_pdf_import_as_vector_toggle.connect_toggled(clone!(@weak appwindow => move |general_pdf_import_as_vector_toggle| {
            appwindow.application().unwrap().change_action_state("pdf-import-as-vector", &general_pdf_import_as_vector_toggle.is_active().to_variant());
        }));

        // Format
        priv_.format_revert_button.get().connect_clicked(
            clone!(@weak self as settings_panel, @weak appwindow => move |_format_revert_button| {
                let priv_ = imp::SettingsPanel::from_instance(&settings_panel);
                priv_.temporary_format.import_format(appwindow.canvas().sheet().format());
                let revert_format = appwindow.canvas().sheet().format();

                settings_panel.set_predefined_format_variant(format::PredefinedFormat::Custom);

                priv_.format_dpi_adj.set_value(revert_format.dpi());

                // Setting the unit dropdowns to Px
                settings_panel.format_width_unitentry().set_unit(format::MeasureUnit::Px);
                settings_panel.format_height_unitentry().set_unit(format::MeasureUnit::Px);

                // Setting the entries, which have callbacks to update the temporary format
                settings_panel.format_width_unitentry()
                    .set_value(f64::from(revert_format.width()));
                settings_panel.format_height_unitentry()
                    .set_value(f64::from(revert_format.height()));
            }),
        );

        priv_.format_apply_button.get().connect_clicked(
            clone!(@weak temporary_format, @weak appwindow => move |_format_apply_button| {
                appwindow.canvas().sheet().format().import_format(temporary_format);
                appwindow.canvas().sheet().set_padding_bottom(appwindow.canvas().sheet().format().height());

                appwindow.canvas().sheet().resize_to_format();
                appwindow.canvas().regenerate_background(false);
                appwindow.canvas().regenerate_content(true, true);
            }),
        );

        // Background
        priv_.background_color_choosebutton.connect_color_set(clone!(@weak appwindow => move |background_color_choosebutton| {
            appwindow.canvas().sheet().background().borrow_mut().set_color(utils::Color::from(background_color_choosebutton.rgba()));
            appwindow.canvas().regenerate_background(true);
        }));

        priv_.background_patterns_row.get().connect_selected_item_notify(clone!(@weak self as settings_panel, @weak appwindow => move |background_patterns_row| {
            if let Some(selected_item) = background_patterns_row.selected_item() {
                match selected_item
                    .downcast::<adw::EnumListItem>()
                    .unwrap()
                    .nick()
                    .unwrap()
                    .as_str()
                {
                    "none" => {
                        appwindow.canvas().sheet().background().borrow_mut().set_pattern(PatternStyle::None);
                        settings_panel.background_pattern_width_unitentry().set_sensitive(false);
                        settings_panel.background_pattern_height_unitentry().set_sensitive(false);

                    },
                    "lines" => {
                        appwindow.canvas().sheet().background().borrow_mut().set_pattern(PatternStyle::Lines);
                        settings_panel.background_pattern_width_unitentry().set_sensitive(false);
                        settings_panel.background_pattern_height_unitentry().set_sensitive(true);
                    },
                    "grid" => {
                        appwindow.canvas().sheet().background().borrow_mut().set_pattern(PatternStyle::Grid);
                        settings_panel.background_pattern_width_unitentry().set_sensitive(true);
                        settings_panel.background_pattern_height_unitentry().set_sensitive(true);
                    },
                    "dots" => {
                        appwindow.canvas().sheet().background().borrow_mut().set_pattern(PatternStyle::Dots);
                        settings_panel.background_pattern_width_unitentry().set_sensitive(true);
                        settings_panel.background_pattern_height_unitentry().set_sensitive(true);
                    },
                    _ => {
                        log::error!(
                            "invalid nick string when selecting a pattern in background_patterns_row"
                        );
                    }
                };

                appwindow.canvas().regenerate_background(true);
            }
        }));

        priv_.background_pattern_color_choosebutton.connect_color_set(clone!(@weak appwindow => move |background_pattern_color_choosebutton| {
            appwindow.canvas().sheet().background().borrow_mut().set_pattern_color(utils::Color::from(background_pattern_color_choosebutton.rgba()));
            appwindow.canvas().regenerate_background(true);
        }));

        priv_.background_pattern_width_unitentry.get().connect_local(
            "measurement-changed",
            false,
            clone!(@weak self as settings_panel, @weak appwindow => @default-return None, move |_args| {
                    let mut pattern_size = appwindow.canvas().sheet().background().borrow().pattern_size();
                    pattern_size[0] = f64::from(settings_panel.background_pattern_width_unitentry().value_in_px());

                    appwindow.canvas().sheet().background().borrow_mut().set_pattern_size(pattern_size);

                    appwindow.canvas().regenerate_background(true);

                    None
            }),
        )
        .unwrap();

        priv_.background_pattern_height_unitentry.get().connect_local(
            "measurement-changed",
            false,
            clone!(@weak self as settings_panel, @weak appwindow => @default-return None, move |_args| {
                    let mut pattern_size = appwindow.canvas().sheet().background().borrow().pattern_size();
                    pattern_size[1] = f64::from(settings_panel.background_pattern_height_unitentry().value_in_px());
                    appwindow.canvas().sheet().background().borrow_mut().set_pattern_size(pattern_size);

                    appwindow.canvas().regenerate_background(true);

                    None
            }),
        )
        .unwrap();
    }

    pub fn update_temporary_format_from_rows(&self) {
        let priv_ = imp::SettingsPanel::from_instance(self);

        // Format
        if priv_.format_orientation_portrait_toggle.is_active() {
            priv_
                .temporary_format
                .set_orientation(format::Orientation::Portrait);
        } else {
            priv_
                .temporary_format
                .set_orientation(format::Orientation::Landscape);
        }

        // DPI (before width, height)
        priv_.temporary_format.set_dpi(
            priv_
                .format_dpi_adj
                .value()
                .clamp(Format::DPI_MIN, Format::DPI_MAX),
        );

        // Width
        priv_.temporary_format.set_width(
            priv_
                .format_width_unitentry
                .value_in_px()
                .clamp(Format::WIDTH_MIN, Format::WIDTH_MAX),
        );
        // Height
        priv_.temporary_format.set_height(
            priv_
                .format_height_unitentry
                .value_in_px()
                .clamp(Format::HEIGHT_MIN, Format::HEIGHT_MAX),
        );
    }
}
