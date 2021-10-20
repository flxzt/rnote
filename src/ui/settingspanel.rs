mod imp {
    use adw::prelude::*;
    use gtk4::{glib, glib::clone, subclass::prelude::*, CompositeTemplate};
    use gtk4::{Adjustment, Button, ColorButton, DropDown, Entry, ToggleButton};

    use crate::sheet::format::{self, Format};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/settingspanel.ui")]
    pub struct SettingsPanel {
        pub temporary_format: Format,
        #[template_child]
        pub predefined_formats_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub format_orientation_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub format_orientation_portrait_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub format_orientation_landscape_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub format_width_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub format_width_entry: TemplateChild<Entry>,
        #[template_child]
        pub format_width_unitdropdown: TemplateChild<DropDown>,
        #[template_child]
        pub format_height_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub format_height_entry: TemplateChild<Entry>,
        #[template_child]
        pub format_height_unitdropdown: TemplateChild<DropDown>,
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

            self.predefined_formats_row.connect_selected_item_notify(
                clone!(@weak obj => move |_predefined_formats_row| {
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

            self.format_width_unitdropdown.connect_selected_item_notify(
                clone!(@weak obj => move |_format_width_unitdropdown| {
                    obj.update_temporary_format_from_rows();
                }),
            );

            self.format_height_unitdropdown
                .connect_selected_item_notify(
                    clone!(@weak obj => move |_format_height_unitdropdown| {
                        obj.update_temporary_format_from_rows();
                    }),
                );

            self.format_width_entry.buffer().connect_text_notify(
                clone!(@weak obj => move |_buffer| {
                    obj.update_temporary_format_from_rows();
                }),
            );

            self.format_height_entry.buffer().connect_text_notify(
                clone!(@weak obj => move |_buffer| {
                obj.update_temporary_format_from_rows();
                }),
            );

            self.format_dpi_adj
                .connect_value_changed(clone!(@weak obj => move |_format_dpi_adj| {
                    obj.update_temporary_format_from_rows();
                }));
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

            if let Some(selected_item) = priv_.predefined_formats_row.selected_item() {
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
                            "invalid nick string when selecting a format in predefined_formats_row"
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

                    // Setting the unit dropdowns to Mm
                    let width_unit_listmodel = priv_
                        .format_width_unitdropdown
                        .get()
                        .model()
                        .unwrap()
                        .downcast::<adw::EnumListModel>()
                        .unwrap();
                    priv_.format_width_unitdropdown.get().set_selected(
                        width_unit_listmodel.find_position(format::MeasureUnit::Mm as i32),
                    );

                    let height_unit_listmodel = priv_
                        .format_height_unitdropdown
                        .get()
                        .model()
                        .unwrap()
                        .downcast::<adw::EnumListModel>()
                        .unwrap();
                    priv_.format_height_unitdropdown.get().set_selected(
                        height_unit_listmodel.find_position(format::MeasureUnit::Mm as i32),
                    );

                    let converted_width_px = format::MeasureUnit::convert_measure_units(
                        preconfigured_dimensions.0,
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                    );
                    let converted_height_px = format::MeasureUnit::convert_measure_units(
                        preconfigured_dimensions.1,
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                        format::MeasureUnit::Mm,
                        priv_.temporary_format.dpi(),
                    );

                    // Setting the entries, which have callbacks to update the temporary format
                    priv_
                        .format_width_entry
                        .buffer()
                        .set_text(&converted_width_px.to_string());
                    priv_
                        .format_height_entry
                        .buffer()
                        .set_text(&converted_height_px.to_string());
                }
            }
        }
    }
}

use adw::prelude::*;
use gtk4::{glib, glib::clone, subclass::prelude::*, Widget};
use gtk4::{Adjustment, ColorButton, Entry};

use super::appwindow::RnoteAppWindow;
use crate::sheet::background::{Background, PatternStyle};
use crate::sheet::format::{self, Format};
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
            .predefined_formats_row
            .get()
            .model()
            .unwrap()
            .downcast::<adw::EnumListModel>()
            .unwrap();
        priv_
            .predefined_formats_row
            .get()
            .set_selected(predefined_format_listmodel.find_position(predefined_format as i32));
    }

    pub fn set_format_orientation(&self, orientation: format::Orientation) {
        let priv_ = imp::SettingsPanel::from_instance(self);
        if orientation == format::Orientation::Portrait {
            priv_.format_orientation_portrait_toggle.set_active(true);
        } else {
            priv_.format_orientation_landscape_toggle.set_active(true);
        }
    }

    pub fn format_width_entry(&self) -> Entry {
        imp::SettingsPanel::from_instance(self)
            .format_width_entry
            .clone()
    }

    pub fn format_height_entry(&self) -> Entry {
        imp::SettingsPanel::from_instance(self)
            .format_height_entry
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

    pub fn load_format(&self, format: Format) {
        self.set_predefined_format_variant(format::PredefinedFormat::Custom);
        self.format_width_entry()
            .buffer()
            .set_text(format.width().to_string().as_str());
        self.format_height_entry()
            .buffer()
            .set_text(format.height().to_string().as_str());
        self.format_dpi_adj().set_value(f64::from(format.dpi()));
        self.set_format_orientation(format.orientation());
    }

    pub fn load_background(&self, background: Background) {
        self.background_color_choosebutton()
            .set_rgba(&background.color().to_gdk());

        self.background_patterns_row()
            .set_selected(background.pattern() as u32);
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::SettingsPanel::from_instance(self);
        let temporary_format = priv_.temporary_format.clone();

        // Format
        priv_.format_revert_button.get().connect_clicked(clone!(@weak self as settingspanel, @weak appwindow => move |_format_revert_button| {
            let priv_ = imp::SettingsPanel::from_instance(&settingspanel);
            priv_.temporary_format.replace_fields(appwindow.canvas().sheet().format());
            let revert_format = appwindow.canvas().sheet().format();

            // Setting the unit dropdowns to Px
            let width_unit_listmodel = priv_.format_width_unitdropdown.get().model().unwrap().downcast::<adw::EnumListModel>().unwrap();
            priv_.format_width_unitdropdown.get().set_selected(width_unit_listmodel.find_position(format::MeasureUnit::Px as i32));

            let height_unit_listmodel = priv_.format_height_unitdropdown.get().model().unwrap().downcast::<adw::EnumListModel>().unwrap();
            priv_.format_height_unitdropdown.get().set_selected(height_unit_listmodel.find_position(format::MeasureUnit::Px as i32));

            // Setting the entries, which have callbacks to update the temporary format
            priv_.format_width_entry.buffer().set_text(&revert_format.width().to_string());
            priv_.format_height_entry.buffer().set_text(&revert_format.height().to_string());
            priv_.format_dpi_adj.set_value(f64::from(revert_format.dpi()));

            settingspanel.set_predefined_format_variant(format::PredefinedFormat::Custom);
        }));

        priv_.format_apply_button.get().connect_clicked(
            clone!(@weak temporary_format, @weak appwindow => move |_format_apply_button| {
                appwindow.canvas().sheet().format().replace_fields(temporary_format);

                appwindow.canvas().sheet().resize_to_format();
                appwindow.canvas().regenerate_content(true);
            }),
        );

        // Background
        priv_.background_color_choosebutton.connect_color_set(clone!(@weak appwindow => move |background_color_choosebutton| {
            appwindow.canvas().sheet().background().borrow_mut().set_color(utils::Color::from(background_color_choosebutton.rgba()));
            appwindow.canvas().regenerate_background(true);
            appwindow.canvas().queue_resize();
        }));

        priv_.background_patterns_row.get().connect_selected_item_notify(clone!(@weak appwindow => move |background_patterns_row| {
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

                    },
                    "lines" => {
                        appwindow.canvas().sheet().background().borrow_mut().set_pattern(PatternStyle::Lines);
                    },
                    "grid" => {
                        appwindow.canvas().sheet().background().borrow_mut().set_pattern(PatternStyle::Grid);
                    },
                    _ => {
                        log::error!(
                            "invalid nick string when selecting a format in predefined_formats_row"
                        );
                    }
                };

                appwindow.canvas().regenerate_background(true);
            }
        }));
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

        // DPI
        {
            priv_.temporary_format.set_dpi(
                (priv_.format_dpi_adj.value().round() as i32)
                    .clamp(Format::DPI_MIN, Format::DPI_MAX),
            );
        }

        {
            // Width
            if let Some(selected_item_width) = priv_.format_width_unitdropdown.selected_item() {
                let measure_unit_width = match selected_item_width
                    .downcast::<adw::EnumListItem>()
                    .unwrap()
                    .nick()
                    .unwrap()
                    .as_str()
                {
                    "px" => Some(format::MeasureUnit::Px),
                    "mm" => Some(format::MeasureUnit::Mm),
                    "cm" => Some(format::MeasureUnit::Cm),
                    _ => None,
                };

                if let Some(measure_unit_width) = measure_unit_width {
                    if let Some(width) = priv_
                        .format_width_entry
                        .buffer()
                        .text()
                        .as_str()
                        .parse::<f64>()
                        .ok()
                    {
                        let converted_width_px = format::MeasureUnit::convert_measure_units(
                            f64::from(width),
                            measure_unit_width,
                            priv_.temporary_format.dpi(),
                            format::MeasureUnit::Px,
                            priv_.temporary_format.dpi(),
                        )
                        .round() as i32;
                        priv_.temporary_format.set_width(
                            converted_width_px.clamp(Format::WIDTH_MIN, Format::WIDTH_MAX),
                        );

                        priv_
                            .format_width_entry
                            .style_context()
                            .remove_class("error");
                        priv_.format_width_entry.style_context().add_class("plain");
                    } else {
                        priv_
                            .format_width_entry
                            .style_context()
                            .remove_class("plain");
                        priv_.format_width_entry.style_context().add_class("error");
                    }
                }
            }
        }

        // Height
        {
            if let Some(selected_item_height) = priv_.format_height_unitdropdown.selected_item() {
                let measure_unit_height = match selected_item_height
                    .downcast::<adw::EnumListItem>()
                    .unwrap()
                    .nick()
                    .unwrap()
                    .as_str()
                {
                    "px" => Some(format::MeasureUnit::Px),
                    "mm" => Some(format::MeasureUnit::Mm),
                    "cm" => Some(format::MeasureUnit::Cm),
                    _ => None,
                };

                if let Some(measure_unit_height) = measure_unit_height {
                    if let Some(height) = priv_
                        .format_height_entry
                        .buffer()
                        .text()
                        .as_str()
                        .parse::<f64>()
                        .ok()
                    {
                        let converted_height_px = format::MeasureUnit::convert_measure_units(
                            f64::from(height),
                            measure_unit_height,
                            priv_.temporary_format.dpi(),
                            format::MeasureUnit::Px,
                            priv_.temporary_format.dpi(),
                        )
                        .round() as i32;
                        priv_.temporary_format.set_height(
                            converted_height_px.clamp(Format::HEIGHT_MIN, Format::HEIGHT_MAX),
                        );

                        priv_
                            .format_height_entry
                            .style_context()
                            .remove_class("error");
                        priv_.format_height_entry.style_context().add_class("plain");
                    } else {
                        priv_
                            .format_height_entry
                            .style_context()
                            .remove_class("plain");
                        priv_.format_height_entry.style_context().add_class("error");
                    }
                }
            }
        }
    }
}
