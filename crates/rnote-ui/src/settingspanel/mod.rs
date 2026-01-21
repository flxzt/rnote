// Modules
mod penshortcutmodels;
mod penshortcutrow;

// Re-exports
pub(crate) use penshortcutrow::RnPenShortcutRow;
use rnote_compose::ext::Vector2Ext;

// Imports
use crate::{RnAppWindow, RnIconPicker, RnUnitEntry};
use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use gtk4::{
    Adjustment, Button, ColorDialogButton, CompositeTemplate, MenuButton, ScrolledWindow,
    StringList, ToggleButton, Widget, gdk, glib, glib::clone, subclass::prelude::*,
};
use num_traits::ToPrimitive;
use rnote_compose::penevent::ShortcutKey;
use rnote_engine::WidgetFlags;
use rnote_engine::document::Layout;
use rnote_engine::document::background::PatternStyle;
use rnote_engine::document::config::{SpellcheckConfig, SpellcheckConfigLanguage};
use rnote_engine::document::format::{self, Format, PredefinedFormat};
use rnote_engine::engine::{SPELLCHECK_AUTOMATIC_LANGUAGE, SPELLCHECK_AVAILABLE_LANGUAGES};
use rnote_engine::ext::GdkRGBAExt;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/settingspanel.ui")]
    pub(crate) struct RnSettingsPanel {
        pub(crate) temporary_format: RefCell<Format>,
        pub(crate) app_restart_toast_singleton: RefCell<Option<adw::Toast>>,
        /// 0 = None, 1.. = available languages
        pub(crate) available_spellcheck_languages: RefCell<Vec<String>>,

        #[template_child]
        pub(crate) settings_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) general_autosave_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) general_autosave_interval_secs_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) general_show_scrollbars_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) general_optimize_epd_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) general_inertial_scrolling_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) general_regular_cursor_picker: TemplateChild<RnIconPicker>,
        #[template_child]
        pub(crate) general_regular_cursor_picker_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) general_show_drawing_cursor_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker: TemplateChild<RnIconPicker>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) format_predefined_formats_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) format_save_preset_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) format_restore_preset_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) format_orientation_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_orientation_portrait_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) format_orientation_landscape_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) format_width_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_width_unitentry: TemplateChild<RnUnitEntry>,
        #[template_child]
        pub(crate) format_height_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_height_unitentry: TemplateChild<RnUnitEntry>,
        #[template_child]
        pub(crate) format_dpi_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) format_dpi_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub(crate) format_revert_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) format_apply_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) doc_preferences_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub(crate) doc_save_preset_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) doc_restore_preset_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) doc_document_layout_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) doc_show_format_borders_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) doc_format_border_color_button: TemplateChild<ColorDialogButton>,
        #[template_child]
        pub(crate) doc_background_color_button: TemplateChild<ColorDialogButton>,
        #[template_child]
        pub(crate) doc_background_patterns_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) doc_background_pattern_color_button: TemplateChild<ColorDialogButton>,
        #[template_child]
        pub(crate) doc_background_pattern_width_unitentry: TemplateChild<RnUnitEntry>,
        #[template_child]
        pub(crate) doc_background_pattern_height_unitentry: TemplateChild<RnUnitEntry>,
        #[template_child]
        pub(crate) doc_show_origin_indicator_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) doc_spellcheck_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) doc_spellcheck_language_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) background_pattern_invert_color_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) penshortcut_stylus_button_primary_row: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_stylus_button_secondary_row: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_mouse_button_secondary_row: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_touch_two_finger_long_press_row: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_keyboard_ctrl_space_row: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_drawing_pad_button_0: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_drawing_pad_button_1: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_drawing_pad_button_2: TemplateChild<RnPenShortcutRow>,
        #[template_child]
        pub(crate) penshortcut_drawing_pad_button_3: TemplateChild<RnPenShortcutRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnSettingsPanel {
        const NAME: &'static str = "RnSettingsPanel";
        type Type = super::RnSettingsPanel;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnSettingsPanel {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.format_predefined_formats_row
                .connect_selected_item_notify(clone!(
                    #[weak(rename_to=settings_panel)]
                    obj,
                    move |_| {
                        settings_panel.imp().apply_predefined_format();
                    }
                ));

            self.format_orientation_portrait_toggle
                .connect_toggled(clone!(
                    #[weak(rename_to=settings_panel)]
                    obj,
                    move |toggle| {
                        if toggle.is_active()
                            && settings_panel.format_orientation()
                                != settings_panel.imp().temporary_format.borrow().orientation()
                        {
                            settings_panel.imp().swap_width_height();
                        }
                    }
                ));

            self.format_orientation_landscape_toggle
                .connect_toggled(clone!(
                    #[weak(rename_to=settings_panel)]
                    obj,
                    move |toggle| {
                        if toggle.is_active()
                            && settings_panel.format_orientation()
                                != settings_panel.imp().temporary_format.borrow().orientation()
                        {
                            settings_panel.imp().swap_width_height();
                        }
                    }
                ));

            self.format_width_unitentry.get().connect_notify_local(
                Some("value"),
                clone!(
                    #[weak(rename_to=settings_panel)]
                    obj,
                    move |entry, _| {
                        settings_panel
                            .imp()
                            .temporary_format
                            .borrow_mut()
                            .set_width(entry.value_in_px());
                        settings_panel.imp().update_orientation_toggles();
                    }
                ),
            );

            self.format_height_unitentry.get().connect_notify_local(
                Some("value"),
                clone!(
                    #[weak(rename_to=settings_panel)]
                    obj,
                    move |entry, _| {
                        settings_panel
                            .imp()
                            .temporary_format
                            .borrow_mut()
                            .set_height(entry.value_in_px());
                        settings_panel.imp().update_orientation_toggles();
                    }
                ),
            );

            self.format_dpi_adj.connect_value_changed(clone!(
                #[weak(rename_to=settings_panel)]
                obj,
                move |adj| {
                    let dpi = adj.value();
                    settings_panel
                        .imp()
                        .format_width_unitentry
                        .set_dpi_keep_value(dpi);
                    settings_panel
                        .imp()
                        .format_height_unitentry
                        .set_dpi_keep_value(dpi);
                    settings_panel
                        .imp()
                        .temporary_format
                        .borrow_mut()
                        .set_dpi(adj.value());
                }
            ));
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnSettingsPanel {}

    impl RnSettingsPanel {
        fn update_orientation_toggles(&self) {
            let width = self.format_width_unitentry.value_in_px();
            let height = self.format_height_unitentry.value_in_px();
            let orientation = if width <= height {
                format::Orientation::Portrait
            } else {
                format::Orientation::Landscape
            };
            self.obj().set_format_orientation(orientation);
        }

        fn swap_width_height(&self) {
            let width = self.format_width_unitentry.value_in_px();
            let height = self.format_height_unitentry.value_in_px();
            self.temporary_format.borrow_mut().set_width(height);
            self.temporary_format.borrow_mut().set_height(width);
            self.format_width_unitentry.set_value_in_px(height);
            self.format_height_unitentry.set_value_in_px(width);
        }

        fn apply_predefined_format(&self) {
            let predefined_format = self.obj().format_predefined_format();
            let orientation = self.temporary_format.borrow().orientation();

            if let Some(predefined_size_mm) = predefined_format.size_mm(orientation) {
                // reset to mm as default for presets
                self.format_width_unitentry
                    .get()
                    .set_unit(format::MeasureUnit::Mm);
                self.format_height_unitentry
                    .get()
                    .set_unit(format::MeasureUnit::Mm);
                self.format_width_unitentry
                    .get()
                    .set_value(predefined_size_mm[0]);
                self.format_height_unitentry
                    .get()
                    .set_value(predefined_size_mm[1]);
            }

            match predefined_format {
                PredefinedFormat::Custom => {
                    self.format_width_row.set_sensitive(true);
                    self.format_height_row.set_sensitive(true);
                }
                _ => {
                    self.format_width_row.set_sensitive(false);
                    self.format_height_row.set_sensitive(false);
                }
            };
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnSettingsPanel(ObjectSubclass<imp::RnSettingsPanel>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnSettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl RnSettingsPanel {
    pub(crate) fn new() -> Self {
        glib::Object::new()
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
        PatternStyle::try_from(self.imp().doc_background_patterns_row.get().selected()).unwrap()
    }

    pub(crate) fn set_background_pattern(&self, pattern: PatternStyle) {
        let position = pattern.to_u32().unwrap();

        self.imp()
            .doc_background_patterns_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn spellcheck_language(&self) -> SpellcheckConfigLanguage {
        let position = self.imp().doc_spellcheck_language_row.selected();

        if position == 0 {
            SpellcheckConfigLanguage::Automatic
        } else {
            SpellcheckConfigLanguage::Language(
                self.imp()
                    .available_spellcheck_languages
                    .borrow()
                    .get((position - 1) as usize)
                    .unwrap()
                    .to_owned(),
            )
        }
    }

    pub(crate) fn set_spellcheck_language(&self, language: &SpellcheckConfigLanguage) {
        match language {
            SpellcheckConfigLanguage::Automatic => {
                self.imp().doc_spellcheck_language_row.set_selected(0);
            }
            SpellcheckConfigLanguage::Language(language) => {
                if let Some(position) = self
                    .imp()
                    .available_spellcheck_languages
                    .borrow()
                    .iter()
                    .position(|l| l == language)
                {
                    self.imp()
                        .doc_spellcheck_language_row
                        .set_selected((position + 1) as u32);
                }
            }
        }
    }

    pub(crate) fn set_spellcheck_enabled(&self, enabled: bool) {
        self.imp().doc_spellcheck_row.set_active(enabled);
    }

    pub(crate) fn set_spellcheck_config(&self, config: &SpellcheckConfig) {
        self.set_spellcheck_enabled(config.enabled);
        self.set_spellcheck_language(&config.language);
    }

    #[allow(unused)]
    pub(crate) fn format_orientation(&self) -> format::Orientation {
        if self.imp().format_orientation_portrait_toggle.is_active() {
            format::Orientation::Portrait
        } else {
            format::Orientation::Landscape
        }
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

    pub(crate) fn general_regular_cursor_picker(&self) -> RnIconPicker {
        self.imp().general_regular_cursor_picker.clone()
    }

    pub(crate) fn general_show_drawing_cursor_row(&self) -> adw::SwitchRow {
        self.imp().general_show_drawing_cursor_row.clone()
    }

    pub(crate) fn general_drawing_cursor_picker(&self) -> RnIconPicker {
        self.imp().general_drawing_cursor_picker.clone()
    }

    pub(crate) fn general_show_scrollbars_row(&self) -> adw::SwitchRow {
        self.imp().general_show_scrollbars_row.clone()
    }

    pub(crate) fn general_inertial_scrolling_row(&self) -> adw::SwitchRow {
        self.imp().general_inertial_scrolling_row.clone()
    }

    pub(crate) fn document_layout(&self) -> Layout {
        Layout::try_from(self.imp().doc_document_layout_row.get().selected()).unwrap()
    }

    pub(crate) fn set_document_layout(&self, layout: &Layout) {
        self.imp()
            .doc_document_layout_row
            .set_selected(layout.to_u32().unwrap());
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnAppWindow) {
        self.refresh_general_ui(appwindow);
        self.refresh_format_ui(appwindow);
        self.refresh_doc_ui(appwindow);
        self.refresh_shortcuts_ui(appwindow);
    }

    fn refresh_general_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let canvas = appwindow.active_tab_canvas();

        let optimize_epd = appwindow.engine_config().read().optimize_epd;
        imp.general_optimize_epd_row.set_active(optimize_epd);

        if let Some(canvas) = canvas {
            let format_border_color = canvas.engine_ref().document.config.format.border_color;

            imp.doc_format_border_color_button
                .set_rgba(&gdk::RGBA::from_compose_color(format_border_color));
        }
    }

    fn refresh_format_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let canvas = appwindow.active_tab_canvas();

        if let Some(canvas) = canvas {
            let format = canvas.engine_ref().document.config.format;
            *self.imp().temporary_format.borrow_mut() = format;

            self.set_format_predefined_format_variant(format::PredefinedFormat::Custom);
            self.set_format_orientation(format.orientation());
            imp.format_dpi_adj.set_value(format.dpi());
            imp.format_width_unitentry.set_dpi(format.dpi());
            imp.format_width_unitentry.set_value_in_px(format.width());
            imp.format_height_unitentry.set_dpi(format.dpi());
            imp.format_height_unitentry.set_value_in_px(format.height());
        }
        // TODO: else insensitive  options
    }

    fn refresh_doc_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let canvas = appwindow.active_tab_canvas();
        imp.doc_preferences_group.set_sensitive(canvas.is_some());

        if let Some(canvas) = canvas {
            let background = canvas.engine_ref().document.config.background;
            let format = canvas.engine_ref().document.config.format;
            let document_layout = canvas.engine_ref().document.config.layout;
            let show_format_borders = canvas.engine_ref().document.config.format.show_borders;
            let show_origin_indicator = canvas
                .engine_ref()
                .document
                .config
                .format
                .show_origin_indicator;
            let spellcheck_config = canvas.engine_ref().document.config.spellcheck.clone();

            imp.doc_show_format_borders_row
                .set_active(show_format_borders);
            imp.doc_background_pattern_color_button
                .set_rgba(&gdk::RGBA::from_compose_color(background.pattern_color));
            imp.doc_background_color_button
                .set_rgba(&gdk::RGBA::from_compose_color(background.color));
            self.set_background_pattern(background.pattern);
            imp.doc_background_pattern_width_unitentry
                .set_dpi(format.dpi());
            imp.doc_background_pattern_width_unitentry
                .set_value_in_px(background.pattern_size[0]);
            imp.doc_background_pattern_height_unitentry
                .set_dpi(format.dpi());
            imp.doc_background_pattern_height_unitentry
                .set_value_in_px(background.pattern_size[1]);
            self.set_document_layout(&document_layout);
            imp.doc_show_origin_indicator_row
                .set_active(show_origin_indicator);
            self.set_spellcheck_config(&spellcheck_config);
        }
    }

    fn refresh_shortcuts_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let current_shortcuts = appwindow
            .engine_config()
            .read()
            .pens_config
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
                ShortcutKey::TouchTwoFingerLongPress => {
                    imp.penshortcut_touch_two_finger_long_press_row
                        .set_action(action);
                }
                ShortcutKey::KeyboardCtrlSpace => {
                    imp.penshortcut_keyboard_ctrl_space_row.set_action(action);
                }
                ShortcutKey::DrawingPadButton0 => {
                    imp.penshortcut_drawing_pad_button_0.set_action(action);
                }
                ShortcutKey::DrawingPadButton1 => {
                    imp.penshortcut_drawing_pad_button_1.set_action(action);
                }
                ShortcutKey::DrawingPadButton2 => {
                    imp.penshortcut_drawing_pad_button_2.set_action(action);
                }
                ShortcutKey::DrawingPadButton3 => {
                    imp.penshortcut_drawing_pad_button_3.set_action(action);
                }
            });
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.setup_general(appwindow);
        self.setup_format(appwindow);
        self.setup_doc(appwindow);
        self.setup_shortcuts(appwindow);
    }

    fn setup_general(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        // autosave enable row
        imp.general_autosave_row
            .bind_property("active", appwindow, "autosave")
            .sync_create()
            .bidirectional()
            .build();

        imp.general_autosave_row
            .get()
            .bind_property(
                "active",
                &*imp.general_autosave_interval_secs_row,
                "sensitive",
            )
            .sync_create()
            .build();

        imp.general_autosave_interval_secs_row
            .get()
            .bind_property("value", appwindow, "autosave-interval-secs")
            .transform_to(|_, val: f64| Some((val.round() as u32).to_value()))
            .transform_from(|_, val: u32| Some(f64::from(val).to_value()))
            .sync_create()
            .bidirectional()
            .build();

        let set_overlays_margins = |appwindow: &RnAppWindow, row_active: bool| {
            let (m1, m2) = if row_active { (18, 72) } else { (9, 63) };
            appwindow.overlays().colorpicker().set_margin_top(m1);
            appwindow.overlays().penpicker().set_margin_bottom(m1);
            appwindow.overlays().sidebar_box().set_margin_start(m1);
            appwindow.overlays().sidebar_box().set_margin_end(m1);
            appwindow.overlays().sidebar_box().set_margin_top(m2);
            appwindow.overlays().sidebar_box().set_margin_bottom(m2);
        };
        // set on init
        set_overlays_margins(appwindow, imp.general_show_scrollbars_row.is_active());
        // and on change
        imp.general_show_scrollbars_row
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    set_overlays_margins(&appwindow, row.is_active());
                }
            ));

        imp.general_optimize_epd_row
            .bind_property(
                "active",
                &appwindow.overlays().colorpicker().active_color_label(),
                "visible",
            )
            .sync_create()
            .build();

        imp.general_optimize_epd_row.connect_active_notify(clone!(
            #[weak]
            appwindow,
            move |row| {
                let optimize_epd = row.is_active();
                appwindow.engine_config().write().optimize_epd = optimize_epd;
            }
        ));

        // Regular cursor picker
        imp.general_regular_cursor_picker.set_list(
            StringList::new(CURSORS_LIST),
            Some(cursors_list_to_display_name),
            true,
        );

        imp.general_regular_cursor_picker
            .bind_property(
                "picked",
                &*imp.general_regular_cursor_picker_menubutton,
                "icon-name",
            )
            .sync_create()
            .build();

        // insensitive picker when drawing cursor is hidden
        imp.general_show_drawing_cursor_row
            .bind_property(
                "active",
                &*imp.general_drawing_cursor_picker_row,
                "sensitive",
            )
            .sync_create()
            .build();

        // Drawing cursor picker
        imp.general_drawing_cursor_picker.set_list(
            StringList::new(CURSORS_LIST),
            Some(cursors_list_to_display_name),
            true,
        );

        imp.general_drawing_cursor_picker
            .bind_property(
                "picked",
                &*imp.general_drawing_cursor_picker_menubutton,
                "icon-name",
            )
            .sync_create()
            .build();

        imp.general_inertial_scrolling_row
            .connect_active_notify(clone!(
                #[weak(rename_to=settingspanel)]
                self,
                #[weak]
                appwindow,
                move |row| {
                    if !row.is_active() {
                        appwindow.overlays().dispatch_toast_text_singleton(
                            &gettext("Application restart is required"),
                            None,
                            &mut settingspanel.imp().app_restart_toast_singleton.borrow_mut(),
                        );
                    }
                }
            ));
    }

    fn setup_format(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.format_save_preset_button.get().connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                let doc_config = canvas.engine_ref().document.config.clone();
                appwindow
                    .document_config_preset_mut()
                    .format
                    .set_width(doc_config.format.width());
                appwindow
                    .document_config_preset_mut()
                    .format
                    .set_height(doc_config.format.height());
                appwindow
                    .document_config_preset_mut()
                    .format
                    .set_dpi(doc_config.format.dpi());

                let widget_flags = WidgetFlags {
                    refresh_ui: true,
                    ..Default::default()
                };
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        imp.format_restore_preset_button
            .get()
            .connect_clicked(clone!(
                #[weak]
                appwindow,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    let doc_config = appwindow.document_config_preset_ref().clone();
                    canvas
                        .engine_mut()
                        .document
                        .config
                        .format
                        .set_width(doc_config.format.width());
                    canvas
                        .engine_mut()
                        .document
                        .config
                        .format
                        .set_height(doc_config.format.height());
                    canvas
                        .engine_mut()
                        .document
                        .config
                        .format
                        .set_dpi(doc_config.format.dpi());

                    let mut widget_flags = canvas.engine_mut().doc_resize_autoexpand();
                    widget_flags |= canvas.engine_mut().background_rendering_regenerate();
                    widget_flags.refresh_ui = true;
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));

        // revert format
        imp.format_revert_button.get().connect_clicked(clone!(
            #[weak(rename_to=settings_panel)]
            self,
            #[weak]
            appwindow,
            move |_format_revert_button| {
                settings_panel.revert_format(&appwindow);
            }
        ));

        // Apply format
        imp.format_apply_button.get().connect_clicked(clone!(
            #[weak(rename_to=settingspanel)]
            self,
            #[weak]
            appwindow,
            move |_| {
                settingspanel.apply_format(&appwindow);
            }
        ));
    }

    fn setup_doc(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.doc_save_preset_button.get().connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                let doc_config = canvas.engine_ref().document.config.clone();
                appwindow.document_config_preset_mut().layout = doc_config.layout;
                appwindow.document_config_preset_mut().format.border_color =
                    doc_config.format.border_color;
                appwindow.document_config_preset_mut().background.color =
                    doc_config.background.color;
                appwindow.document_config_preset_mut().background.pattern =
                    doc_config.background.pattern;
                appwindow
                    .document_config_preset_mut()
                    .background
                    .pattern_size = doc_config.background.pattern_size;
                appwindow
                    .document_config_preset_mut()
                    .background
                    .pattern_color = doc_config.background.pattern_color;
                appwindow.document_config_preset_mut().format.show_borders =
                    doc_config.format.show_borders;
                appwindow
                    .document_config_preset_mut()
                    .format
                    .show_origin_indicator = doc_config.format.show_origin_indicator;
                appwindow.document_config_preset_mut().spellcheck = doc_config.spellcheck;

                let widget_flags = WidgetFlags {
                    refresh_ui: true,
                    ..Default::default()
                };
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        imp.doc_restore_preset_button.get().connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                let doc_config = appwindow.document_config_preset_ref().clone();
                canvas.engine_mut().document.config.layout = doc_config.layout;
                canvas.engine_mut().document.config.format.border_color =
                    doc_config.format.border_color;
                canvas.engine_mut().document.config.background.color = doc_config.background.color;
                canvas.engine_mut().document.config.background.pattern =
                    doc_config.background.pattern;
                canvas.engine_mut().document.config.background.pattern_size =
                    doc_config.background.pattern_size;
                canvas.engine_mut().document.config.background.pattern_color =
                    doc_config.background.pattern_color;
                canvas.engine_mut().document.config.format.show_borders =
                    doc_config.format.show_borders;
                canvas
                    .engine_mut()
                    .document
                    .config
                    .format
                    .show_origin_indicator = doc_config.format.show_origin_indicator;
                canvas.engine_mut().document.config.spellcheck = doc_config.spellcheck;

                let mut widget_flags = canvas.engine_mut().doc_resize_autoexpand();
                widget_flags |= canvas.engine_mut().background_rendering_regenerate();
                widget_flags.refresh_ui = true;
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        imp.doc_show_format_borders_row
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    canvas.engine_mut().document.config.format.show_borders = row.is_active();
                    canvas.queue_draw();
                }
            ));

        imp.doc_show_format_borders_row
            .bind_property(
                "active",
                &imp.doc_format_border_color_button.get(),
                "sensitive",
            )
            .sync_create()
            .build();

        imp.doc_format_border_color_button
            .connect_rgba_notify(clone!(
                #[weak(rename_to=settingspanel)]
                self,
                #[weak]
                appwindow,
                move |button| {
                    let format_border_color = button.rgba().into_compose_color();
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    // Because the format border color is applied immediately to the engine,
                    // we need to update the temporary format too.
                    settingspanel
                        .imp()
                        .temporary_format
                        .borrow_mut()
                        .border_color = format_border_color;
                    let current_color = canvas.engine_ref().document.config.format.border_color;

                    if !current_color.approx_eq_f32(format_border_color) {
                        canvas.engine_mut().document.config.format.border_color =
                            format_border_color;
                        let mut widget_flags =
                            canvas.engine_mut().update_rendering_current_viewport();
                        widget_flags.store_modified = true;
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));

        imp.doc_background_color_button.connect_rgba_notify(clone!(
            #[weak]
            appwindow,
            move |button| {
                let background_color = button.rgba().into_compose_color();
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                if !canvas
                    .engine_ref()
                    .document
                    .config
                    .background
                    .color
                    .approx_eq_f32(background_color)
                {
                    canvas.engine_mut().document.config.background.color = background_color;
                    let mut widget_flags = canvas.engine_mut().background_rendering_regenerate();
                    widget_flags.store_modified = true;
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            }
        ));

        imp.doc_document_layout_row
            .get()
            .connect_selected_item_notify(clone!(
                #[weak(rename_to=settings_panel)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    let document_layout = settings_panel.document_layout();
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    appwindow
                        .main_header()
                        .canvasmenu()
                        .fixedsize_quickactions_box()
                        .set_sensitive(document_layout == Layout::FixedSize);

                    if canvas.engine_ref().document.config.layout != document_layout {
                        let mut widget_flags = canvas.engine_mut().set_doc_layout(document_layout);
                        widget_flags.store_modified = true;
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));

        imp.doc_background_patterns_row
            .get()
            .connect_selected_item_notify(clone!(
                #[weak(rename_to=settings_panel)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    let pattern = settings_panel.background_pattern();
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    match pattern {
                        PatternStyle::None => {
                            settings_panel
                                .imp()
                                .doc_background_pattern_width_unitentry
                                .set_sensitive(false);
                            settings_panel
                                .imp()
                                .doc_background_pattern_height_unitentry
                                .set_sensitive(false);
                        }
                        PatternStyle::Lines => {
                            settings_panel
                                .imp()
                                .doc_background_pattern_width_unitentry
                                .set_sensitive(false);
                            settings_panel
                                .imp()
                                .doc_background_pattern_height_unitentry
                                .set_sensitive(true);
                        }
                        PatternStyle::Grid => {
                            settings_panel
                                .imp()
                                .doc_background_pattern_width_unitentry
                                .set_sensitive(true);
                            settings_panel
                                .imp()
                                .doc_background_pattern_height_unitentry
                                .set_sensitive(true);
                        }
                        PatternStyle::Dots => {
                            settings_panel
                                .imp()
                                .doc_background_pattern_width_unitentry
                                .set_sensitive(true);
                            settings_panel
                                .imp()
                                .doc_background_pattern_height_unitentry
                                .set_sensitive(true);
                        }
                        PatternStyle::IsometricGrid => {
                            settings_panel
                                .imp()
                                .doc_background_pattern_width_unitentry
                                .set_sensitive(false);
                            settings_panel
                                .imp()
                                .doc_background_pattern_height_unitentry
                                .set_sensitive(true);
                        }
                        PatternStyle::IsometricDots => {
                            settings_panel
                                .imp()
                                .doc_background_pattern_width_unitentry
                                .set_sensitive(false);
                            settings_panel
                                .imp()
                                .doc_background_pattern_height_unitentry
                                .set_sensitive(true);
                        }
                    }

                    if canvas.engine_ref().document.config.background.pattern != pattern {
                        canvas.engine_mut().document.config.background.pattern = pattern;
                        let mut widget_flags =
                            canvas.engine_mut().background_rendering_regenerate();
                        widget_flags.store_modified = true;
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));

        imp.doc_background_pattern_color_button
            .connect_rgba_notify(clone!(
                #[weak]
                appwindow,
                move |button| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let pattern_color = button.rgba().into_compose_color();

                    if !canvas
                        .engine_ref()
                        .document
                        .config
                        .background
                        .pattern_color
                        .approx_eq_f32(pattern_color)
                    {
                        canvas.engine_mut().document.config.background.pattern_color =
                            pattern_color;
                        let mut widget_flags =
                            canvas.engine_mut().background_rendering_regenerate();
                        widget_flags.store_modified = true;
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                }
            ));

        imp.doc_background_pattern_width_unitentry
            .get()
            .connect_notify_local(
                Some("value"),
                clone!(
                    #[weak]
                    appwindow,
                    move |unit_entry, _| {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        let mut pattern_size =
                            canvas.engine_ref().document.config.background.pattern_size;
                        pattern_size[0] = unit_entry.value_in_px();

                        if !canvas
                            .engine_ref()
                            .document
                            .config
                            .background
                            .pattern_size
                            .approx_eq(&pattern_size)
                        {
                            canvas.engine_mut().document.config.background.pattern_size =
                                pattern_size;
                            let mut widget_flags =
                                canvas.engine_mut().background_rendering_regenerate();
                            widget_flags.store_modified = true;
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                    }
                ),
            );

        imp.doc_background_pattern_height_unitentry
            .get()
            .connect_notify_local(
                Some("value"),
                clone!(
                    #[weak]
                    appwindow,
                    move |unit_entry, _| {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        let mut pattern_size =
                            canvas.engine_ref().document.config.background.pattern_size;
                        pattern_size[1] = unit_entry.value_in_px();

                        if !canvas
                            .engine_ref()
                            .document
                            .config
                            .background
                            .pattern_size
                            .approx_eq(&pattern_size)
                        {
                            canvas.engine_mut().document.config.background.pattern_size =
                                pattern_size;
                            let mut widget_flags =
                                canvas.engine_mut().background_rendering_regenerate();
                            widget_flags.store_modified = true;
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                    }
                ),
            );

        imp.doc_show_origin_indicator_row
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    canvas
                        .engine_mut()
                        .document
                        .config
                        .format
                        .show_origin_indicator = row.is_active();
                    canvas.queue_draw();
                }
            ));

        imp.doc_spellcheck_row
            .get()
            .bind_property("active", &*imp.doc_spellcheck_language_row, "sensitive")
            .sync_create()
            .build();

        imp.doc_spellcheck_row.get().connect_active_notify(clone!(
            #[weak]
            appwindow,
            move |row| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                canvas.engine_mut().document.config.spellcheck.enabled = row.is_active();

                let widget_flags = canvas.engine_mut().refresh_spellcheck_language();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        imp.available_spellcheck_languages
            .replace(SPELLCHECK_AVAILABLE_LANGUAGES.clone());

        imp.doc_spellcheck_language_row.get().set_model(Some(
            &std::iter::once(format!(
                "{} ({})",
                gettext("Automatic"),
                SPELLCHECK_AUTOMATIC_LANGUAGE.unwrap_or(&gettext("None"))
            ))
            .chain(imp.available_spellcheck_languages.borrow().iter().cloned())
            .collect::<StringList>(),
        ));

        imp.doc_spellcheck_language_row
            .get()
            .connect_selected_item_notify(clone!(
                #[weak(rename_to=settings_panel)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    let language = settings_panel.spellcheck_language();
                    canvas.engine_mut().document.config.spellcheck.language = language;

                    let widget_flags = canvas.engine_mut().refresh_spellcheck_language();
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));

        imp.background_pattern_invert_color_button
            .get()
            .connect_clicked(clone!(
                #[weak]
                appwindow,
                move |_| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    let mut widget_flags = {
                        let mut engine = canvas.engine_mut();
                        engine.document.config.background.color = engine
                            .document
                            .config
                            .background
                            .color
                            .to_inverted_brightness_color();
                        engine.document.config.background.pattern_color = engine
                            .document
                            .config
                            .background
                            .pattern_color
                            .to_inverted_brightness_color();
                        engine.document.config.format.border_color = engine
                            .document
                            .config
                            .format
                            .border_color
                            .to_inverted_brightness_color();
                        engine.background_rendering_regenerate()
                    };

                    widget_flags.refresh_ui = true;
                    widget_flags.store_modified = true;
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ));
    }

    fn setup_shortcuts(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let penshortcut_stylus_button_primary_row = imp.penshortcut_stylus_button_primary_row.get();
        let penshortcut_stylus_button_secondary_row =
            imp.penshortcut_stylus_button_secondary_row.get();
        let penshortcut_mouse_button_secondary_row =
            imp.penshortcut_mouse_button_secondary_row.get();
        let penshortcut_touch_two_finger_long_press_row =
            imp.penshortcut_touch_two_finger_long_press_row.get();
        let penshortcut_keyboard_ctrl_space_row = imp.penshortcut_keyboard_ctrl_space_row.get();
        let penshortcut_drawing_pad_button_0 = imp.penshortcut_drawing_pad_button_0.get();
        let penshortcut_drawing_pad_button_1 = imp.penshortcut_drawing_pad_button_1.get();
        let penshortcut_drawing_pad_button_2 = imp.penshortcut_drawing_pad_button_2.get();
        let penshortcut_drawing_pad_button_3 = imp.penshortcut_drawing_pad_button_3.get();

        imp.penshortcut_stylus_button_primary_row.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_stylus_button_primary_row,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_stylus_button_primary_row.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::StylusPrimaryButton, action);
                    None
                }
            ),
        );

        imp.penshortcut_stylus_button_secondary_row.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_stylus_button_secondary_row,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_stylus_button_secondary_row.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::StylusSecondaryButton, action);
                    None
                }
            ),
        );

        imp.penshortcut_mouse_button_secondary_row.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_mouse_button_secondary_row,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_mouse_button_secondary_row.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::MouseSecondaryButton, action);
                    None
                }
            ),
        );

        imp.penshortcut_touch_two_finger_long_press_row
            .connect_local(
                "action-changed",
                false,
                clone!(
                    #[weak]
                    penshortcut_touch_two_finger_long_press_row,
                    #[weak]
                    appwindow,
                    #[upgrade_or]
                    None,
                    move |_values| {
                        let action = penshortcut_touch_two_finger_long_press_row.action();
                        appwindow
                            .engine_config()
                            .write()
                            .pens_config
                            .register_shortcut(ShortcutKey::TouchTwoFingerLongPress, action);
                        None
                    }
                ),
            );

        imp.penshortcut_keyboard_ctrl_space_row.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_keyboard_ctrl_space_row,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_keyboard_ctrl_space_row.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::KeyboardCtrlSpace, action);
                    None
                }
            ),
        );

        imp.penshortcut_drawing_pad_button_0.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_drawing_pad_button_0,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_drawing_pad_button_0.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::DrawingPadButton0, action);
                    None
                }
            ),
        );

        imp.penshortcut_drawing_pad_button_1.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_drawing_pad_button_1,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_drawing_pad_button_1.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::DrawingPadButton1, action);
                    None
                }
            ),
        );

        imp.penshortcut_drawing_pad_button_2.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_drawing_pad_button_2,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_drawing_pad_button_2.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::DrawingPadButton2, action);
                    None
                }
            ),
        );

        imp.penshortcut_drawing_pad_button_3.connect_local(
            "action-changed",
            false,
            clone!(
                #[weak]
                penshortcut_drawing_pad_button_3,
                #[weak]
                appwindow,
                #[upgrade_or]
                None,
                move |_values| {
                    let action = penshortcut_drawing_pad_button_3.action();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .register_shortcut(ShortcutKey::DrawingPadButton3, action);
                    None
                }
            ),
        );
    }

    fn revert_format(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let Some(canvas) = appwindow.active_tab_canvas() else {
            return;
        };
        *imp.temporary_format.borrow_mut() = canvas.engine_ref().document.config.format;
        let revert_format = canvas.engine_ref().document.config.format;

        self.set_format_predefined_format_variant(format::PredefinedFormat::Custom);
        imp.format_dpi_adj.set_value(revert_format.dpi());
        imp.format_width_unitentry.set_dpi(revert_format.dpi());
        imp.format_width_unitentry
            .set_value_in_px(revert_format.width());
        imp.format_height_unitentry.set_dpi(revert_format.dpi());
        imp.format_height_unitentry
            .set_value_in_px(revert_format.height());
    }

    fn apply_format(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let Some(canvas) = appwindow.active_tab_canvas() else {
            return;
        };
        let temporary_format = *imp.temporary_format.borrow();

        imp.doc_background_pattern_width_unitentry
            .set_dpi_keep_value(temporary_format.dpi());
        imp.doc_background_pattern_height_unitentry
            .set_dpi_keep_value(temporary_format.dpi());

        canvas.engine_mut().document.config.format = temporary_format;
        let mut widget_flags = canvas.engine_mut().doc_resize_to_fit_content();
        widget_flags.store_modified = true;
        appwindow.handle_widget_flags(widget_flags, &canvas);
    }
}

const CURSORS_LIST: &[&str] = &[
    "cursor-crosshair-small",
    "cursor-crosshair-medium",
    "cursor-crosshair-large",
    "cursor-dot-small",
    "cursor-dot-medium",
    "cursor-dot-large",
    "cursor-teardrop-nw-small",
    "cursor-teardrop-nw-medium",
    "cursor-teardrop-nw-large",
    "cursor-teardrop-ne-small",
    "cursor-teardrop-ne-medium",
    "cursor-teardrop-ne-large",
    "cursor-teardrop-n-small",
    "cursor-teardrop-n-medium",
    "cursor-teardrop-n-large",
    "cursor-beam-small",
    "cursor-beam-medium",
    "cursor-beam-large",
];

fn cursors_list_to_display_name(icon_name: &str) -> String {
    match icon_name {
        "cursor-crosshair-small" => pgettext("a cursor type", "Crosshair (Small)"),
        "cursor-crosshair-medium" => pgettext("a cursor type", "Crosshair (Medium)"),
        "cursor-crosshair-large" => pgettext("a cursor type", "Crosshair (Large)"),
        "cursor-dot-small" => pgettext("a cursor type", "Dot (Small)"),
        "cursor-dot-medium" => pgettext("a cursor type", "Dot (Medium)"),
        "cursor-dot-large" => pgettext("a cursor type", "Dot (Large)"),
        "cursor-teardrop-nw-small" => pgettext("a cursor type", "Teardrop North-West (Small)"),
        "cursor-teardrop-nw-medium" => pgettext("a cursor type", "Teardrop North-West (Medium)"),
        "cursor-teardrop-nw-large" => pgettext("a cursor type", "Teardrop North-West (Large)"),
        "cursor-teardrop-ne-small" => pgettext("a cursor type", "Teardrop North-East (Small)"),
        "cursor-teardrop-ne-medium" => pgettext("a cursor type", "Teardrop North-East (Medium)"),
        "cursor-teardrop-ne-large" => pgettext("a cursor type", "Teardrop North-East (Large)"),
        "cursor-teardrop-n-small" => pgettext("a cursor type", "Teardrop North (Small)"),
        "cursor-teardrop-n-medium" => pgettext("a cursor type", "Teardrop North (Medium)"),
        "cursor-teardrop-n-large" => pgettext("a cursor type", "Teardrop North (Large)"),
        "cursor-beam-small" => pgettext("a cursor type", "Beam (Small)"),
        "cursor-beam-medium" => pgettext("a cursor type", "Beam (Medium)"),
        "cursor-beam-large" => pgettext("a cursor type", "Beam (Large)"),
        _ => unimplemented!(),
    }
}
