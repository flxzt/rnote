pub(crate) mod penshortcutmodels;
mod penshortcutrow;

use gtk4::{Image, StringList};
// Re-exports
pub(crate) use penshortcutrow::PenShortcutRow;

use adw::prelude::*;
use gtk4::{
    gdk, glib, glib::clone, subclass::prelude::*, Adjustment, Button, ColorButton,
    CompositeTemplate, ScrolledWindow, SpinButton, Switch, ToggleButton, Widget,
};
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;

use super::appwindow::RnoteAppWindow;
use crate::globals;
use crate::unitentry::UnitEntry;
use crate::IconPicker;
use rnote_compose::penevents::ShortcutKey;
use rnote_engine::document::background::PatternStyle;
use rnote_engine::document::format::{self, Format, PredefinedFormat};
use rnote_engine::utils::GdkRGBAHelpers;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/settingspanel.ui")]
    pub(crate) struct SettingsPanel {
        pub(crate) temporary_format: Rc<RefCell<Format>>,

        #[template_child]
        pub(crate) settings_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) general_autosave_enable_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) general_autosave_interval_secs_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) general_autosave_interval_secs_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) general_format_border_color_choosebutton: TemplateChild<ColorButton>,
        #[template_child]
        pub(crate) general_permanently_hide_scrollbars_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) general_regular_cursor_picker: TemplateChild<IconPicker>,
        #[template_child]
        pub(crate) general_regular_cursor_picker_image: TemplateChild<Image>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker: TemplateChild<IconPicker>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker_image: TemplateChild<Image>,
        #[template_child]
        pub(crate) format_predefined_formats_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) format_orientation_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_orientation_portrait_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) format_orientation_landscape_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) format_width_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_width_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub(crate) format_height_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_height_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub(crate) format_dpi_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_dpi_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub(crate) format_revert_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) format_apply_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) background_color_choosebutton: TemplateChild<ColorButton>,
        #[template_child]
        pub(crate) background_patterns_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) background_pattern_color_choosebutton: TemplateChild<ColorButton>,
        #[template_child]
        pub(crate) background_pattern_width_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub(crate) background_pattern_height_unitentry: TemplateChild<UnitEntry>,
        #[template_child]
        pub(crate) penshortcut_stylus_button_primary_row: TemplateChild<PenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_stylus_button_secondary_row: TemplateChild<PenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_mouse_button_secondary_row: TemplateChild<PenShortcutRow>,
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
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            self.format_predefined_formats_row
                .connect_selected_item_notify(
                    clone!(@weak inst as settings_panel => move |_format_predefined_formats_row| {
                        settings_panel.imp().update_temporary_format_from_rows();
                        settings_panel.imp().apply_predefined_format();
                    }),
                );

            self.format_orientation_portrait_toggle.connect_toggled(
                clone!(@weak inst as settings_panel => move |_format_orientation_portrait_toggle| {
                    settings_panel.imp().update_temporary_format_from_rows();
                    settings_panel.imp().apply_predefined_format();
                }),
            );

            self.format_orientation_landscape_toggle.connect_toggled(
                clone!(@weak inst as settings_panel => move |_format_orientation_landscape_toggle| {
                    settings_panel.imp().update_temporary_format_from_rows();
                    settings_panel.imp().apply_predefined_format();
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
                clone!(@weak inst as settings_panel => @default-return None, move |_args| {
                        settings_panel.imp().update_temporary_format_from_rows();
                        None
                }),
            );

            self.format_height_unitentry.get().connect_local(
                "measurement-changed",
                false,
                clone!(@weak inst as settings_panel => @default-return None, move |_args| {
                        settings_panel.imp().update_temporary_format_from_rows();
                        None
                }),
            );

            self.format_dpi_adj.connect_value_changed(
                clone!(@weak inst as settings_panel => move |format_dpi_adj| {
                    settings_panel.imp().update_temporary_format_from_rows();
                    settings_panel.imp().format_width_unitentry.set_dpi(format_dpi_adj.value());
                    settings_panel.imp().format_height_unitentry.set_dpi(format_dpi_adj.value());
                }),
            );
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SettingsPanel {}

    impl SettingsPanel {
        pub(crate) fn update_temporary_format_from_rows(&self) {
            // border color
            self.temporary_format.borrow_mut().border_color = self
                .general_format_border_color_choosebutton
                .rgba()
                .into_compose_color();

            // Format orientation
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
            let predefined_format = self.instance().format_predefined_format();

            let preconfigured_dimensions = predefined_format.size_portrait_mm();
            match predefined_format {
                PredefinedFormat::A6 => {
                    self.format_orientation_row.set_sensitive(true);
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
                PredefinedFormat::A5 => {
                    self.format_orientation_row.set_sensitive(true);
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
                PredefinedFormat::A4 => {
                    self.format_orientation_row.set_sensitive(true);
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
                PredefinedFormat::A3 => {
                    self.format_orientation_row.set_sensitive(true);
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
                PredefinedFormat::A2 => {
                    self.format_orientation_row.set_sensitive(true);
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
                PredefinedFormat::UsLetter => {
                    self.format_orientation_row.set_sensitive(true);
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
                PredefinedFormat::UsLegal => {
                    self.format_orientation_row.set_sensitive(true);
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
                PredefinedFormat::Custom => {
                    self.format_orientation_row.set_sensitive(false);
                    self.format_width_row.set_sensitive(true);
                    self.format_height_row.set_sensitive(true);
                    self.format_orientation_portrait_toggle.set_active(true);
                    self.temporary_format.borrow_mut().orientation = format::Orientation::Portrait;
                }
            };

            if let Some(mut preconfigured_dimensions) = preconfigured_dimensions {
                if self.temporary_format.borrow().orientation == format::Orientation::Landscape {
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

glib::wrapper! {
    pub(crate) struct SettingsPanel(ObjectSubclass<imp::SettingsPanel>)
    @extends Widget;
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsPanel {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn format_predefined_format(&self) -> PredefinedFormat {
        PredefinedFormat::try_from(self.imp().format_predefined_formats_row.get().selected())
            .unwrap()
    }

    pub(crate) fn set_format_predefined_format_variant(
        &self,
        predefined_format: format::PredefinedFormat,
    ) {
        let position = predefined_format.to_u32().unwrap();

        self.imp()
            .format_predefined_formats_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn background_pattern(&self) -> PatternStyle {
        PatternStyle::try_from(self.imp().background_patterns_row.get().selected()).unwrap()
    }

    pub(crate) fn set_background_pattern(&self, pattern: PatternStyle) {
        let position = pattern.to_u32().unwrap();

        self.imp()
            .background_patterns_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn set_format_orientation(&self, orientation: format::Orientation) {
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

    pub(crate) fn settings_scroller(&self) -> ScrolledWindow {
        self.imp().settings_scroller.clone()
    }

    pub(crate) fn general_regular_cursor_picker_image(&self) -> Image {
        self.imp().general_regular_cursor_picker_image.clone()
    }

    pub(crate) fn general_drawing_cursor_picker_image(&self) -> Image {
        self.imp().general_drawing_cursor_picker_image.clone()
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        *self.imp().temporary_format.borrow_mut() =
            appwindow.canvas().engine().borrow().document.format.clone();

        self.refresh_general(appwindow);
        self.fresh_format(appwindow);
        self.refresh_background(appwindow);
        self.refresh_shortcuts(appwindow);
    }

    fn refresh_general(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        let format_border_color = appwindow
            .canvas()
            .engine()
            .borrow()
            .document
            .format
            .border_color;

        imp.general_format_border_color_choosebutton
            .set_rgba(&gdk::RGBA::from_compose_color(format_border_color));
    }

    fn fresh_format(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();
        let format = appwindow.canvas().engine().borrow().document.format.clone();

        self.set_format_predefined_format_variant(format::PredefinedFormat::Custom);
        self.set_format_orientation(format.orientation);
        imp.format_dpi_adj.set_value(format.dpi);

        imp.format_width_unitentry.set_unit(format::MeasureUnit::Px);
        imp.format_width_unitentry.set_value(format.width);

        imp.format_height_unitentry
            .set_unit(format::MeasureUnit::Px);
        imp.format_height_unitentry.set_value(format.height);
    }

    fn refresh_background(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();
        let background = appwindow
            .canvas()
            .engine()
            .borrow()
            .document
            .background
            .clone();
        let format = appwindow.canvas().engine().borrow().document.format.clone();

        imp.background_color_choosebutton
            .set_rgba(&gdk::RGBA::from_compose_color(background.color));

        self.set_background_pattern(background.pattern);
        imp.background_pattern_color_choosebutton
            .set_rgba(&gdk::RGBA::from_compose_color(background.pattern_color));

        // Background pattern Unit Entries
        imp.background_pattern_width_unitentry.set_dpi(format.dpi);
        imp.background_pattern_width_unitentry
            .set_unit(format::MeasureUnit::Px);
        imp.background_pattern_width_unitentry
            .set_value(background.pattern_size[0]);

        imp.background_pattern_height_unitentry.set_dpi(format.dpi);
        imp.background_pattern_height_unitentry
            .set_unit(format::MeasureUnit::Px);
        imp.background_pattern_height_unitentry
            .set_value(background.pattern_size[1]);
    }

    fn refresh_shortcuts(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();
        let current_shortcuts = appwindow
            .canvas()
            .engine()
            .borrow()
            .penholder
            .list_current_shortcuts();

        current_shortcuts
            .into_iter()
            .for_each(|(key, action)| match key {
                ShortcutKey::StylusPrimaryButton => {
                    imp.penshortcut_stylus_button_primary_row.set_action(action);
                }
                ShortcutKey::StylusSecondaryButton => {
                    imp.penshortcut_stylus_button_secondary_row
                        .set_action(action);
                }
                ShortcutKey::MouseSecondaryButton => {
                    imp.penshortcut_mouse_button_secondary_row
                        .set_action(action);
                }
                _ => {}
            });
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();
        let temporary_format = imp.temporary_format.clone();
        let penshortcut_stylus_button_primary_row = imp.penshortcut_stylus_button_primary_row.get();
        let penshortcut_stylus_button_secondary_row =
            imp.penshortcut_stylus_button_secondary_row.get();
        let penshortcut_mouse_button_secondary_row =
            imp.penshortcut_mouse_button_secondary_row.get();

        // autosave enable switch
        imp.general_autosave_enable_switch
            .bind_property("state", appwindow, "autosave")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        imp.general_autosave_enable_switch
            .get()
            .bind_property(
                "state",
                &*imp.general_autosave_interval_secs_row,
                "sensitive",
            )
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
            .build();

        imp.general_autosave_interval_secs_spinbutton
            .get()
            .bind_property("value", appwindow, "autosave-interval-secs")
            .transform_to(|_, val: f64| Some((val.round() as u32).to_value()))
            .transform_from(|_, val: u32| Some(f64::from(val).to_value()))
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        // permanently hide canvas scrollbars
        imp.general_permanently_hide_scrollbars_switch
            .get()
            .bind_property(
                "state",
                &appwindow.canvas_wrapper(),
                "permanently-hide-scrollbars",
            )
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        // Regular cursor picker
        imp.general_regular_cursor_picker
            .set_list(StringList::new(globals::CURSORS_LIST));

        imp.general_regular_cursor_picker.connect_local(
            "icon-picked",
            false,
            clone!(@weak appwindow => @default-return None, move |args| {
                let picked = args[1].get::<String>().unwrap();

                appwindow.canvas().set_regular_cursor(picked);

                None
            }),
        );

        // Drawing cursor picker
        imp.general_drawing_cursor_picker
            .set_list(StringList::new(globals::CURSORS_LIST));

        imp.general_drawing_cursor_picker.connect_local(
            "icon-picked",
            false,
            clone!(@weak appwindow => @default-return None, move |args| {
                let picked = args[1].get::<String>().unwrap();

                appwindow.canvas().set_drawing_cursor(picked);

                None
            }),
        );

        // revert format
        imp.format_revert_button.get().connect_clicked(
            clone!(@weak self as settings_panel, @weak appwindow => move |_format_revert_button| {
                settings_panel.revert_format(&appwindow);
            }),
        );

        // Apply format
        imp.format_apply_button.get().connect_clicked(
            clone!(@weak temporary_format, @weak appwindow => move |_format_apply_button| {
                let temporary_format = temporary_format.borrow().clone();
                appwindow.canvas().engine().borrow_mut().document.format = temporary_format;

                appwindow.canvas().engine().borrow_mut().resize_to_fit_strokes();
                appwindow.canvas().update_engine_rendering();
            }),
        );

        // Background
        imp.background_color_choosebutton.connect_color_set(clone!(@weak appwindow => move |background_color_choosebutton| {
            appwindow.canvas().engine().borrow_mut().document.background.color = background_color_choosebutton.rgba().into_compose_color();

            appwindow.canvas().regenerate_background_pattern();
            appwindow.canvas().update_engine_rendering();
        }));

        imp.background_patterns_row.get().connect_selected_item_notify(clone!(@weak self as settings_panel, @weak appwindow => move |_background_patterns_row| {
            let pattern = settings_panel.background_pattern();

            appwindow.canvas().engine().borrow_mut().document.background.pattern = pattern;

            match pattern {
                PatternStyle::None => {
                    settings_panel.imp().background_pattern_width_unitentry.set_sensitive(false);
                    settings_panel.imp().background_pattern_height_unitentry.set_sensitive(false);
                },
                PatternStyle::Lines => {
                    settings_panel.imp().background_pattern_width_unitentry.set_sensitive(false);
                    settings_panel.imp().background_pattern_height_unitentry.set_sensitive(true);
                },
                PatternStyle::Grid => {
                    settings_panel.imp().background_pattern_width_unitentry.set_sensitive(true);
                    settings_panel.imp().background_pattern_height_unitentry.set_sensitive(true);
                },
                PatternStyle::Dots => {
                    settings_panel.imp().background_pattern_width_unitentry.set_sensitive(true);
                    settings_panel.imp().background_pattern_height_unitentry.set_sensitive(true);
                },
            }

            appwindow.canvas().regenerate_background_pattern();
            appwindow.canvas().update_engine_rendering();
        }));

        imp.background_pattern_color_choosebutton.connect_color_set(clone!(@weak appwindow => move |background_pattern_color_choosebutton| {
            appwindow.canvas().engine().borrow_mut().document.background.pattern_color = background_pattern_color_choosebutton.rgba().into_compose_color();

            appwindow.canvas().regenerate_background_pattern();
            appwindow.canvas().update_engine_rendering();
        }));

        imp.general_format_border_color_choosebutton.connect_color_set(clone!(@weak self as settingspanel, @weak appwindow => move |general_format_border_color_choosebutton| {
            let format_border_color = general_format_border_color_choosebutton.rgba().into_compose_color();
            appwindow.canvas().engine().borrow_mut().document.format.border_color = format_border_color;

            // Because the format border color is applied immediately to the engine, we need to update the temporary format too.
            settingspanel.imp().temporary_format.borrow_mut().border_color = format_border_color;

            appwindow.canvas().update_engine_rendering();
        }));

        imp.background_pattern_width_unitentry.get().connect_local(
            "measurement-changed",
            false,
            clone!(@weak self as settings_panel, @weak appwindow => @default-return None, move |_args| {
                    let mut pattern_size = appwindow.canvas().engine().borrow().document.background.pattern_size;
                    pattern_size[0] = settings_panel.imp().background_pattern_width_unitentry.value_in_px();

                    appwindow.canvas().engine().borrow_mut().document.background.pattern_size = pattern_size;

                    appwindow.canvas().regenerate_background_pattern();
                    appwindow.canvas().update_engine_rendering();
                    None
            }),
        );

        imp.background_pattern_height_unitentry.get().connect_local(
            "measurement-changed",
            false,
            clone!(@weak self as settings_panel, @weak appwindow => @default-return None, move |_args| {
                    let mut pattern_size = appwindow.canvas().engine().borrow().document.background.pattern_size;
                    pattern_size[1] = settings_panel.imp().background_pattern_height_unitentry.value_in_px();

                    appwindow.canvas().engine().borrow_mut().document.background.pattern_size = pattern_size;

                    appwindow.canvas().regenerate_background_pattern();
                    appwindow.canvas().update_engine_rendering();
                    None
            }),
        );

        // Shortcuts
        imp.penshortcut_stylus_button_primary_row
            .set_key(Some(ShortcutKey::StylusPrimaryButton));
        imp.penshortcut_stylus_button_primary_row.connect_local("action-changed", false, clone!(@weak penshortcut_stylus_button_primary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_stylus_button_primary_row.action();
            appwindow.canvas().engine().borrow_mut().penholder.register_new_shortcut(ShortcutKey::StylusPrimaryButton, action);
            None
        }));

        imp.penshortcut_stylus_button_secondary_row
            .set_key(Some(ShortcutKey::StylusSecondaryButton));
        imp.penshortcut_stylus_button_secondary_row.connect_local("action-changed", false, clone!(@weak penshortcut_stylus_button_secondary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_stylus_button_secondary_row.action();
            appwindow.canvas().engine().borrow_mut().penholder.register_new_shortcut(ShortcutKey::StylusSecondaryButton, action);
            None
        }));

        imp.penshortcut_mouse_button_secondary_row
            .set_key(Some(ShortcutKey::StylusSecondaryButton));
        imp.penshortcut_mouse_button_secondary_row.connect_local("action-changed", false, clone!(@weak penshortcut_mouse_button_secondary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_mouse_button_secondary_row.action();
            appwindow.canvas().engine().borrow_mut().penholder.register_new_shortcut(ShortcutKey::MouseSecondaryButton, action);
            None
        }));
    }

    fn revert_format(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        *imp.temporary_format.borrow_mut() =
            appwindow.canvas().engine().borrow().document.format.clone();
        let revert_format = appwindow.canvas().engine().borrow().document.format.clone();

        self.set_format_predefined_format_variant(format::PredefinedFormat::Custom);

        imp.format_dpi_adj.set_value(revert_format.dpi);

        // Setting the unit dropdowns to Px
        imp.format_width_unitentry.set_unit(format::MeasureUnit::Px);
        imp.format_height_unitentry
            .set_unit(format::MeasureUnit::Px);

        // Setting the entries, which have callbacks to update the temporary format
        imp.format_width_unitentry.set_value(revert_format.width);
        imp.format_height_unitentry.set_value(revert_format.height);
    }
}
