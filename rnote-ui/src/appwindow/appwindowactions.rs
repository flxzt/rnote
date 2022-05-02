use super::RnoteAppWindow;
use crate::config;
use crate::{
    app::RnoteApp,
    {dialogs, RnoteCanvas},
};
use rnote_compose::builders::ShapeBuilderType;
use rnote_engine::engine::ExpandMode;
use rnote_engine::pens::brush::BrushStyle;
use rnote_engine::pens::eraser::EraserStyle;
use rnote_engine::pens::penholder::{PenHolderEvent, PenStyle};
use rnote_engine::pens::selector::SelectorType;
use rnote_engine::pens::shaper::ShaperStyle;
use rnote_engine::pens::tools::ToolsStyle;
use rnote_engine::pens::{brush, selector, shaper, tools};
use rnote_engine::{render, Camera};

use gettextrs::gettext;
use gtk4::PrintStatus;
use gtk4::{
    gdk, gio, glib, glib::clone, prelude::*, Align, ArrowType, CornerType, PackType, PositionType,
    PrintOperation, PrintOperationAction, Unit,
};

impl RnoteAppWindow {
    /// Boolean actions have no target, and a boolean state. They have a default implementation for the activate signal, which requests the state to be inverted, and the default implementation for change_state, which sets the state to the request.
    /// We generally want to connect to the change_state signal. (but then have to set the state with action.set_state() )
    /// We can then either toggle the state through activating the action, or set the state explicitely through action.change_state(<request>)
    pub fn setup_actions(&self) {
        let action_close_active = gio::SimpleAction::new("close-active", None);
        self.add_action(&action_close_active);
        let action_fullscreen = gio::PropertyAction::new("fullscreen", self, "fullscreened");
        self.add_action(&action_fullscreen);
        let action_about = gio::SimpleAction::new("about", None);
        self.add_action(&action_about);
        let action_keyboard_shortcuts_dialog = gio::SimpleAction::new("keyboard-shortcuts", None);
        self.add_action(&action_keyboard_shortcuts_dialog);
        let action_open_canvasmenu = gio::SimpleAction::new("open-canvasmenu", None);
        self.add_action(&action_open_canvasmenu);
        let action_open_appmenu = gio::SimpleAction::new("open-appmenu", None);
        self.add_action(&action_open_appmenu);
        let action_text_toast =
            gio::SimpleAction::new("text-toast", Some(&glib::VariantType::new("s").unwrap()));
        self.add_action(&action_text_toast);
        let action_error_toast =
            gio::SimpleAction::new("error-toast", Some(&glib::VariantType::new("s").unwrap()));
        self.add_action(&action_error_toast);
        let action_devel_mode =
            gio::SimpleAction::new_stateful("devel-mode", None, &false.to_variant());
        self.add_action(&action_devel_mode);
        let action_devel_settings = gio::SimpleAction::new("devel-settings", None);
        self.add_action(&action_devel_settings);
        let action_visual_debug =
            gio::SimpleAction::new_stateful("visual-debug", None, &false.to_variant());
        self.add_action(&action_visual_debug);
        let action_debug_export_engine_state =
            gio::SimpleAction::new("debug-export-engine-state", None);
        self.add_action(&action_debug_export_engine_state);
        let action_righthanded = gio::PropertyAction::new("righthanded", self, "righthanded");
        self.add_action(&action_righthanded);
        let action_touch_drawing =
            gio::PropertyAction::new("touch-drawing", &self.canvas(), "touch-drawing");
        self.add_action(&action_touch_drawing);

        // Engine actions
        let action_pen_sounds =
            gio::SimpleAction::new_stateful("pen-sounds", None, &false.to_variant());
        self.add_action(&action_pen_sounds);
        let action_format_borders =
            gio::SimpleAction::new_stateful("format-borders", None, &true.to_variant());
        self.add_action(&action_format_borders);
        let action_expand_mode = gio::SimpleAction::new_stateful(
            "expand-mode",
            Some(&glib::VariantType::new("s").unwrap()),
            &String::from("infinite").to_variant(),
        );
        self.add_action(&action_expand_mode);
        let action_undo_stroke = gio::SimpleAction::new("undo", None);
        self.add_action(&action_undo_stroke);
        let action_redo_stroke = gio::SimpleAction::new("redo", None);
        self.add_action(&action_redo_stroke);
        let action_zoom_reset = gio::SimpleAction::new("zoom-reset", None);
        self.add_action(&action_zoom_reset);
        let action_zoom_fit_width = gio::SimpleAction::new("zoom-fit-width", None);
        self.add_action(&action_zoom_fit_width);
        let action_zoomin = gio::SimpleAction::new("zoom-in", None);
        self.add_action(&action_zoomin);
        let action_zoomout = gio::SimpleAction::new("zoom-out", None);
        self.add_action(&action_zoomout);
        let action_zoom_to_value =
            gio::SimpleAction::new("zoom-to-value", Some(&glib::VariantType::new("d").unwrap()));
        self.add_action(&action_zoom_to_value);
        let action_add_page_to_sheet = gio::SimpleAction::new("add-page-to-sheet", None);
        self.add_action(&action_add_page_to_sheet);
        let action_resize_to_fit_strokes = gio::SimpleAction::new("resize-to-fit-strokes", None);
        self.add_action(&action_resize_to_fit_strokes);
        let action_return_origin_page = gio::SimpleAction::new("return-origin-page", None);
        self.add_action(&action_return_origin_page);

        let action_selection_trash = gio::SimpleAction::new("selection-trash", None);
        self.add_action(&action_selection_trash);
        let action_selection_duplicate = gio::SimpleAction::new("selection-duplicate", None);
        self.add_action(&action_selection_duplicate);
        let action_selection_select_all = gio::SimpleAction::new("selection-select-all", None);
        self.add_action(&action_selection_select_all);
        let action_selection_deselect_all = gio::SimpleAction::new("selection-deselect-all", None);
        self.add_action(&action_selection_deselect_all);
        let action_clear_sheet = gio::SimpleAction::new("clear-sheet", None);
        self.add_action(&action_clear_sheet);
        let action_new_sheet = gio::SimpleAction::new("new-sheet", None);
        self.add_action(&action_new_sheet);
        let action_save_sheet = gio::SimpleAction::new("save-sheet", None);
        self.add_action(&action_save_sheet);
        let action_save_sheet_as = gio::SimpleAction::new("save-sheet-as", None);
        self.add_action(&action_save_sheet_as);
        let action_autosave = gio::PropertyAction::new("autosave", self, "autosave");
        self.add_action(&action_autosave);
        let action_open_sheet = gio::SimpleAction::new("open-sheet", None);
        self.add_action(&action_open_sheet);
        let action_open_workspace = gio::SimpleAction::new("open-workspace", None);
        self.add_action(&action_open_workspace);
        let action_print_sheet = gio::SimpleAction::new("print-sheet", None);
        self.add_action(&action_print_sheet);
        let action_import_file = gio::SimpleAction::new("import-file", None);
        self.add_action(&action_import_file);
        let action_export_selection_as_svg =
            gio::SimpleAction::new("export-selection-as-svg", None);
        self.add_action(&action_export_selection_as_svg);
        let action_export_sheet_as_svg = gio::SimpleAction::new("export-sheet-as-svg", None);
        self.add_action(&action_export_sheet_as_svg);
        let action_export_sheet_as_pdf = gio::SimpleAction::new("export-sheet-as-pdf", None);
        self.add_action(&action_export_sheet_as_pdf);
        let action_export_sheet_as_xopp = gio::SimpleAction::new("export-sheet-as-xopp", None);
        self.add_action(&action_export_sheet_as_xopp);
        let action_clipboard_copy_selection =
            gio::SimpleAction::new("clipboard-copy-selection", None);
        self.add_action(&action_clipboard_copy_selection);
        let action_clipboard_paste_selection =
            gio::SimpleAction::new("clipboard-paste-selection", None);
        self.add_action(&action_clipboard_paste_selection);
        let action_pen_override = gio::SimpleAction::new(
            "pen-style-override",
            Some(&glib::VariantType::new("s").unwrap()),
        );
        self.add_action(&action_pen_override);
        let action_pen_style =
            gio::SimpleAction::new("pen-style", Some(&glib::VariantType::new("s").unwrap()));
        self.add_action(&action_pen_style);
        let action_brush_style =
            gio::SimpleAction::new("brush-style", Some(&glib::VariantType::new("s").unwrap()));
        self.add_action(&action_brush_style);
        let action_shape_buildertype = gio::SimpleAction::new(
            "shape-buildertype",
            Some(&glib::VariantType::new("s").unwrap()),
        );
        self.add_action(&action_shape_buildertype);
        let action_shaper_style =
            gio::SimpleAction::new("shaper-style", Some(&glib::VariantType::new("s").unwrap()));
        self.add_action(&action_shaper_style);
        let action_eraser_style =
            gio::SimpleAction::new("eraser-style", Some(&glib::VariantType::new("s").unwrap()));
        self.add_action(&action_eraser_style);
        let action_selector_style = gio::SimpleAction::new(
            "selector-style",
            Some(&glib::VariantType::new("s").unwrap()),
        );
        self.add_action(&action_selector_style);
        let action_tool_style =
            gio::SimpleAction::new("tool-style", Some(&glib::VariantType::new("s").unwrap()));
        self.add_action(&action_tool_style);
        let action_refresh_ui_for_engine = gio::SimpleAction::new("refresh-ui-for-engine", None);
        self.add_action(&action_refresh_ui_for_engine);

        // Close active window
        action_close_active.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            appwindow.close();
        }));

        // About Dialog
        action_about.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_about(&appwindow);
        }));

        // Keyboard shortcuts
        action_keyboard_shortcuts_dialog.connect_activate(
            clone!(@weak self as appwindow => move |_action_keyboard_shortcuts_dialog, _parameter| {
                dialogs::dialog_keyboard_shortcuts(&appwindow);
            }),
        );

        // Open Canvas Menu
        action_open_canvasmenu.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            appwindow.mainheader().canvasmenu().popovermenu().popup();
        }));

        // Open App Menu
        action_open_appmenu.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            appwindow.mainheader().appmenu().popovermenu().popup();
        }));

        // Notify with a text toast
        action_text_toast.connect_activate(
            clone!(@weak self as appwindow => move |_action_text_toast, parameter| {
                let text = parameter.unwrap().get::<String>().unwrap();
                let text_notify_toast = adw::Toast::builder().title(text.as_str()).priority(adw::ToastPriority::High).timeout(5).build();

                appwindow.toast_overlay().add_toast(&text_notify_toast);
            }),
        );

        // Error toast
        action_error_toast.connect_activate(
            clone!(@weak self as appwindow => move |_action_error_toast, parameter| {
                let error_text = parameter.unwrap().get::<String>().unwrap();
                log::error!("{}", error_text);
                let error_toast = adw::Toast::builder().title(error_text.as_str()).priority(adw::ToastPriority::High).timeout(0).build();

                appwindow.toast_overlay().add_toast(&error_toast);
            }),
        );

        // Developer mode
        action_devel_mode.connect_activate(
            clone!(@weak self as appwindow, @weak action_devel_settings => move |action_devel_mode, _target| {
                let state = action_devel_mode.state().unwrap().get::<bool>().unwrap();

                // Enable the devel settings action to reveal the settings in the menu
                action_devel_settings.set_enabled(!state);

                // If toggled to disable
                if state {
                    log::debug!("disabling visual debugging");
                    appwindow.lookup_action("visual-debug").unwrap().change_state(&false.to_variant());
                }
                action_devel_mode.change_state(&(!state).to_variant());
            }),
        );

        // Developer settings
        // Its enabled state toggles the visibility of the developer setttings menu entry. Is only modified inside action_devel_mode
        action_devel_settings.set_enabled(false);

        // Visual debugging
        action_visual_debug.connect_change_state(
            clone!(@weak self as appwindow => move |action_visual_debug, state_request| {
                let requested_state = state_request.unwrap().get::<bool>().unwrap();


                appwindow.canvas().engine().borrow_mut().visual_debug = requested_state;
                appwindow.canvas().queue_draw();
                action_visual_debug.set_state(&requested_state.to_variant());
            }),
        );

        // Export engine state
        action_debug_export_engine_state.connect_activate(
            clone!(@weak self as appwindow => move |_action_debug_export_engine_state, _target| {
                dialogs::dialog_export_engine_state(&appwindow);
            }),
        );

        // Expand Mode
        action_expand_mode.connect_activate(
            clone!(@weak self as appwindow => move |action_expand_mode, target| {
                let expand_mode = target.unwrap().str().unwrap();

                match expand_mode {
                    "fixed-size" => {
                        appwindow.canvas().engine().borrow_mut().set_expand_mode(ExpandMode::FixedSize);
                        appwindow.canvas_fixedsize_quickactions_revealer().set_reveal_child(true);
                    },
                    "endless-vertical" => {
                        appwindow.canvas().engine().borrow_mut().set_expand_mode(ExpandMode::EndlessVertical);
                        appwindow.canvas_fixedsize_quickactions_revealer().set_reveal_child(false);
                    },
                    "infinite" => {
                        appwindow.canvas().engine().borrow_mut().set_expand_mode(ExpandMode::Infinite);
                        appwindow.canvas_fixedsize_quickactions_revealer().set_reveal_child(false);
                    }
                    invalid_str => {
                        log::error!("action expand mode failed, invalid str: {}", invalid_str);
                        return;
                    }
                }
                appwindow.canvas().update_engine_rendering();

                action_expand_mode.set_state(&expand_mode.to_variant());
            }));

        // Righthanded
        action_righthanded.connect_state_notify(
            clone!(@weak self as appwindow => move |action_righthanded| {
                let current_state = action_righthanded.state().unwrap().get::<bool>().unwrap();

                if current_state {
                    appwindow.main_grid().remove(&appwindow.sidebar_grid());
                    appwindow.main_grid().remove(&appwindow.sidebar_sep());
                    appwindow.main_grid().remove(&appwindow.narrow_pens_toggles_revealer());
                    appwindow.main_grid().remove(&appwindow.canvas_box());
                    appwindow
                        .main_grid()
                        .attach(&appwindow.sidebar_grid(), 0, 1, 1, 2);
                    appwindow
                        .main_grid()
                        .attach(&appwindow.sidebar_sep(), 1, 1, 1, 2);
                    appwindow
                        .main_grid()
                        .attach(&appwindow.narrow_pens_toggles_revealer(), 2, 1, 1, 1);
                    appwindow
                        .main_grid()
                        .attach(&appwindow.canvas_box(), 2, 2, 1, 1);
                    appwindow.canvas_quickactions_box().set_halign(Align::End);

                    appwindow
                        .mainheader()
                        .appmenu()
                        .righthanded_toggle()
                        .set_active(true);
                    appwindow
                        .mainheader()
                        .headerbar()
                        .remove(&appwindow.mainheader().pens_toggles_squeezer());
                    appwindow
                        .mainheader()
                        .headerbar()
                        .pack_start(&appwindow.mainheader().pens_toggles_squeezer());

                    appwindow
                        .canvas_scroller()
                        .set_window_placement(CornerType::BottomLeft);
                    appwindow
                        .sidebar_scroller()
                        .set_window_placement(CornerType::TopRight);

                    appwindow
                        .settings_panel()
                        .settings_scroller()
                        .set_window_placement(CornerType::TopRight);
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .colorpicker()
                        .set_property("position", PositionType::Left.to_value());
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .stroke_colorpicker()
                        .set_property("position", PositionType::Left.to_value());
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .fill_colorpicker()
                        .set_property("position", PositionType::Left.to_value());
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .brushconfig_menubutton()
                        .set_direction(ArrowType::Right);
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapeconfig_menubutton()
                        .set_direction(ArrowType::Right);
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .brushstyle_menubutton()
                        .set_direction(ArrowType::Right);
                    appwindow.flap().set_flap_position(PackType::Start);
                } else {
                    appwindow.main_grid().remove(&appwindow.canvas_box());
                    appwindow.main_grid().remove(&appwindow.narrow_pens_toggles_revealer());
                    appwindow.main_grid().remove(&appwindow.sidebar_sep());
                    appwindow.main_grid().remove(&appwindow.sidebar_grid());
                    appwindow
                        .main_grid()
                        .attach(&appwindow.canvas_box(), 0, 2, 1, 1);
                    appwindow
                        .main_grid()
                        .attach(&appwindow.narrow_pens_toggles_revealer(), 0, 1, 1, 1);
                    appwindow
                        .main_grid()
                        .attach(&appwindow.sidebar_sep(), 1, 1, 1, 2);
                    appwindow
                        .main_grid()
                        .attach(&appwindow.sidebar_grid(), 2, 1, 1, 2);
                    appwindow.canvas_quickactions_box().set_halign(Align::Start);

                    appwindow
                        .mainheader()
                        .headerbar()
                        .remove(&appwindow.mainheader().pens_toggles_squeezer());
                    appwindow
                        .mainheader()
                        .appmenu()
                        .lefthanded_toggle()
                        .set_active(true);
                    appwindow
                        .mainheader()
                        .headerbar()
                        .pack_end(&appwindow.mainheader().pens_toggles_squeezer());

                    appwindow
                        .canvas_scroller()
                        .set_window_placement(CornerType::BottomRight);
                    appwindow
                        .sidebar_scroller()
                        .set_window_placement(CornerType::TopLeft);

                    appwindow
                        .settings_panel()
                        .settings_scroller()
                        .set_window_placement(CornerType::TopLeft);
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .colorpicker()
                        .set_property("position", PositionType::Right.to_value());
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .stroke_colorpicker()
                        .set_property("position", PositionType::Right.to_value());
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .fill_colorpicker()
                        .set_property("position", PositionType::Right.to_value());
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .brushconfig_menubutton()
                        .set_direction(ArrowType::Left);
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapeconfig_menubutton()
                        .set_direction(ArrowType::Left);
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .brushstyle_menubutton()
                        .set_direction(ArrowType::Left);
                    appwindow.flap().set_flap_position(PackType::End);
                }
            }),
        );

        // Pen sounds
        action_pen_sounds.connect_change_state(
            clone!(@weak self as appwindow => move |action_pen_sounds, state_request| {
                let pen_sounds = state_request.unwrap().get::<bool>().unwrap();

                appwindow.canvas().engine().borrow_mut().penholder.set_pen_sounds(pen_sounds);

                action_pen_sounds.set_state(&pen_sounds.to_variant());
            }),
        );

        // Format borders
        action_format_borders.connect_change_state(
            clone!(@weak self as appwindow => move |action_format_borders, state_request| {
                let format_borders = state_request.unwrap().get::<bool>().unwrap();

                appwindow.canvas().engine().borrow_mut().sheet.format.show_borders = format_borders;
                appwindow.canvas().queue_draw();

                action_format_borders.set_state(&format_borders.to_variant());
            }),
        );

        // Pen style
        action_pen_style.connect_activate(
            clone!(@weak self as appwindow => move |_action_pen_style, target| {
                let pen_style = target.unwrap().str().unwrap();

                let new_pen_style = match pen_style {
                    "brush" => {
                        Some(PenStyle::Brush)
                    }
                    "shaper" => {
                        Some(PenStyle::Shaper)
                    }
                    "eraser" => {
                        Some(PenStyle::Eraser)
                    }
                    "selector" => {
                        Some(PenStyle::Selector)
                    }
                    "tools" => {
                        Some(PenStyle::Tools)
                    }
                    _ => {
                        log::error!("invalid target for action_pen_style, `{}`", pen_style);
                        None
                    }
                };

                if let Some(new_pen_style) = new_pen_style {
                    // don't change the style if the current style with override is already the same (e.g. when switched to from the pen button, not by clicking the pen page)
                    if new_pen_style != appwindow.canvas().engine().borrow().penholder.style_w_override() {
                        let mut surface_flags = appwindow.canvas().engine().borrow_mut().handle_penholder_event(
                            PenHolderEvent::ChangeStyle(new_pen_style),
                        );
                        surface_flags = surface_flags.merged_with_other(appwindow.canvas().engine().borrow_mut().handle_penholder_event(
                            PenHolderEvent::ChangeStyleOverride(None),
                        ));

                        appwindow.handle_surface_flags(surface_flags);
                    }
                }
            }),
        );

        // Pen override
        action_pen_override.connect_activate(
            clone!(@weak self as appwindow => move |_action_pen_override, target| {
                let pen_style_override = target.unwrap().str().unwrap();
                log::trace!("pen overwrite activated with target: {}", pen_style_override);

                let change_pen_style_override_event = match pen_style_override {
                    "brush" => {
                        Some(PenHolderEvent::ChangeStyleOverride(Some(PenStyle::Brush)))
                    }
                    "shaper" => {
                        Some(PenHolderEvent::ChangeStyleOverride(Some(PenStyle::Shaper)))
                    }
                    "eraser" => {
                        Some(PenHolderEvent::ChangeStyleOverride(Some(PenStyle::Eraser)))
                    }
                    "selector" => {
                        Some(PenHolderEvent::ChangeStyleOverride(Some(PenStyle::Selector)))
                    }
                    "tools" => {
                        Some(PenHolderEvent::ChangeStyleOverride(Some(PenStyle::Tools)))
                    }
                    "none" => {
                        Some(PenHolderEvent::ChangeStyleOverride(None))
                    }
                    _ => {
                        log::error!("invalid target for action_pen_overwrite, `{}`", pen_style_override);
                        None
                    }
                };

                if let Some(change_pen_style_override_event) = change_pen_style_override_event {
                    let surface_flags = appwindow.canvas().engine().borrow_mut().handle_penholder_event(
                        change_pen_style_override_event,
                    );
                    appwindow.handle_surface_flags(surface_flags);
                }
            }),
        );

        // Brush Style
        action_brush_style.connect_activate(
        clone!(@weak self as appwindow => move |_action_brush_style, target| {
            let brush_style = target.unwrap().str().unwrap();

            match brush_style {
                "marker" => {
                    appwindow.canvas().engine().borrow_mut().penholder.brush.style = brush::BrushStyle::Marker;
                    appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_width = appwindow.penssidebar().brush_page().width_spinbutton().value();
                    appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_color = Some(appwindow.penssidebar().brush_page().colorpicker().current_color());
                },
                "solid" => {
                    appwindow.canvas().engine().borrow_mut().penholder.brush.style = brush::BrushStyle::Solid;
                    appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_width = appwindow.penssidebar().brush_page().width_spinbutton().value();
                    appwindow.canvas().engine().borrow_mut().penholder.brush.smooth_options.stroke_color = Some(appwindow.penssidebar().brush_page().colorpicker().current_color());
                },
                "textured" => {
                    appwindow.canvas().engine().borrow_mut().penholder.brush.style = brush::BrushStyle::Textured;
                    appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.stroke_width = appwindow.penssidebar().brush_page().width_spinbutton().value();
                    appwindow.canvas().engine().borrow_mut().penholder.brush.textured_options.stroke_color = Some(appwindow.penssidebar().brush_page().colorpicker().current_color());
                },
                _ => { log::error!("set invalid state of action `brush-style`")}
            }


            adw::prelude::ActionGroupExt::activate_action(&appwindow, "refresh-ui-for-engine", None);
        }),
        );

        // Shape type
        action_shape_buildertype.connect_activate(
        clone!(@weak self as appwindow => move |_action_shaper_type, target| {
            let shape_type = target.unwrap().str().unwrap();

            match shape_type {
                "line" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::Line;
                },
                "rectangle" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::Rectangle;
                },
                "ellipse" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::Ellipse;
                },
                "fociellipse" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::FociEllipse;
                },
                "quadbez" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::QuadBez;
                },
                "cubbez" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::CubBez;
                },
                _ => { log::error!("set invalid state of action `shape-buildertype`")}
            }


            adw::prelude::ActionGroupExt::activate_action(&appwindow, "refresh-ui-for-engine", None);
        }),
        );

        // Shaper style
        action_shaper_style.connect_activate(
        clone!(@weak self as appwindow => move |_action_shaper_style, target| {
            let shaper_style = target.unwrap().str().unwrap();

            match shaper_style {
                "smooth" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.style = shaper::ShaperStyle::Smooth;
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.stroke_width = appwindow.penssidebar().shaper_page().width_spinbutton().value();
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.stroke_color = Some(appwindow.penssidebar().shaper_page().stroke_colorpicker().current_color());
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.fill_color = Some(appwindow.penssidebar().shaper_page().fill_colorpicker().current_color());
                },
                "rough" => {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.style = shaper::ShaperStyle::Rough;
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.stroke_width = appwindow.penssidebar().shaper_page().width_spinbutton().value();
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.stroke_color = Some(appwindow.penssidebar().shaper_page().stroke_colorpicker().current_color());
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.fill_color = Some(appwindow.penssidebar().shaper_page().fill_colorpicker().current_color());
                },
                _ => { log::error!("set invalid state of action `shaper-style`")}
            }

            adw::prelude::ActionGroupExt::activate_action(&appwindow, "refresh-ui-for-engine", None);
        }));

        // Eraser Style
        action_eraser_style.connect_activate(
        clone!(@weak self as appwindow => move |_action_eraser_style, target| {
            let eraser_style = target.unwrap().str().unwrap();

            match eraser_style {
                "trash-colliding-strokes" => {
                    appwindow.canvas().engine().borrow_mut().penholder.eraser.style = EraserStyle::TrashCollidingStrokes;
                },
                "split-colliding-strokes" => {
                    appwindow.canvas().engine().borrow_mut().penholder.eraser.style = EraserStyle::SplitCollidingStrokes;
                },
                _ => { log::error!("set invalid state of action `eraser-style`")}
            }

            adw::prelude::ActionGroupExt::activate_action(&appwindow, "refresh-ui-for-engine", None);
        }),
        );

        // Selector Style
        action_selector_style.connect_activate(
        clone!(@weak self as appwindow => move |_action_selector_style, target| {
            let selector_style = target.unwrap().str().unwrap();

            match selector_style {
                "polygon" => {
                    appwindow.canvas().engine().borrow_mut().penholder.selector.style = selector::SelectorType::Polygon;
                },
                "rectangle" => {
                    appwindow.canvas().engine().borrow_mut().penholder.selector.style = selector::SelectorType::Rectangle;
                },
                _ => { log::error!("set invalid state of action `selector-style`")}
            }

            adw::prelude::ActionGroupExt::activate_action(&appwindow, "refresh-ui-for-engine", None);
        }),
        );

        // Tool Style
        action_tool_style.connect_activate(
        clone!(@weak self as appwindow => move |_action_tool_style, target| {
            let tool_style = target.unwrap().str().unwrap();

            match tool_style {
                "expandsheet" => {
                    appwindow.canvas().engine().borrow_mut().penholder.tools.style = tools::ToolsStyle::ExpandSheet;
                },
                "dragproximity" => {
                    appwindow.canvas().engine().borrow_mut().penholder.tools.style = tools::ToolsStyle::DragProximity;
                },
                "offsetcamera" => {
                    appwindow.canvas().engine().borrow_mut().penholder.tools.style = tools::ToolsStyle::OffsetCamera;
                },
                _ => { log::error!("set invalid state of action `tool-style`")}
            }

            adw::prelude::ActionGroupExt::activate_action(&appwindow, "refresh-ui-for-engine", None);
        }),
        );

        // Refresh UI state
        action_refresh_ui_for_engine.connect_activate(
            clone!(@weak self as appwindow, @strong action_pen_sounds, @strong action_expand_mode, @strong action_format_borders => move |_action_refresh_ui_for_sheet, _| {
                // Avoids borrow errors
                let format = appwindow.canvas().engine().borrow().sheet.format.clone();
                let expand_mode = appwindow.canvas().engine().borrow().expand_mode();
                let pen_sounds = appwindow.canvas().engine().borrow().penholder.pen_sounds();
                let pen_style = appwindow.canvas().engine().borrow().penholder.style_w_override();
                let brush = appwindow.canvas().engine().borrow().penholder.brush.clone();
                let eraser = appwindow.canvas().engine().borrow().penholder.eraser.clone();
                let selector = appwindow.canvas().engine().borrow().penholder.selector.clone();
                let tools = appwindow.canvas().engine().borrow().penholder.tools.clone();

                {
                    // Engine
                    let expand_mode_str = match expand_mode {
                        ExpandMode::FixedSize => "fixed-size",
                        ExpandMode::EndlessVertical => "endless-vertical",
                        ExpandMode::Infinite => "infinite",
                    };
                    action_expand_mode.activate(Some(&expand_mode_str.to_variant()));

                    action_pen_sounds.change_state(&pen_sounds.to_variant());

                    action_format_borders.change_state(&format.show_borders.to_variant());
                }

                // Current pen
                match pen_style {
                    PenStyle::Brush => {
                        appwindow.mainheader().brush_toggle().set_active(true);
                        appwindow.narrow_brush_toggle().set_active(true);
                        appwindow.penssidebar().sidebar_stack().set_visible_child_name("brush_page");
                    }
                    PenStyle::Shaper => {
                        appwindow.mainheader().shaper_toggle().set_active(true);
                        appwindow.narrow_shaper_toggle().set_active(true);
                        appwindow.penssidebar().sidebar_stack().set_visible_child_name("shaper_page");
                    }
                    PenStyle::Eraser => {
                        appwindow.mainheader().eraser_toggle().set_active(true);
                        appwindow.narrow_eraser_toggle().set_active(true);
                        appwindow.penssidebar().sidebar_stack().set_visible_child_name("eraser_page");
                    }
                    PenStyle::Selector => {
                        appwindow.mainheader().selector_toggle().set_active(true);
                        appwindow.narrow_selector_toggle().set_active(true);
                        appwindow.penssidebar().sidebar_stack().set_visible_child_name("selector_page");
                    }
                    PenStyle::Tools => {
                        appwindow.mainheader().tools_toggle().set_active(true);
                        appwindow.narrow_tools_toggle().set_active(true);
                        appwindow.penssidebar().sidebar_stack().set_visible_child_name("tools_page");
                    }
                }

                // Brush
                appwindow.penssidebar().brush_page().texturedstyle_density_spinbutton()
                    .set_value(brush.textured_options.density);
                appwindow.penssidebar().brush_page().texturedstyle_radius_x_spinbutton()
                    .set_value(brush.textured_options.radii[0]);
                appwindow.penssidebar().brush_page().texturedstyle_radius_y_spinbutton()
                    .set_value(brush.textured_options.radii[1]);
                appwindow.penssidebar().brush_page().set_texturedstyle_distribution_variant(brush.textured_options.distribution);
                match brush.style {
                    BrushStyle::Marker => {
                        appwindow.penssidebar().brush_page().brushstyle_listbox().select_row(Some(&appwindow.penssidebar().brush_page().brushstyle_marker_row()));
                        appwindow.penssidebar().brush_page().width_spinbutton().set_value(brush.smooth_options.stroke_width);
                        appwindow.penssidebar().brush_page().colorpicker().set_current_color(brush.smooth_options.stroke_color);
                        appwindow.penssidebar().brush_page().brushstyle_image().set_icon_name(Some("pen-brush-style-marker-symbolic"));
                    },
                    BrushStyle::Solid => {
                        appwindow.penssidebar().brush_page().brushstyle_listbox().select_row(Some(&appwindow.penssidebar().brush_page().brushstyle_solid_row()));
                        appwindow.penssidebar().brush_page().width_spinbutton().set_value(brush.smooth_options.stroke_width);
                        appwindow.penssidebar().brush_page().colorpicker().set_current_color(brush.smooth_options.stroke_color);
                        appwindow.penssidebar().brush_page().brushstyle_image().set_icon_name(Some("pen-brush-style-solid-symbolic"));
                    },
                    BrushStyle::Textured => {
                        appwindow.penssidebar().brush_page().brushstyle_listbox().select_row(Some(&appwindow.penssidebar().brush_page().brushstyle_textured_row()));
                        appwindow.penssidebar().brush_page().width_spinbutton().set_value(brush.textured_options.stroke_width);
                        appwindow.penssidebar().brush_page().colorpicker().set_current_color(brush.textured_options.stroke_color);
                        appwindow.penssidebar().brush_page().brushstyle_image().set_icon_name(Some("pen-brush-style-textured-symbolic"));
                    },
                }

                // Shaper
                {
                    let builder_type = appwindow.canvas().engine().borrow().penholder.shaper.builder_type.clone();
                    let style = appwindow.canvas().engine().borrow().penholder.shaper.style.clone();
                    let rough_options = appwindow.canvas().engine().borrow().penholder.shaper.rough_options.clone();
                    let smooth_options = appwindow.canvas().engine().borrow().penholder.shaper.smooth_options.clone();

                    appwindow.penssidebar().shaper_page()
                        .roughconfig_roughness_spinbutton()
                        .set_value(rough_options.roughness);
                    appwindow.penssidebar().shaper_page()
                        .roughconfig_bowing_spinbutton()
                        .set_value(rough_options.bowing);
                    appwindow.penssidebar().shaper_page()
                        .roughconfig_curvestepcount_spinbutton()
                        .set_value(rough_options.curve_stepcount);
                    appwindow.penssidebar().shaper_page()
                        .roughconfig_multistroke_switch()
                        .set_active(!rough_options.disable_multistroke);

                    match builder_type {
                        ShapeBuilderType::Line => {
                            appwindow.penssidebar().shaper_page().shapebuildertype_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shapebuildertype_line_row()));
                            appwindow.penssidebar().shaper_page().shapebuildertype_image().set_icon_name(Some("shape-line-symbolic"));
                        }
                        ShapeBuilderType::Rectangle => {
                            appwindow.penssidebar().shaper_page().shapebuildertype_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shapebuildertype_rectangle_row()));
                            appwindow.penssidebar().shaper_page().shapebuildertype_image().set_icon_name(Some("shape-rectangle-symbolic"));
                        }
                        ShapeBuilderType::Ellipse => {
                            appwindow.penssidebar().shaper_page().shapebuildertype_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shapebuildertype_ellipse_row()));
                            appwindow.penssidebar().shaper_page().shapebuildertype_image().set_icon_name(Some("shape-ellipse-symbolic"));
                        }
                        ShapeBuilderType::FociEllipse => {
                            appwindow.penssidebar().shaper_page().shapebuildertype_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shapebuildertype_fociellipse_row()));
                            appwindow.penssidebar().shaper_page().shapebuildertype_image().set_icon_name(Some("shape-fociellipse-symbolic"));
                        }
                        ShapeBuilderType::QuadBez => {
                            appwindow.penssidebar().shaper_page().shapebuildertype_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shapebuildertype_quadbez_row()));
                            appwindow.penssidebar().shaper_page().shapebuildertype_image().set_icon_name(Some("shape-quadbez-symbolic"));
                        }
                        ShapeBuilderType::CubBez => {
                            appwindow.penssidebar().shaper_page().shapebuildertype_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shapebuildertype_cubbez_row()));
                            appwindow.penssidebar().shaper_page().shapebuildertype_image().set_icon_name(Some("shape-cubbez-symbolic"));
                        }
                    }

                    match style {
                        ShaperStyle::Smooth => {
                            appwindow.penssidebar().shaper_page().shaperstyle_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shaperstyle_smooth_row()));
                            appwindow.penssidebar().shaper_page().width_spinbutton().set_value(smooth_options.stroke_width);
                            appwindow.penssidebar().shaper_page().stroke_colorpicker().set_current_color(smooth_options.stroke_color);
                            appwindow.penssidebar().shaper_page().fill_colorpicker().set_current_color(smooth_options.fill_color);
                            appwindow.penssidebar().shaper_page().shaperstyle_image().set_icon_name(Some("pen-shaper-style-smooth-symbolic"));
                        },
                        ShaperStyle::Rough => {
                            appwindow.penssidebar().shaper_page().shaperstyle_listbox().select_row(Some(&appwindow.penssidebar().shaper_page().shaperstyle_rough_row()));
                            appwindow.penssidebar().shaper_page().width_spinbutton().set_value(rough_options.stroke_width);
                            appwindow.penssidebar().shaper_page().stroke_colorpicker().set_current_color(rough_options.stroke_color);
                            appwindow.penssidebar().shaper_page().fill_colorpicker().set_current_color(rough_options.fill_color);
                            appwindow.penssidebar().shaper_page().shaperstyle_image().set_icon_name(Some("pen-shaper-style-rough-symbolic"));
                        },
                    }
                }

                // Eraser
                appwindow.penssidebar().eraser_page().width_spinbutton().set_value(eraser.width);
                match eraser.style {
                    EraserStyle::TrashCollidingStrokes => appwindow.penssidebar().eraser_page().eraserstyle_trash_colliding_strokes_toggle().set_active(true),
                    EraserStyle::SplitCollidingStrokes => appwindow.penssidebar().eraser_page().eraserstyle_split_colliding_strokes_toggle().set_active(true),
                }

                // Selector
                match selector.style {
                    SelectorType::Polygon => appwindow.penssidebar().selector_page().selectorstyle_polygon_toggle().set_active(true),
                    SelectorType::Rectangle => appwindow.penssidebar().selector_page().selectorstyle_rect_toggle().set_active(true),
                }
                appwindow.penssidebar().selector_page().resize_lock_aspectratio_togglebutton().set_active(selector.resize_lock_aspectratio);

                // Tools
                match tools.style {
                    ToolsStyle::ExpandSheet => appwindow.penssidebar().tools_page().toolstyle_expandsheet_toggle().set_active(true),
                    ToolsStyle::DragProximity => appwindow.penssidebar().tools_page().toolstyle_dragproximity_toggle().set_active(true),
                    ToolsStyle::OffsetCamera => appwindow.penssidebar().tools_page().toolstyle_offsetcamera_toggle().set_active(true),
                }

                // Settings panel
                appwindow.settings_panel().refresh_for_engine(&appwindow);
            }),
        );

        // Trash Selection
        action_selection_trash.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_trash, _| {
                appwindow.canvas().engine().borrow_mut().store.record();

                let selection_keys = appwindow.canvas().engine().borrow().store.selection_keys_as_rendered();
                appwindow.canvas().engine().borrow_mut().store.set_trashed_keys(&selection_keys, true);
                appwindow.canvas().engine().borrow_mut().update_selector();

                appwindow.canvas().update_engine_rendering();
            }),
        );

        // Duplicate Selection
        action_selection_duplicate.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_duplicate, _| {
                appwindow.canvas().engine().borrow_mut().store.record();

                appwindow.canvas().engine().borrow_mut().store.duplicate_selection();
                appwindow.canvas().engine().borrow_mut().update_selector();

                appwindow.canvas().update_engine_rendering();
            }),
        );

        // select all strokes
        action_selection_select_all.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_select_all, _| {
                appwindow.canvas().engine().borrow_mut().store.record();

                let all_strokes = appwindow.canvas().engine().borrow().store.stroke_keys_as_rendered();
                appwindow.canvas().engine().borrow_mut().store.set_selected_keys(&all_strokes, true);
                appwindow.canvas().engine().borrow_mut().update_selector();
                let surface_flags = appwindow.canvas().engine().borrow_mut().handle_penholder_event(PenHolderEvent::ChangeStyle(PenStyle::Selector));
                appwindow.handle_surface_flags(surface_flags);

                appwindow.canvas().update_engine_rendering();
            }),
        );

        // deselect all strokes
        action_selection_deselect_all.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_deselect_all, _| {
                appwindow.canvas().engine().borrow_mut().store.record();

                let all_strokes = appwindow.canvas().engine().borrow().store.stroke_keys_as_rendered();
                appwindow.canvas().engine().borrow_mut().store.set_selected_keys(&all_strokes, false);
                appwindow.canvas().engine().borrow_mut().update_selector();

                appwindow.canvas().update_engine_rendering();
            }),
        );

        // Clear sheet
        action_clear_sheet.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_clear_sheet(&appwindow);
        }));

        // Undo stroke
        action_undo_stroke.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let surface_flags =appwindow.canvas().engine().borrow_mut().undo();
            appwindow.handle_surface_flags(surface_flags);

            appwindow.canvas().update_engine_rendering();
        }));

        // Redo stroke
        action_redo_stroke.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let surface_flags =appwindow.canvas().engine().borrow_mut().redo();
            appwindow.handle_surface_flags(surface_flags);

            appwindow.canvas().update_engine_rendering();
        }));

        // Zoom reset
        action_zoom_reset.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let new_zoom = Camera::ZOOM_DEFAULT;

            let current_sheet_center = appwindow.canvas().current_center_on_sheet();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_sheet(current_sheet_center);
        }));

        // Zoom fit to width
        action_zoom_fit_width.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let mut new_zoom = appwindow.canvas().engine().borrow().camera.total_zoom();

            for _ in 0..2 {
                new_zoom = f64::from(appwindow.canvas_scroller().width()) / appwindow.canvas().engine().borrow().sheet.format.width as f64;
            }

            let current_sheet_center = appwindow.canvas().current_center_on_sheet();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_sheet(current_sheet_center);
        }));

        // Zoom in
        action_zoomin.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let new_zoom = appwindow.canvas().engine().borrow().camera.total_zoom() * (1.0 + RnoteCanvas::ZOOM_STEP);

            let current_sheet_center = appwindow.canvas().current_center_on_sheet();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_sheet(current_sheet_center);
        }));

        // Zoom out
        action_zoomout.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let new_zoom = appwindow.canvas().engine().borrow().camera.total_zoom() * (1.0 - RnoteCanvas::ZOOM_STEP);

            let current_sheet_center = appwindow.canvas().current_center_on_sheet();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_sheet(current_sheet_center);
        }));

        // Zoom to value
        action_zoom_to_value.connect_activate(
            clone!(@weak self as appwindow => move |_action_zoom_to_value, target| {
                let new_zoom = target.unwrap().get::<f64>().unwrap().clamp(Camera::ZOOM_MIN, Camera::ZOOM_MAX);

                appwindow.canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom, RnoteCanvas::ZOOM_TIMEOUT_TIME);

                appwindow.mainheader().canvasmenu().zoomreset_button().set_label(format!("{:.0}%", (100.0 * new_zoom).round()).as_str());
            }));

        // Add page to sheet in fixed size mode
        action_add_page_to_sheet.connect_activate(
            clone!(@weak self as appwindow => move |_action_add_page_to_sheet, _target| {
            let format_height = appwindow.canvas().engine().borrow().sheet.format.height;
            let new_sheet_height = appwindow.canvas().engine().borrow().sheet.height + format_height;
            appwindow.canvas().engine().borrow_mut().sheet.height = new_sheet_height;

            appwindow.canvas().update_engine_rendering();
        }));

        // Resize to fit strokes
        action_resize_to_fit_strokes.connect_activate(
            clone!(@weak self as appwindow => move |_action_resize_to_fit_strokes, _target| {
                appwindow.canvas().engine().borrow_mut().resize_to_fit_strokes();

                appwindow.canvas().update_engine_rendering();
            }),
        );

        // Return to the origin page
        action_return_origin_page.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            appwindow.canvas().return_to_origin_page();

            appwindow.canvas().update_engine_rendering();
        }));

        // New sheet
        action_new_sheet.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_new_sheet(&appwindow);
        }));

        // Open workspace
        action_open_workspace.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_open_workspace(&appwindow);
        }));

        // Open sheet
        action_open_sheet.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_open_sheet(&appwindow);
        }));

        // Save sheet
        action_save_sheet.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            if appwindow.canvas().output_file().is_none() {
                dialogs::dialog_save_sheet_as(&appwindow);
            }

            // check again if a file was selected from the dialog
            if let Some(output_file) = appwindow.canvas().output_file() {
                glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                    appwindow.canvas_progressbar().pulse();

                    if let Err(e) = appwindow.save_sheet_to_file(&output_file).await {
                        appwindow.canvas().set_output_file(None);

                        log::error!("saving sheet failed with error `{}`", e);
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving sheet failed.").to_variant()));
                    }

                    appwindow.finish_canvas_progressbar();
                }));
                // No success toast on saving without dialog, success is already indicated in the header title
            }
        }));

        // Save sheet as
        action_save_sheet_as.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_save_sheet_as(&appwindow);
        }));

        // Print sheet
        action_print_sheet.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            appwindow.canvas_progressbar().pulse();

            let print_op = PrintOperation::builder()
                .unit(Unit::Points)
                .build();

                let pages_bounds = appwindow.canvas().engine().borrow().pages_bounds_containing_content();
                let n_pages = pages_bounds.len();

            print_op.connect_begin_print(clone!(@weak appwindow => move |print_op, _print_cx| {
                print_op.set_n_pages(n_pages as i32);
            }));

            let sheet_bounds = appwindow.canvas().engine().borrow().sheet.bounds();

            print_op.connect_draw_page(clone!(@weak appwindow => move |_print_op, print_cx, page_nr| {
                appwindow.canvas_progressbar().pulse();

                let cx = print_cx.cairo_context();

                if let Err(e) = || -> anyhow::Result<()> {
                    let print_zoom = {
                        let width_scale = print_cx.width() / appwindow.canvas().engine().borrow().sheet.format.width;
                        let height_scale = print_cx.height() / appwindow.canvas().engine().borrow().sheet.format.height;
                        width_scale.min(height_scale)
                    };

                    let page_bounds = pages_bounds[page_nr as usize];

                    let page_svgs = appwindow.canvas().engine().borrow().gen_svgs_intersecting_bounds(page_bounds)?;

                    cx.scale(print_zoom, print_zoom);
                    cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);

                    cx.rectangle(
                        page_bounds.mins[0],
                        page_bounds.mins[1],
                        page_bounds.extents()[0],
                        page_bounds.extents()[1]
                    );
                    cx.clip();

                    render::Svg::draw_svgs_to_cairo_context(&page_svgs, sheet_bounds, &cx)?;
                    Ok(())
                }() {
                    log::error!("draw_page() failed while printing page: {}, Err {}", page_nr, e);
                }
            }));

            print_op.connect_status_changed(clone!(@weak appwindow => move |print_op| {
                log::debug!("{:?}", print_op.status());
                match print_op.status() {
                    PrintStatus::Finished => {
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Printed sheet successfully").to_variant()));
                    }
                    _ => {}
                }
            }));

            // Run the print op
            if let Err(e) = print_op.run(PrintOperationAction::PrintDialog, Some(&appwindow)){
                log::error!("print_op.run() failed with Err, {}", e);
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Printing sheet failed").to_variant()));
            }


            appwindow.finish_canvas_progressbar();
        }));

        // Import
        action_import_file.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::dialog_import_file(&appwindow);
        }));

        // Export selection as SVG
        action_export_selection_as_svg.connect_activate(
            clone!(@weak self as appwindow => move |_,_| {
                dialogs::dialog_export_selection(&appwindow);
            }),
        );

        // Export sheet as SVG
        action_export_sheet_as_svg.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::dialog_export_sheet_as_svg(&appwindow);
        }));

        // Export sheet as PDF
        action_export_sheet_as_pdf.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::dialog_export_sheet_as_pdf(&appwindow);
        }));

        // Export sheet as Xopp
        action_export_sheet_as_xopp.connect_activate(
            clone!(@weak self as appwindow => move |_,_| {
                dialogs::dialog_export_sheet_as_xopp(&appwindow);
            }),
        );

        // Clipboard copy selection
        action_clipboard_copy_selection.connect_activate(clone!(@weak self as appwindow => move |_, _| {
        match appwindow.canvas().engine().borrow().export_selection_as_svg_string() {
            Ok(Some(selection_svg_data)) => {
                let svg_content_provider = gdk::ContentProvider::for_bytes("image/svg+xml", &glib::Bytes::from(selection_svg_data.as_bytes()));
                if let Err(e) = appwindow.clipboard().set_content(Some(&svg_content_provider)) {
                    log::error!("set_content() failed in clipboard_copy_selection actino, Err {}", e);
                }
            }
            Ok(None) => {
                log::debug!("can't copy selection into clipboard. Is empty");
            }
            Err(e) => {
                log::error!("export_selection_as_svg_string() failed in clipboard_copy_selection action, Err {}", e);
            }
        }
    }));

        // Clipboard paste as selection
        action_clipboard_paste_selection.connect_activate(clone!(@weak self as appwindow => move |_, _| {
        let clipboard = appwindow.clipboard();
            for mime_type in clipboard.formats().mime_types() {
                    match mime_type.as_str() {
                        "image/svg+xml" => {
                            appwindow.clipboard().read_text_async(None::<&gio::Cancellable>, clone!(@weak appwindow => move |text_res| {
                                match text_res {
                                    Ok(Some(text)) => {
                                        appwindow.load_in_vectorimage_bytes(text.as_bytes(), None).unwrap_or_else(|e| {
                                            log::error!("failed to paste clipboard as vector image, load_in_vectorimage_bytes() returned Err, {}", e);
                                        });
                                    }
                                    Ok(None) => {}
                                    Err(e) => {
                                        log::error!("failed to paste clipboard as vector image, read_text_async() returned Err, {}", e);

                                    }
                                }
                            }));
                            break;
                        }
/*                         "image/png" | "image/jpeg" => {
                            appwindow.clipboard().read_texture_async(gio::NONE_CANCELLABLE, clone!(@weak appwindow => move |texture_res| {
                                match texture_res {
                                    Ok(Some(texture)) => {
                                        let mut texture_bytes: Vec<u8> = Vec::new();
                                        texture.download(&mut texture_bytes, texture.width() as usize * 4);

                                        if let Some(image) = image::ImageBuffer::<image::Bgra<u8>, Vec<u8>>::from_vec(texture.width() as u32, texture.height() as u32, texture_bytes) {
                                            let mut image_bytes = Vec::<u8>::new();
                                            image::DynamicImage::ImageBgra8(image).write_to(&mut image_bytes, image::ImageOutputFormat::Png).unwrap_or_else(|e| {
                                                log::error!("failed to paste clipboard as BitmapImage, DynamicImage.write_to() returned Err, {}", e);
                                            });

                                            appwindow.load_in_bitmapimage_bytes(&image_bytes).unwrap_or_else(|e| {
                                                log::error!("failed to paste clipboard as BitmapImage, load_in_vectorimage_bytes() returned Err, {}", e);
                                            });
                                        };


                                    }
                                    Ok(None) => {
                                    }
                                    Err(e) => {
                                        log::error!("failed to paste clipboard as BitmapImage, texture in callback is Err, {}", e);
                                    }
                                }
                            }));
                            break;
                        }
                        */
                        // Pdfs are not supported in the clipboard
                        _ => {}
                    }
            }
    }));
    }

    pub fn setup_action_accels(&self) {
        let app = self.application().unwrap().downcast::<RnoteApp>().unwrap();

        app.set_accels_for_action("win.close-active", &["<Ctrl>w"]);
        app.set_accels_for_action("win.fullscreen", &["F11"]);
        app.set_accels_for_action("win.keyboard-shortcuts", &["<Ctrl>question"]);
        app.set_accels_for_action("win.open-canvasmenu", &["F9"]);
        app.set_accels_for_action("win.open-appmenu", &["F10"]);
        app.set_accels_for_action("win.new-sheet", &["<Ctrl>n"]);
        app.set_accels_for_action("win.open-sheet", &["<Ctrl>o"]);
        app.set_accels_for_action("win.save-sheet", &["<Ctrl>s"]);
        app.set_accels_for_action("win.save-sheet-as", &["<Ctrl><Shift>s"]);
        app.set_accels_for_action("win.clear-sheet", &["<Ctrl>l"]);
        app.set_accels_for_action("win.print-sheet", &["<Ctrl>p"]);
        app.set_accels_for_action("win.import-file", &["<Ctrl>i"]);
        app.set_accels_for_action("win.undo", &["<Ctrl>z"]);
        app.set_accels_for_action("win.redo", &["<Ctrl><Shift>z"]);
        app.set_accels_for_action("win.zoomin", &["plus"]);
        app.set_accels_for_action("win.zoomout", &["minus"]);
        app.set_accels_for_action("win.selection-trash", &["Delete"]);
        app.set_accels_for_action("win.selection-duplicate", &["<Ctrl>d"]);
        app.set_accels_for_action("win.selection-select-all", &["<Ctrl>a"]);
        app.set_accels_for_action("win.selection-deselect-all", &["Escape"]);
        app.set_accels_for_action("win.clipboard-copy-selection", &["<Ctrl>c"]);
        app.set_accels_for_action("win.clipboard-paste-selection", &["<Ctrl>v"]);

        // shortcuts for devel builds
        if config::PROFILE.to_lowercase().as_str() == "devel" {
            app.set_accels_for_action("win.visual-debug", &["<Ctrl><Shift>v"]);
        }
    }
}
