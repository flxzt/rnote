// Modules
mod penshortcutmodels;
mod penshortcutrow;

// Re-exports
pub(crate) use penshortcutrow::RnPenShortcutRow;

// Imports
use crate::{RnAppWindow, RnCanvasWrapper, RnIconPicker, RnUnitEntry};
use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use gtk4::{
    gdk, glib, glib::clone, subclass::prelude::*, Adjustment, Button, ColorDialogButton,
    CompositeTemplate, MenuButton, ScrolledWindow, SpinButton, StringList, Switch, ToggleButton,
    Widget,
};
use num_traits::ToPrimitive;
use rnote_compose::penevent::ShortcutKey;
use rnote_engine::document::background::PatternStyle;
use rnote_engine::document::format::{self, Format, PredefinedFormat};
use rnote_engine::ext::GdkRGBAExt;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/settingspanel.ui")]
    pub(crate) struct RnSettingsPanel {
        pub(crate) temporary_format: RefCell<Format>,
        pub(crate) app_restart_toast_singleton: RefCell<Option<adw::Toast>>,

        #[template_child]
        pub(crate) settings_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) general_autosave_enable_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) general_autosave_interval_secs_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) general_autosave_interval_secs_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) general_show_scrollbars_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) general_inertial_scrolling_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) general_regular_cursor_picker: TemplateChild<RnIconPicker>,
        #[template_child]
        pub(crate) general_regular_cursor_picker_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) general_show_drawing_cursor_switch: TemplateChild<Switch>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker: TemplateChild<RnIconPicker>,
        #[template_child]
        pub(crate) general_drawing_cursor_picker_menubutton: TemplateChild<MenuButton>,
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
        pub(crate) format_width_unitentry: TemplateChild<RnUnitEntry>,
        #[template_child]
        pub(crate) format_height_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_height_unitentry: TemplateChild<RnUnitEntry>,
        #[template_child]
        pub(crate) format_dpi_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) format_dpi_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub(crate) format_revert_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) format_apply_button: TemplateChild<Button>,
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
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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
                .connect_selected_item_notify(clone!(@weak obj as settings_panel => move |_| {
                    settings_panel.imp().apply_predefined_format();
                }));

            self.format_orientation_portrait_toggle.connect_toggled(
                clone!(@weak obj as settings_panel => move |toggle| {
                    if toggle.is_active() && settings_panel.format_orientation() != settings_panel.imp().temporary_format.borrow().orientation() {
                        settings_panel.imp().swap_width_height();
                    }
                }),
            );

            self.format_orientation_landscape_toggle.connect_toggled(
                clone!(@weak obj as settings_panel => move |toggle| {
                    if toggle.is_active() && settings_panel.format_orientation() != settings_panel.imp().temporary_format.borrow().orientation() {
                        settings_panel.imp().swap_width_height();
                    }
                }),
            );

            self.format_width_unitentry.get().connect_notify_local(
                Some("value"),
                clone!(@weak obj as settings_panel => move |entry, _| {
                        settings_panel.imp().temporary_format
                            .borrow_mut()
                            .set_width(entry.value_in_px());
                        settings_panel.imp().update_orientation_toggles();
                }),
            );

            self.format_height_unitentry.get().connect_notify_local(
                Some("value"),
                clone!(@weak obj as settings_panel => move |entry, _| {
                        settings_panel.imp().temporary_format
                            .borrow_mut()
                            .set_height(entry.value_in_px());
                        settings_panel.imp().update_orientation_toggles();
                }),
            );

            self.format_dpi_adj.connect_value_changed(
                clone!(@weak obj as settings_panel => move |adj| {
                    let dpi = adj.value();
                    settings_panel.imp().format_width_unitentry.set_dpi_keep_value(dpi);
                    settings_panel.imp().format_height_unitentry.set_dpi_keep_value(dpi);
                    settings_panel.imp().temporary_format
                        .borrow_mut()
                        .set_dpi(adj.value());
                }),
            );
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
    @extends Widget;
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

    pub(crate) fn general_show_drawing_cursor_switch(&self) -> Switch {
        self.imp().general_show_drawing_cursor_switch.clone()
    }

    pub(crate) fn general_drawing_cursor_picker(&self) -> RnIconPicker {
        self.imp().general_drawing_cursor_picker.clone()
    }

    pub(crate) fn general_show_scrollbars_switch(&self) -> Switch {
        self.imp().general_show_scrollbars_switch.clone()
    }

    pub(crate) fn general_inertial_scrolling_switch(&self) -> Switch {
        self.imp().general_inertial_scrolling_switch.clone()
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnCanvasWrapper) {
        self.refresh_general_ui(active_tab);
        self.refresh_format_ui(active_tab);
        self.refresh_doc_ui(active_tab);
        self.refresh_shortcuts_ui(active_tab);
    }

    fn refresh_general_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();
        let canvas = active_tab.canvas();

        let format_border_color = canvas.engine_ref().document.format.border_color;

        imp.doc_format_border_color_button
            .set_rgba(&gdk::RGBA::from_compose_color(format_border_color));
    }

    fn refresh_format_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();
        let canvas = active_tab.canvas();
        let format = canvas.engine_ref().document.format;
        *self.imp().temporary_format.borrow_mut() = format;

        self.set_format_predefined_format_variant(format::PredefinedFormat::Custom);
        self.set_format_orientation(format.orientation());
        imp.format_dpi_adj.set_value(format.dpi());
        imp.format_width_unitentry.set_dpi(format.dpi());
        imp.format_width_unitentry.set_value_in_px(format.width());
        imp.format_height_unitentry.set_dpi(format.dpi());
        imp.format_height_unitentry.set_value_in_px(format.height());
    }

    fn refresh_doc_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();
        let canvas = active_tab.canvas();
        let background = canvas.engine_ref().document.background;
        let format = canvas.engine_ref().document.format;

        imp.doc_background_color_button
            .set_rgba(&gdk::RGBA::from_compose_color(background.color));
        self.set_background_pattern(background.pattern);
        imp.doc_background_pattern_color_button
            .set_rgba(&gdk::RGBA::from_compose_color(background.pattern_color));
        imp.doc_background_pattern_width_unitentry
            .set_dpi(format.dpi());
        imp.doc_background_pattern_width_unitentry
            .set_value_in_px(background.pattern_size[0]);
        imp.doc_background_pattern_height_unitentry
            .set_dpi(format.dpi());
        imp.doc_background_pattern_height_unitentry
            .set_value_in_px(background.pattern_size[1]);
    }

    fn refresh_shortcuts_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();
        let canvas = active_tab.canvas();
        let current_shortcuts = canvas.engine_ref().penholder.list_current_shortcuts();

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

        // autosave enable switch
        imp.general_autosave_enable_switch
            .bind_property("state", appwindow, "autosave")
            .sync_create()
            .bidirectional()
            .build();

        imp.general_autosave_enable_switch
            .get()
            .bind_property(
                "state",
                &*imp.general_autosave_interval_secs_row,
                "sensitive",
            )
            .sync_create()
            .build();

        imp.general_autosave_interval_secs_spinbutton
            .get()
            .bind_property("value", appwindow, "autosave-interval-secs")
            .transform_to(|_, val: f64| Some((val.round() as u32).to_value()))
            .transform_from(|_, val: u32| Some(f64::from(val).to_value()))
            .sync_create()
            .bidirectional()
            .build();

        let set_overlays_margins = |appwindow: &RnAppWindow, switch_active: bool| {
            let (m1, m2) = if switch_active { (18, 72) } else { (9, 63) };
            appwindow.overlays().colorpicker().set_margin_top(m1);
            appwindow
                .overlays()
                .pens_toggles_box()
                .set_margin_bottom(m1);
            appwindow.overlays().sidebar_box().set_margin_start(m1);
            appwindow.overlays().sidebar_box().set_margin_end(m1);
            appwindow.overlays().sidebar_box().set_margin_top(m2);
            appwindow.overlays().sidebar_box().set_margin_bottom(m2);
        };
        // set on init
        set_overlays_margins(appwindow, imp.general_show_scrollbars_switch.is_active());
        // and on change
        imp.general_show_scrollbars_switch.connect_active_notify(
            clone!(@weak appwindow => move |switch| {
                    set_overlays_margins(&appwindow, switch.is_active());
            }),
        );

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
        imp.general_show_drawing_cursor_switch
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

        imp.general_inertial_scrolling_switch.connect_active_notify(
            clone!(@weak self as settingspanel, @weak appwindow => move |switch| {
                if !switch.is_active() {
                    appwindow.overlays().dispatch_toast_text_singleton(
                        &gettext("Application restart is required"),
                        0,
                        &mut settingspanel.imp().app_restart_toast_singleton.borrow_mut()
                    );
                }
            }),
        );
    }

    fn setup_format(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        // revert format
        imp.format_revert_button.get().connect_clicked(
            clone!(@weak self as settings_panel, @weak appwindow => move |_format_revert_button| {
                settings_panel.revert_format(&appwindow);
            }),
        );

        // Apply format
        imp.format_apply_button.get().connect_clicked(
            clone!(@weak self as settingspanel, @weak appwindow => move |_| {
                settingspanel.apply_format(&appwindow);
            }),
        );
    }

    fn setup_doc(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.doc_format_border_color_button.connect_rgba_notify(clone!(@weak self as settingspanel, @weak appwindow => move |button| {
            let format_border_color = button.rgba().into_compose_color();
            let canvas = appwindow.active_tab_wrapper().canvas();
            canvas.engine_mut().document.format.border_color = format_border_color;
            // Because the format border color is applied immediately to the engine,
            // we need to update the temporary format too.
            settingspanel.imp().temporary_format.borrow_mut().border_color = format_border_color;
            let widget_flags = canvas.engine_mut().update_rendering_current_viewport();
            appwindow.handle_widget_flags(widget_flags, &canvas);
        }));

        imp.doc_background_color_button.connect_rgba_notify(
            clone!(@weak appwindow => move |button| {
                let canvas = appwindow.active_tab_wrapper().canvas();
                canvas.engine_mut().document.background.color = button.rgba().into_compose_color();
                let widget_flags = canvas.engine_mut().background_regenerate_pattern();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        imp.doc_background_patterns_row.get().connect_selected_item_notify(clone!(@weak self as settings_panel, @weak appwindow => move |_| {
            let pattern = settings_panel.background_pattern();
            let canvas = appwindow.active_tab_wrapper().canvas();

            match pattern {
                PatternStyle::None => {
                    settings_panel.imp().doc_background_pattern_width_unitentry.set_sensitive(false);
                    settings_panel.imp().doc_background_pattern_height_unitentry.set_sensitive(false);
                },
                PatternStyle::Lines => {
                    settings_panel.imp().doc_background_pattern_width_unitentry.set_sensitive(false);
                    settings_panel.imp().doc_background_pattern_height_unitentry.set_sensitive(true);
                },
                PatternStyle::Grid => {
                    settings_panel.imp().doc_background_pattern_width_unitentry.set_sensitive(true);
                    settings_panel.imp().doc_background_pattern_height_unitentry.set_sensitive(true);
                },
                PatternStyle::Dots => {
                    settings_panel.imp().doc_background_pattern_width_unitentry.set_sensitive(true);
                    settings_panel.imp().doc_background_pattern_height_unitentry.set_sensitive(true);
                },
                PatternStyle::IsometricGrid => {
                    settings_panel.imp().doc_background_pattern_width_unitentry.set_sensitive(false);
                    settings_panel.imp().doc_background_pattern_height_unitentry.set_sensitive(true);
                },
                PatternStyle::IsometricDots => {
                    settings_panel.imp().doc_background_pattern_width_unitentry.set_sensitive(false);
                    settings_panel.imp().doc_background_pattern_height_unitentry.set_sensitive(true);
                },
            }

            canvas.engine_mut().document.background.pattern = pattern;
            let widget_flags = canvas.engine_mut().background_regenerate_pattern();
            appwindow.handle_widget_flags(widget_flags, &canvas);
        }));

        imp.doc_background_pattern_color_button.connect_rgba_notify(clone!(@weak appwindow => move |button| {
            let canvas = appwindow.active_tab_wrapper().canvas();
            canvas.engine_mut().document.background.pattern_color = button.rgba().into_compose_color();
            let widget_flags = canvas.engine_mut().background_regenerate_pattern();
            appwindow.handle_widget_flags(widget_flags, &canvas);
        }));

        imp.doc_background_pattern_width_unitentry
            .get()
            .connect_notify_local(
                Some("value"),
                clone!(@weak self as settings_panel, @weak appwindow => move |unit_entry, _| {
                        let canvas = appwindow.active_tab_wrapper().canvas();
                        let mut pattern_size = canvas.engine_ref().document.background.pattern_size;
                        pattern_size[0] = unit_entry.value_in_px();
                        canvas.engine_mut().document.background.pattern_size = pattern_size;
                        let widget_flags = canvas.engine_mut().background_regenerate_pattern();
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                }),
            );

        imp.doc_background_pattern_height_unitentry
            .get()
            .connect_notify_local(
                Some("value"),
                clone!(@weak self as settings_panel, @weak appwindow => move |unit_entry, _| {
                        let canvas = appwindow.active_tab_wrapper().canvas();
                        let mut pattern_size = canvas.engine_ref().document.background.pattern_size;
                        pattern_size[1] = unit_entry.value_in_px();
                        canvas.engine_mut().document.background.pattern_size = pattern_size;
                        let widget_flags = canvas.engine_mut().background_regenerate_pattern();
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                }),
            );

        imp.background_pattern_invert_color_button.get().connect_clicked(
                clone!(@weak self as settings_panel, @weak appwindow => move |_| {
                    let canvas = appwindow.active_tab_wrapper().canvas();

                    let mut widget_flags = {
                        let mut engine = canvas.engine_mut();

                        engine.document.background.color = engine.document.background.color.to_inverted_brightness_color();
                        engine.document.background.pattern_color = engine.document.background.pattern_color.to_inverted_brightness_color();
                        engine.document.format.border_color = engine.document.format.border_color.to_inverted_brightness_color();

                        engine.background_regenerate_pattern()
                    };

                    widget_flags.refresh_ui = true;
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }),
            );
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

        imp.penshortcut_stylus_button_primary_row.connect_local("action-changed", false, clone!(@weak penshortcut_stylus_button_primary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_stylus_button_primary_row.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::StylusPrimaryButton, action);
            None
        }));

        imp.penshortcut_stylus_button_secondary_row.connect_local("action-changed", false, clone!(@weak penshortcut_stylus_button_secondary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_stylus_button_secondary_row.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::StylusSecondaryButton, action);
            None
        }));

        imp.penshortcut_mouse_button_secondary_row.connect_local("action-changed", false, clone!(@weak penshortcut_mouse_button_secondary_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_mouse_button_secondary_row.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::MouseSecondaryButton, action);
            None
        }));

        imp.penshortcut_touch_two_finger_long_press_row.connect_local("action-changed", false, clone!(@weak penshortcut_touch_two_finger_long_press_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_touch_two_finger_long_press_row.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::TouchTwoFingerLongPress, action);
            None
        }));

        imp.penshortcut_keyboard_ctrl_space_row.connect_local("action-changed", false, clone!(@weak penshortcut_keyboard_ctrl_space_row, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_keyboard_ctrl_space_row.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::KeyboardCtrlSpace, action);
            None
        }));

        imp.penshortcut_drawing_pad_button_0.connect_local("action-changed", false, clone!(@weak penshortcut_drawing_pad_button_0, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_drawing_pad_button_0.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::DrawingPadButton0, action);
            None
        }));

        imp.penshortcut_drawing_pad_button_1.connect_local("action-changed", false, clone!(@weak penshortcut_drawing_pad_button_1, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_drawing_pad_button_1.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::DrawingPadButton1, action);
            None
        }));

        imp.penshortcut_drawing_pad_button_2.connect_local("action-changed", false, clone!(@weak penshortcut_drawing_pad_button_2, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_drawing_pad_button_2.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::DrawingPadButton2, action);
            None
        }));

        imp.penshortcut_drawing_pad_button_3.connect_local("action-changed", false, clone!(@weak penshortcut_drawing_pad_button_3, @weak appwindow => @default-return None, move |_values| {
            let action = penshortcut_drawing_pad_button_3.action();
            appwindow.active_tab_wrapper().canvas().engine_mut().penholder.register_shortcut(ShortcutKey::DrawingPadButton3, action);
            None
        }));
    }

    fn revert_format(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let canvas = appwindow.active_tab_wrapper().canvas();
        *imp.temporary_format.borrow_mut() = canvas.engine_ref().document.format;
        let revert_format = canvas.engine_ref().document.format;

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
        let canvas = appwindow.active_tab_wrapper().canvas();
        let temporary_format = *imp.temporary_format.borrow();

        imp.doc_background_pattern_width_unitentry
            .set_dpi_keep_value(temporary_format.dpi());
        imp.doc_background_pattern_height_unitentry
            .set_dpi_keep_value(temporary_format.dpi());

        canvas.engine_mut().document.format = temporary_format;
        let widget_flags = canvas.engine_mut().doc_resize_to_fit_content();
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
