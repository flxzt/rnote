pub mod penshortcutmodels;
pub mod penshortcutrow;

mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;

    use adw::prelude::*;
    use gtk4::{glib, glib::clone, subclass::prelude::*, CompositeTemplate};
    use gtk4::{Adjustment, Button, ColorButton, ScrolledWindow, ToggleButton};

    use crate::unitentry::UnitEntry;
    use rnote_engine::sheet::format::{self, Format};

    use super::penshortcutrow::PenShortcutRow;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/settingspanel.ui")]
    pub struct SettingsPanel {
        pub temporary_format: Rc<RefCell<Format>>,

        #[template_child]
        pub settings_scroller: TemplateChild<ScrolledWindow>,
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
        #[template_child]
        pub penshortcut_stylus_button_primary_row: TemplateChild<PenShortcutRow>,
        #[template_child]
        pub penshortcut_stylus_button_secondary_row: TemplateChild<PenShortcutRow>,
        #[template_child]
        pub penshortcut_stylus_button_eraser_row: TemplateChild<PenShortcutRow>,
        #[template_child]
        pub penshortcut_mouse_button_secondary_row: TemplateChild<PenShortcutRow>,
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

            self.format_predefined_formats_row
                .connect_selected_item_notify(
                    clone!(@weak obj => move |_format_predefined_formats_row| {
                        obj.imp().update_temporary_format_from_rows();
                        obj.imp().apply_predefined_format();
                    }),
                );

            self.format_orientation_portrait_toggle.connect_toggled(
                clone!(@weak obj => move |_format_orientation_portrait_toggle| {
                    obj.imp().update_temporary_format_from_rows();
                    obj.imp().apply_predefined_format();
                }),
            );

            self.format_orientation_landscape_toggle.connect_toggled(
                clone!(@weak obj => move |_format_orientation_landscape_toggle| {
                    obj.imp().update_temporary_format_from_rows();
                    obj.imp().apply_predefined_format();
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

            /*             self.temporary_format.connect_notify_local(
                Some("dpi"),
                clone!(@weak obj as settings_panel => move |format, _pspec| {
                    settings_panel.format_width_unitentry().set_dpi(format.dpi());
                    settings_panel.format_height_unitentry().set_dpi(format.dpi());
                }),
            ); */

            self.format_width_unitentry.get().connect_local(
                "measurement-changed",
                false,
                clone!(@weak obj as settings_panel => @default-return None, move |_args| {
                        settings_panel.imp().update_temporary_format_from_rows();
                        None
                }),
            );

            self.format_height_unitentry.get().connect_local(
                "measurement-changed",
                false,
                clone!(@weak obj as settings_panel => @default-return None, move |_args| {
                        settings_panel.imp().update_temporary_format_from_rows();
                        None
                }),
            );

            self.format_dpi_adj.connect_value_changed(
                clone!(@weak obj as settings_panel => move |format_dpi_adj| {
                    settings_panel.imp().update_temporary_format_from_rows();
                    settings_panel.format_width_unitentry().set_dpi(format_dpi_adj.value());
                    settings_panel.format_height_unitentry().set_dpi(format_dpi_adj.value());
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
        pub fn update_temporary_format_from_rows(&self) {
            // Format
            if self.format_orientation_portrait_toggle.is_active() {
                self.temporary_format.borrow_mut().orientation = format::Orientation::Portrait;
            } else {
                self.temporary_format.borrow_mut().orientation = format::Orientation::Landscape;
            }

            // DPI (before width, height)
            self.temporary_format.borrow_mut().dpi = self
                .format_dpi_adj
                .value()
                .clamp(Format::DPI_MIN, Format::DPI_MAX);

            // Width
            self.temporary_format.borrow_mut().width = self
                .format_width_unitentry
                .value_in_px()
                .clamp(Format::WIDTH_MIN, Format::WIDTH_MAX);
            // Height
            self.temporary_format.borrow_mut().height = self
                .format_height_unitentry
                .value_in_px()
                .clamp(Format::HEIGHT_MIN, Format::HEIGHT_MAX);
        }
        fn apply_predefined_format(&self) {
            if let Some(selected_item) = self.format_predefined_formats_row.selected_item() {
                // Dimensions are in mm
                let mut preconfigured_dimensions = match selected_item
                    .downcast::<adw::EnumListItem>()
                    .unwrap()
                    .nick()
                    .as_str()
                {
                    "a2" => {
                        self.format_orientation_row.set_sensitive(true);
                        self.format_width_row.set_sensitive(false);
                        self.format_height_row.set_sensitive(false);
                        Some((420.0, 594.0))
                    }
                    "a3" => {
                        self.format_orientation_row.set_sensitive(true);
                        self.format_width_row.set_sensitive(false);
                        self.format_height_row.set_sensitive(false);
                        Some((297.0, 420.0))
                    }
                    "a4" => {
                        self.format_orientation_row.set_sensitive(true);
                        self.format_width_row.set_sensitive(false);
                        self.format_height_row.set_sensitive(false);
                        Some((210.0, 297.0))
                    }
                    "a5" => {
                        self.format_orientation_row.set_sensitive(true);
                        self.format_width_row.set_sensitive(false);
                        self.format_height_row.set_sensitive(false);
                        Some((148.0, 210.0))
                    }
                    "a6" => {
                        self.format_orientation_row.set_sensitive(true);
                        self.format_width_row.set_sensitive(false);
                        self.format_height_row.set_sensitive(false);
                        Some((105.0, 148.0))
                    }
                    "us-letter" => {
                        self.format_orientation_row.set_sensitive(true);
                        self.format_width_row.set_sensitive(false);
                        self.format_height_row.set_sensitive(false);
                        Some((215.9, 279.4))
                    }
                    "us-legal" => {
                        self.format_orientation_row.set_sensitive(true);
                        self.format_width_row.set_sensitive(false);
                        self.format_height_row.set_sensitive(false);
                        Some((215.9, 355.6))
                    }
                    "custom" => {
                        self.format_orientation_row.set_sensitive(false);
                        self.format_width_row.set_sensitive(true);
                        self.format_height_row.set_sensitive(true);
                        self.format_orientation_portrait_toggle.set_active(true);
                        self.temporary_format.borrow_mut().orientation =
                            format::Orientation::Portrait;
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
                    if self.temporary_format.borrow().orientation == format::Orientation::Landscape
                    {
                        std::mem::swap(
                            &mut preconfigured_dimensions.0,
                            &mut preconfigured_dimensions.1,
                        );
                    }

                    let converted_width_mm = format::MeasureUnit::convert_measurement(
                        preconfigured_dimensions.0,
                        format::MeasureUnit::Mm,
                        self.temporary_format.borrow().dpi,
                        format::MeasureUnit::Mm,
                        self.temporary_format.borrow().dpi,
                    );
                    let converted_height_mm = format::MeasureUnit::convert_measurement(
                        preconfigured_dimensions.1,
                        format::MeasureUnit::Mm,
                        self.temporary_format.borrow().dpi,
                        format::MeasureUnit::Mm,
                        self.temporary_format.borrow().dpi,
                    );

                    // Setting the unit dropdowns to Mm
                    self.format_width_unitentry
                        .get()
                        .set_unit(format::MeasureUnit::Mm);
                    self.format_height_unitentry
                        .get()
                        .set_unit(format::MeasureUnit::Mm);

                    // setting the values
                    self.format_width_unitentry
                        .get()
                        .set_value(converted_width_mm);
                    self.format_height_unitentry
                        .get()
                        .set_value(converted_height_mm);
                }
            }
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk4::{glib, glib::clone, subclass::prelude::*, Widget};
use gtk4::{Adjustment, ColorButton, ScrolledWindow, ToggleButton};
use rnote_engine::pens::shortcuts::ShortcutKey;

use super::appwindow::RnoteAppWindow;
use crate::unitentry::UnitEntry;
use rnote_engine::compose::color::Color;
use rnote_engine::sheet::background::PatternStyle;
use rnote_engine::sheet::format::{self, Format};

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

    pub fn temporary_format(&self) -> Rc<RefCell<Format>> {
        Rc::clone(&self.imp().temporary_format)
    }

    pub fn set_predefined_format_variant(&self, predefined_format: format::PredefinedFormat) {
        let predefined_format_listmodel = self
            .imp()
            .format_predefined_formats_row
            .get()
            .model()
            .unwrap()
            .downcast::<adw::EnumListModel>()
            .unwrap();
        self.imp()
            .format_predefined_formats_row
            .get()
            .set_selected(predefined_format_listmodel.find_position(predefined_format as i32));
    }

    pub fn set_background_pattern_variant(&self, pattern: PatternStyle) {
        let background_pattern_listmodel = self
            .imp()
            .background_patterns_row
            .get()
            .model()
            .unwrap()
            .downcast::<adw::EnumListModel>()
            .unwrap();
        self.imp()
            .background_patterns_row
            .get()
            .set_selected(background_pattern_listmodel.find_position(pattern as i32));
    }

    pub fn set_format_orientation(&self, orientation: format::Orientation) {
        if orientation == format::Orientation::Portrait {
            self.imp()
                .format_orientation_portrait_toggle
                .set_active(true);
        } else {
            self.imp()
                .format_orientation_landscape_toggle
                .set_active(true);
        }
    }

    pub fn settings_scroller(&self) -> ScrolledWindow {
        imp::SettingsPanel::from_instance(self)
            .settings_scroller
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

    pub fn refresh_for_sheet(&self, appwindow: &RnoteAppWindow) {
        self.load_misc(appwindow);
        self.load_format(appwindow);
        self.load_background(appwindow);
        self.load_shortcuts(appwindow);
    }

    pub fn load_misc(&self, appwindow: &RnoteAppWindow) {
        self.general_pdf_import_width_adj()
            .set_value(appwindow.canvas().pdf_import_width());
    }

    pub fn load_format(&self, appwindow: &RnoteAppWindow) {
        let format = appwindow.canvas().sheet().borrow().format.clone();

        self.set_predefined_format_variant(format::PredefinedFormat::Custom);
        self.set_format_orientation(format.orientation);
        self.format_dpi_adj().set_value(format.dpi);

        self.format_width_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.format_width_unitentry()
            .set_value(f64::from(format.width));

        self.format_height_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.format_height_unitentry()
            .set_value(f64::from(format.height));
    }

    pub fn load_background(&self, appwindow: &RnoteAppWindow) {
        let background = appwindow.canvas().sheet().borrow().background.clone();
        let format = appwindow.canvas().sheet().borrow().format.clone();

        self.background_color_choosebutton()
            .set_rgba(&background.color.to_gdk());

        self.set_background_pattern_variant(background.pattern);
        self.background_pattern_color_choosebutton()
            .set_rgba(&background.pattern_color.to_gdk());

        // Background pattern Unit Entries
        self.background_pattern_width_unitentry()
            .set_dpi(format.dpi);
        self.background_pattern_width_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.background_pattern_width_unitentry()
            .set_value(background.pattern_size[0]);

        self.background_pattern_height_unitentry()
            .set_dpi(format.dpi);
        self.background_pattern_height_unitentry()
            .set_unit(format::MeasureUnit::Px);
        self.background_pattern_height_unitentry()
            .set_value(background.pattern_size[1]);
    }

    pub fn load_shortcuts(&self, appwindow: &RnoteAppWindow) {
        let current_shortcuts = appwindow.canvas().pens().borrow().list_current_shortcuts();

        current_shortcuts
            .into_iter()
            .for_each(|(key, action)| match key.clone() {
                ShortcutKey::StylusPrimaryButton => {
                    self.imp()
                        .penshortcut_stylus_button_primary_row
                        .set_action(action.clone());
                }
                ShortcutKey::StylusSecondaryButton => {
                    self.imp()
                        .penshortcut_stylus_button_secondary_row
                        .set_action(action.clone());
                }
                ShortcutKey::StylusEraserButton => {
                    self.imp()
                        .penshortcut_stylus_button_eraser_row
                        .set_action(action.clone());
                }
                ShortcutKey::MouseSecondaryButton => {
                    self.imp()
                        .penshortcut_mouse_button_secondary_row
                        .set_action(action.clone());
                }
                _ => {}
            });
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let temporary_format = self.imp().temporary_format.clone();
        let penshortcut_stylus_button_primary_row =
            self.imp().penshortcut_stylus_button_primary_row.get();
        let penshortcut_stylus_button_secondary_row =
            self.imp().penshortcut_stylus_button_secondary_row.get();
        let penshortcut_stylus_button_eraser_row =
            self.imp().penshortcut_stylus_button_eraser_row.get();
        let penshortcut_mouse_button_secondary_row =
            self.imp().penshortcut_mouse_button_secondary_row.get();

        // Pdf import width
        self.imp()
            .general_pdf_import_width_adj
            .get()
            .bind_property("value", &appwindow.canvas(), "pdf-import-width")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        // Pdf import as vector or bitmap
        self.imp()
            .general_pdf_import_as_vector_toggle
            .get()
            .bind_property("active", &appwindow.canvas(), "pdf-import-as-vector")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
        self.imp()
            .general_pdf_import_as_bitmap_toggle
            .get()
            .bind_property("active", &appwindow.canvas(), "pdf-import-as-vector")
            .flags(
                glib::BindingFlags::SYNC_CREATE
                    | glib::BindingFlags::BIDIRECTIONAL
                    | glib::BindingFlags::INVERT_BOOLEAN,
            )
            .build();

        // revert format
        self.imp().format_revert_button.get().connect_clicked(
            clone!(@weak self as settings_panel, @weak appwindow => move |_format_revert_button| {
                *settings_panel.imp().temporary_format.borrow_mut() = appwindow.canvas().sheet().borrow().format.clone();
                let revert_format = appwindow.canvas().sheet().borrow().format.clone();

                settings_panel.set_predefined_format_variant(format::PredefinedFormat::Custom);

                settings_panel.imp().format_dpi_adj.set_value(revert_format.dpi);

                // Setting the unit dropdowns to Px
                settings_panel.format_width_unitentry().set_unit(format::MeasureUnit::Px);
                settings_panel.format_height_unitentry().set_unit(format::MeasureUnit::Px);

                // Setting the entries, which have callbacks to update the temporary format
                settings_panel.format_width_unitentry()
                    .set_value(f64::from(revert_format.width));
                settings_panel.format_height_unitentry()
                    .set_value(f64::from(revert_format.height));
            }));

        // Apply format
        self.imp().format_apply_button.get().connect_clicked(
            clone!(@weak temporary_format, @weak appwindow => move |_format_apply_button| {
                let temporary_format = temporary_format.borrow().clone();
                appwindow.canvas().sheet().borrow_mut().format = temporary_format;

                appwindow.canvas().resize_sheet_to_fit_strokes();
                appwindow.canvas().regenerate_background(false);
                appwindow.canvas().regenerate_content(true, true);
            }),
        );

        // Background
        self.imp().background_color_choosebutton.connect_color_set(clone!(@weak appwindow => move |background_color_choosebutton| {
            appwindow.canvas().sheet().borrow_mut().background.color = Color::from(background_color_choosebutton.rgba());
            appwindow.canvas().regenerate_background(true);
        }));

        self.imp().background_patterns_row.get().connect_selected_item_notify(clone!(@weak self as settings_panel, @weak appwindow => move |background_patterns_row| {
            if let Some(selected_item) = background_patterns_row.selected_item() {
                match selected_item
                    .downcast::<adw::EnumListItem>()
                    .unwrap()
                    .nick()
                    .as_str()
                {
                    "none" => {
                        appwindow.canvas().sheet().borrow_mut().background.pattern = PatternStyle::None;
                        settings_panel.background_pattern_width_unitentry().set_sensitive(false);
                        settings_panel.background_pattern_height_unitentry().set_sensitive(false);

                    },
                    "lines" => {
                        appwindow.canvas().sheet().borrow_mut().background.pattern = PatternStyle::Lines;
                        settings_panel.background_pattern_width_unitentry().set_sensitive(false);
                        settings_panel.background_pattern_height_unitentry().set_sensitive(true);
                    },
                    "grid" => {
                        appwindow.canvas().sheet().borrow_mut().background.pattern = PatternStyle::Grid;
                        settings_panel.background_pattern_width_unitentry().set_sensitive(true);
                        settings_panel.background_pattern_height_unitentry().set_sensitive(true);
                    },
                    "dots" => {
                        appwindow.canvas().sheet().borrow_mut().background.pattern = PatternStyle::Dots;
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

        self.imp().background_pattern_color_choosebutton.connect_color_set(clone!(@weak appwindow => move |background_pattern_color_choosebutton| {
            appwindow.canvas().sheet().borrow_mut().background.pattern_color = Color::from(background_pattern_color_choosebutton.rgba());
            appwindow.canvas().regenerate_background(true);
        }));

        self.imp().background_pattern_width_unitentry.get().connect_local(
            "measurement-changed",
            false,
            clone!(@weak self as settings_panel, @weak appwindow => @default-return None, move |_args| {
                    let mut pattern_size = appwindow.canvas().sheet().borrow().background.pattern_size;
                    pattern_size[0] = f64::from(settings_panel.background_pattern_width_unitentry().value_in_px());

                    appwindow.canvas().sheet().borrow_mut().background.pattern_size = pattern_size;
                    appwindow.canvas().regenerate_background(true);

                    None
            }),
        );

        self.imp().background_pattern_height_unitentry.get().connect_local(
            "measurement-changed",
            false,
            clone!(@weak self as settings_panel, @weak appwindow => @default-return None, move |_args| {
                    let mut pattern_size = appwindow.canvas().sheet().borrow().background.pattern_size;
                    pattern_size[1] = f64::from(settings_panel.background_pattern_height_unitentry().value_in_px());

                    appwindow.canvas().sheet().borrow_mut().background.pattern_size = pattern_size;
                    appwindow.canvas().regenerate_background(true);

                    None
            }),
        );

        // Shortcuts
        self.imp()
            .penshortcut_stylus_button_primary_row
            .set_key(Some(ShortcutKey::StylusPrimaryButton));
        self.imp().penshortcut_stylus_button_primary_row.connect_local("action-changed", false, clone!(@weak penshortcut_stylus_button_primary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_stylus_button_primary_row.action();
            appwindow.canvas().pens().borrow_mut().register_new_shortcut(ShortcutKey::StylusPrimaryButton, action);
            None
        }));

        self.imp()
            .penshortcut_stylus_button_secondary_row
            .set_key(Some(ShortcutKey::StylusSecondaryButton));
        self.imp().penshortcut_stylus_button_secondary_row.connect_local("action-changed", false, clone!(@weak penshortcut_stylus_button_secondary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_stylus_button_secondary_row.action();
            appwindow.canvas().pens().borrow_mut().register_new_shortcut(ShortcutKey::StylusSecondaryButton, action);
            None
        }));

        self.imp()
            .penshortcut_stylus_button_eraser_row
            .set_key(Some(ShortcutKey::StylusEraserButton));
        self.imp().penshortcut_stylus_button_eraser_row.connect_local("action-changed", false, clone!(@weak penshortcut_stylus_button_eraser_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_stylus_button_eraser_row.action();
            appwindow.canvas().pens().borrow_mut().register_new_shortcut(ShortcutKey::StylusEraserButton, action);
            None
        }));

        self.imp()
            .penshortcut_mouse_button_secondary_row
            .set_key(Some(ShortcutKey::StylusSecondaryButton));
        self.imp().penshortcut_mouse_button_secondary_row.connect_local("action-changed", false, clone!(@weak penshortcut_mouse_button_secondary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_mouse_button_secondary_row.action();
            appwindow.canvas().pens().borrow_mut().register_new_shortcut(ShortcutKey::MouseSecondaryButton, action);
            None
        }));
    }
}
