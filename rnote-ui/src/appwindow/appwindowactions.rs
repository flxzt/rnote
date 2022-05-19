use super::RnoteAppWindow;
use crate::config;
use crate::{
    app::RnoteApp,
    {dialogs, RnoteCanvas},
};
use rnote_compose::builders::ShapeBuilderType;
use rnote_engine::document::Layout;
use rnote_engine::pens::brush::BrushStyle;
use rnote_engine::pens::eraser::EraserStyle;
use rnote_engine::pens::penholder::PenStyle;
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
        let action_devel_menu = gio::SimpleAction::new("devel-menu", None);
        self.add_action(&action_devel_menu);
        let action_visual_debug =
            gio::SimpleAction::new_stateful("visual-debug", None, &false.to_variant());
        self.add_action(&action_visual_debug);
        let action_debug_export_engine_state =
            gio::SimpleAction::new("debug-export-engine-state", None);
        self.add_action(&action_debug_export_engine_state);
        let action_debug_export_engine_config =
            gio::SimpleAction::new("debug-export-engine-config", None);
        self.add_action(&action_debug_export_engine_config);
        let action_righthanded = gio::PropertyAction::new("righthanded", self, "righthanded");
        self.add_action(&action_righthanded);
        let action_touch_drawing =
            gio::PropertyAction::new("touch-drawing", &self.canvas(), "touch-drawing");
        self.add_action(&action_touch_drawing);

        // Engine actions
        let action_pdf_import_width_perc = gio::SimpleAction::new(
            "pdf-import-width-perc",
            Some(&glib::VariantType::new("d").unwrap()),
        );
        self.add_action(&action_pdf_import_width_perc);
        let action_pdf_import_as_vector = gio::SimpleAction::new(
            "pdf-import-as-vector",
            Some(&glib::VariantType::new("b").unwrap()),
        );
        self.add_action(&action_pdf_import_as_vector);
        let action_pen_sounds =
            gio::SimpleAction::new_stateful("pen-sounds", None, &false.to_variant());
        self.add_action(&action_pen_sounds);
        let action_format_borders =
            gio::SimpleAction::new_stateful("format-borders", None, &true.to_variant());
        self.add_action(&action_format_borders);
        let action_doc_layout = gio::SimpleAction::new_stateful(
            "doc-layout",
            Some(&glib::VariantType::new("s").unwrap()),
            &String::from("infinite").to_variant(),
        );
        self.add_action(&action_doc_layout);
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
        let action_add_page_to_doc = gio::SimpleAction::new("add-page-to-doc", None);
        self.add_action(&action_add_page_to_doc);
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
        let action_clear_doc = gio::SimpleAction::new("clear-doc", None);
        self.add_action(&action_clear_doc);
        let action_new_doc = gio::SimpleAction::new("new-doc", None);
        self.add_action(&action_new_doc);
        let action_save_doc = gio::SimpleAction::new("save-doc", None);
        self.add_action(&action_save_doc);
        let action_save_doc_as = gio::SimpleAction::new("save-doc-as", None);
        self.add_action(&action_save_doc_as);
        let action_autosave = gio::PropertyAction::new("autosave", self, "autosave");
        self.add_action(&action_autosave);
        let action_open_doc = gio::SimpleAction::new("open-doc", None);
        self.add_action(&action_open_doc);
        let action_open_workspace = gio::SimpleAction::new("open-workspace", None);
        self.add_action(&action_open_workspace);
        let action_print_doc = gio::SimpleAction::new("print-doc", None);
        self.add_action(&action_print_doc);
        let action_import_file = gio::SimpleAction::new("import-file", None);
        self.add_action(&action_import_file);
        let action_export_selection_as_svg =
            gio::SimpleAction::new("export-selection-as-svg", None);
        self.add_action(&action_export_selection_as_svg);
        let action_export_doc_as_svg = gio::SimpleAction::new("export-doc-as-svg", None);
        self.add_action(&action_export_doc_as_svg);
        let action_export_doc_as_pdf = gio::SimpleAction::new("export-doc-as-pdf", None);
        self.add_action(&action_export_doc_as_pdf);
        let action_export_doc_as_xopp = gio::SimpleAction::new("export-doc-as-xopp", None);
        self.add_action(&action_export_doc_as_xopp);
        let action_clipboard_copy_selection =
            gio::SimpleAction::new("clipboard-copy-selection", None);
        self.add_action(&action_clipboard_copy_selection);
        let action_clipboard_paste = gio::SimpleAction::new("clipboard-paste", None);
        self.add_action(&action_clipboard_paste);
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
            clone!(@weak self as appwindow, @weak action_devel_menu => move |action_devel_mode, _target| {
                let state = action_devel_mode.state().unwrap().get::<bool>().unwrap();

                // Enable the devel menu action to reveal it in the app menu
                action_devel_menu.set_enabled(!state);

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
        action_devel_menu.set_enabled(false);

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

        // Export engine config
        action_debug_export_engine_config.connect_activate(
            clone!(@weak self as appwindow => move |_action_debug_export_engine_config, _target| {
                dialogs::dialog_export_engine_config(&appwindow);
            }),
        );

        // Doc layout
        action_doc_layout.connect_activate(
            clone!(@weak self as appwindow => move |action_doc_layout, target| {
                let doc_layout = target.unwrap().str().unwrap();

                match doc_layout {
                    "fixed-size" => {
                        appwindow.canvas().engine().borrow_mut().set_doc_layout(Layout::FixedSize);
                        appwindow.canvas_fixedsize_quickactions_revealer().set_reveal_child(true);
                    },
                    "continuous-vertical" => {
                        appwindow.canvas().engine().borrow_mut().set_doc_layout(Layout::ContinuousVertical);
                        appwindow.canvas_fixedsize_quickactions_revealer().set_reveal_child(false);
                    },
                    "infinite" => {
                        appwindow.canvas().engine().borrow_mut().set_doc_layout(Layout::Infinite);
                        appwindow.canvas_fixedsize_quickactions_revealer().set_reveal_child(false);
                    }
                    invalid_str => {
                        log::error!("action doc-layout failed, invalid str: {}", invalid_str);
                        return;
                    }
                }
                appwindow.canvas().update_engine_rendering();

                action_doc_layout.set_state(&doc_layout.to_variant());
            }));

        // Righthanded
        action_righthanded.connect_state_notify(
            clone!(@weak self as appwindow => move |action_righthanded| {
                let current_state = action_righthanded.state().unwrap().get::<bool>().unwrap();

                if current_state {
                    appwindow.flap().set_flap_position(PackType::Start);
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
                        .brushconfig_menubutton()
                        .set_direction(ArrowType::Right);
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .brushstyle_menubutton()
                        .set_direction(ArrowType::Right);
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
                        .shaper_page()
                        .shapeconfig_menubutton()
                        .set_direction(ArrowType::Right);
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapebuildertype_menubutton()
                        .set_direction(ArrowType::Right);
                } else {
                    appwindow.flap().set_flap_position(PackType::End);
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
                        .brushconfig_menubutton()
                        .set_direction(ArrowType::Left);
                    appwindow
                        .penssidebar()
                        .brush_page()
                        .brushstyle_menubutton()
                        .set_direction(ArrowType::Left);
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
                        .shaper_page()
                        .shapeconfig_menubutton()
                        .set_direction(ArrowType::Left);
                    appwindow
                        .penssidebar()
                        .shaper_page()
                        .shapebuildertype_menubutton()
                        .set_direction(ArrowType::Left);
                }
            }),
        );

        // Pdf import width perc
        action_pdf_import_width_perc.connect_activate(
            clone!(@weak self as appwindow => move |_action_pdf_import_width_perc, target| {
                let pdf_import_width_perc = target.unwrap().get::<f64>().unwrap();

                appwindow.canvas().engine().borrow_mut().pdf_import_width_perc = pdf_import_width_perc;

                appwindow.settings_panel().refresh_for_engine(&appwindow);
            }),
        );

        // Pdf import as vector
        action_pdf_import_as_vector.connect_activate(
            clone!(@weak self as appwindow => move |_action_pdf_import_as_vector, target| {
                let pdf_import_as_vector = target.unwrap().get::<bool>().unwrap();

                appwindow.canvas().engine().borrow_mut().pdf_import_as_vector = pdf_import_as_vector;

                appwindow.settings_panel().refresh_for_engine(&appwindow);
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

                appwindow.canvas().engine().borrow_mut().document.format.show_borders = format_borders;
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
                    if new_pen_style != appwindow.canvas().engine().borrow().penholder.current_style_w_override() {
                        let mut surface_flags = appwindow.canvas().engine().borrow_mut().change_pen_style(
                            new_pen_style,
                        );
                        surface_flags = surface_flags.merged_with_other(appwindow.canvas().engine().borrow_mut().change_pen_style_override(
                            None,
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

                let new_pen_style_override= match pen_style_override {
                    "brush" => {
                        Some(Some(PenStyle::Brush))
                    }
                    "shaper" => {
                        Some(Some(PenStyle::Shaper))
                    }
                    "eraser" => {
                        Some(Some(PenStyle::Eraser))
                    }
                    "selector" => {
                        Some(Some(PenStyle::Selector))
                    }
                    "tools" => {
                        Some(Some(PenStyle::Tools))
                    }
                    "none" => {
                        Some(None)
                    }
                    _ => {
                        log::error!("invalid target for action_pen_overwrite, `{}`", pen_style_override);
                        None
                    }
                };

                if let Some(new_pen_style_override) = new_pen_style_override {
                    let surface_flags = appwindow.canvas().engine().borrow_mut().change_pen_style_override(
                        new_pen_style_override,
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
                "verticalspace" => {
                    appwindow.canvas().engine().borrow_mut().penholder.tools.style = tools::ToolsStyle::VerticalSpace;
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
            clone!(
                @weak self as appwindow,
                @strong action_pen_sounds,
                @strong action_doc_layout,
                @strong action_format_borders,
                @strong action_pdf_import_width_perc,
                @strong action_pdf_import_as_vector
                => move |_action_refresh_ui_for_engine, _| {
                // Avoids borrow errors
                let format = appwindow.canvas().engine().borrow().document.format.clone();
                let doc_layout = appwindow.canvas().engine().borrow().doc_layout();
                let pdf_import_as_vector = appwindow.canvas().engine().borrow().pdf_import_as_vector;
                let pdf_import_width_perc = appwindow.canvas().engine().borrow().pdf_import_width_perc;
                let pen_sounds = appwindow.canvas().engine().borrow().penholder.pen_sounds();
                let pen_style = appwindow.canvas().engine().borrow().penholder.current_style_w_override();
                let brush = appwindow.canvas().engine().borrow().penholder.brush.clone();
                let eraser = appwindow.canvas().engine().borrow().penholder.eraser.clone();
                let selector = appwindow.canvas().engine().borrow().penholder.selector.clone();
                let tools = appwindow.canvas().engine().borrow().penholder.tools.clone();

                {
                    // Engine
                    let doc_layout = match doc_layout {
                        Layout::FixedSize => "fixed-size",
                        Layout::ContinuousVertical => "continuous-vertical",
                        Layout::Infinite => "infinite",
                    };
                    action_doc_layout.activate(Some(&doc_layout.to_variant()));
                    action_pdf_import_as_vector.activate(Some(&pdf_import_as_vector.to_variant()));
                    action_pdf_import_width_perc.activate(Some(&pdf_import_width_perc.to_variant()));
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
                appwindow.penssidebar().brush_page().set_solidstyle_pressure_curve(brush.smooth_options.pressure_curve);
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
                    ToolsStyle::VerticalSpace => appwindow.penssidebar().tools_page().toolstyle_verticalspace_toggle().set_active(true),
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
                let surface_flags = appwindow.canvas().engine().borrow_mut().record();
                appwindow.handle_surface_flags(surface_flags);

                let selection_keys = appwindow.canvas().engine().borrow().store.selection_keys_as_rendered();
                appwindow.canvas().engine().borrow_mut().store.set_trashed_keys(&selection_keys, true);

                appwindow.canvas().engine().borrow_mut().update_selector();
                appwindow.canvas().engine().borrow_mut().resize_autoexpand();
                appwindow.canvas().update_engine_rendering();
            }),
        );

        // Duplicate Selection
        action_selection_duplicate.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_duplicate, _| {
                let surface_flags = appwindow.canvas().engine().borrow_mut().record();
                appwindow.handle_surface_flags(surface_flags);

                let new_selected = appwindow.canvas().engine().borrow_mut().store.duplicate_selection();
                appwindow.canvas().engine().borrow_mut().store.update_geometry_for_strokes(&new_selected);


                appwindow.canvas().engine().borrow_mut().update_selector();
                appwindow.canvas().engine().borrow_mut().resize_autoexpand();
                appwindow.canvas().update_engine_rendering();
            }),
        );

        // select all strokes
        action_selection_select_all.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_select_all, _| {
                let surface_flags = appwindow.canvas().engine().borrow_mut().record();
                appwindow.handle_surface_flags(surface_flags);

                let all_strokes = appwindow.canvas().engine().borrow().store.stroke_keys_as_rendered();
                appwindow.canvas().engine().borrow_mut().store.set_selected_keys(&all_strokes, true);
                appwindow.canvas().engine().borrow_mut().update_selector();
                let surface_flags = appwindow.canvas().engine().borrow_mut().change_pen_style(PenStyle::Selector);
                appwindow.handle_surface_flags(surface_flags);

                appwindow.canvas().update_engine_rendering();
            }),
        );

        // deselect all strokes
        action_selection_deselect_all.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_deselect_all, _| {
                let surface_flags = appwindow.canvas().engine().borrow_mut().record();
                appwindow.handle_surface_flags(surface_flags);

                let all_strokes = appwindow.canvas().engine().borrow().store.selection_keys_as_rendered();
                appwindow.canvas().engine().borrow_mut().store.set_selected_keys(&all_strokes, false);

                appwindow.canvas().engine().borrow_mut().update_selector();
                appwindow.canvas().engine().borrow_mut().resize_autoexpand();
                appwindow.canvas().update_engine_rendering();
            }),
        );

        // Clear doc
        action_clear_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_clear_doc(&appwindow);
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

            let current_doc_center = appwindow.canvas().current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom fit to width
        action_zoom_fit_width.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let mut new_zoom = appwindow.canvas().engine().borrow().camera.total_zoom();

            for _ in 0..2 {
                new_zoom = f64::from(appwindow.canvas_scroller().width()) / appwindow.canvas().engine().borrow().document.format.width as f64;
            }

            let current_doc_center = appwindow.canvas().current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom in
        action_zoomin.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let new_zoom = appwindow.canvas().engine().borrow().camera.total_zoom() * (1.0 + RnoteCanvas::ZOOM_STEP);

            let current_doc_center = appwindow.canvas().current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom out
        action_zoomout.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let new_zoom = appwindow.canvas().engine().borrow().camera.total_zoom() * (1.0 - RnoteCanvas::ZOOM_STEP);

            let current_doc_center = appwindow.canvas().current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            appwindow.canvas().center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom to value
        action_zoom_to_value.connect_activate(
            clone!(@weak self as appwindow => move |_action_zoom_to_value, target| {
                let new_zoom = target.unwrap().get::<f64>().unwrap().clamp(Camera::ZOOM_MIN, Camera::ZOOM_MAX);

                appwindow.canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom, RnoteCanvas::ZOOM_TIMEOUT_TIME);

                appwindow.mainheader().canvasmenu().zoomreset_button().set_label(format!("{:.0}%", (100.0 * new_zoom).round()).as_str());
            }));

        // Add page to doc in fixed size mode
        action_add_page_to_doc.connect_activate(
            clone!(@weak self as appwindow => move |_action_add_page_to_doc, _target| {
            let format_height = appwindow.canvas().engine().borrow().document.format.height;
            let new_doc_height = appwindow.canvas().engine().borrow().document.height + format_height;
            appwindow.canvas().engine().borrow_mut().document.height = new_doc_height;

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

            appwindow.canvas().engine().borrow_mut().resize_autoexpand();
            appwindow.canvas().update_engine_rendering();
        }));

        // New doc
        action_new_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_new_doc(&appwindow);
        }));

        // Open workspace
        action_open_workspace.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_open_workspace(&appwindow);
        }));

        // Open doc
        action_open_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_open_doc(&appwindow);
        }));

        // Save doc
        action_save_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            if appwindow.canvas().output_file().is_none() {
                dialogs::dialog_save_doc_as(&appwindow);
            }

            // check again if a file was selected from the dialog
            if let Some(output_file) = appwindow.canvas().output_file() {
                glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                    appwindow.start_pulsing_canvas_progressbar();

                    if let Err(e) = appwindow.save_document_to_file(&output_file).await {
                        appwindow.canvas().set_output_file(None);

                        log::error!("saving document failed with error `{}`", e);
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving document failed.").to_variant()));
                    }

                    appwindow.finish_canvas_progressbar();
                }));
                // No success toast on saving without dialog, success is already indicated in the header title
            }
        }));

        // Save doc as
        action_save_doc_as.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_save_doc_as(&appwindow);
        }));

        // Print doc
        action_print_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            appwindow.start_pulsing_canvas_progressbar();

            let print_op = PrintOperation::builder()
                .unit(Unit::Points)
                .build();

                let pages_bounds = appwindow.canvas().engine().borrow().pages_bounds_containing_content();
                let n_pages = pages_bounds.len();

            print_op.connect_begin_print(clone!(@weak appwindow => move |print_op, _print_cx| {
                print_op.set_n_pages(n_pages as i32);
            }));

            let doc_bounds = appwindow.canvas().engine().borrow().document.bounds();

            print_op.connect_draw_page(clone!(@weak appwindow => move |_print_op, print_cx, page_nr| {
                let cx = print_cx.cairo_context();

                if let Err(e) = || -> anyhow::Result<()> {
                    let print_zoom = {
                        let width_scale = print_cx.width() / appwindow.canvas().engine().borrow().document.format.width;
                        let height_scale = print_cx.height() / appwindow.canvas().engine().borrow().document.format.height;
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

                    render::Svg::draw_svgs_to_cairo_context(&page_svgs, doc_bounds, &cx)?;
                    Ok(())
                }() {
                    log::error!("draw_page() failed while printing page: {}, Err {}", page_nr, e);
                }
            }));

            print_op.connect_status_changed(clone!(@weak appwindow => move |print_op| {
                log::debug!("{:?}", print_op.status());
                match print_op.status() {
                    PrintStatus::Finished => {
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Printed document successfully").to_variant()));
                    }
                    _ => {}
                }
            }));

            // Run the print op
            if let Err(e) = print_op.run(PrintOperationAction::PrintDialog, Some(&appwindow)){
                log::error!("print_op.run() failed with Err, {}", e);
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Printing document failed").to_variant()));
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

        // Export document as SVG
        action_export_doc_as_svg.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::dialog_export_doc_as_svg(&appwindow);
        }));

        // Export document as PDF
        action_export_doc_as_pdf.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::dialog_export_doc_as_pdf(&appwindow);
        }));

        // Export document as Xopp
        action_export_doc_as_xopp.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::dialog_export_doc_as_xopp(&appwindow);
        }));

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
        action_clipboard_paste.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            let content_formats = appwindow.clipboard().formats();

            if content_formats.contain_mime_type("image/svg+xml") {
                glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                    match appwindow.clipboard().read_text_future().await {
                        Ok(Some(text)) => {
                                if let Err(e) = appwindow.load_in_vectorimage_bytes(text.as_bytes().to_vec(), None).await {
                                    log::error!("failed to paste clipboard as vector image, load_in_vectorimage_bytes() returned Err, {}", e);
                                };
                        }
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("failed to paste clipboard as vector image, read_text() failed with Err {}", e);

                        }
                    }
                }));
            } else if content_formats.contain_mime_type("text/uri-list") {
                glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                    match appwindow.clipboard().read_text_future().await {
                        Ok(Some(text)) => {
                            let path = std::path::Path::new(text.as_str());

                            if path.exists() {
                                appwindow.open_file_w_dialogs(&gio::File::for_path(&path), None);
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("failed to paste clipboard from path, read_text() failed with Err {}", e);

                        }
                    }
                }));
            } else if content_formats.contain_mime_type("image/png") {
                glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                    match appwindow.clipboard().read_texture_future().await {
                        Ok(Some(texture)) => {
                            if let Err(e) = appwindow.load_in_bitmapimage_bytes(texture.save_to_png_bytes().to_vec(), None).await {
                                log::error!("failed to paste clipboard as png image, load_in_bitmapimage_bytes() returned Err {}", e);
                            };
                        }
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("failed to paste clipboard as png image, read_texture_future() failed with Err {}", e);
                        }
                    }
                }));
            } else {
                log::debug!("failed to paste clipboard as vector image, unsupported mime-type");
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
        app.set_accels_for_action("win.new-doc", &["<Ctrl>n"]);
        app.set_accels_for_action("win.open-doc", &["<Ctrl>o"]);
        app.set_accels_for_action("win.save-doc", &["<Ctrl>s"]);
        app.set_accels_for_action("win.save-doc-as", &["<Ctrl><Shift>s"]);
        app.set_accels_for_action("win.clear-doc", &["<Ctrl>l"]);
        app.set_accels_for_action("win.print-doc", &["<Ctrl>p"]);
        app.set_accels_for_action("win.import-file", &["<Ctrl>i"]);
        app.set_accels_for_action("win.undo", &["<Ctrl>z"]);
        app.set_accels_for_action("win.redo", &["<Ctrl><Shift>z"]);
        app.set_accels_for_action("win.zoomin", &["plus"]);
        app.set_accels_for_action("win.zoomout", &["minus"]);
        app.set_accels_for_action("win.selection-trash", &["Delete"]);
        app.set_accels_for_action("win.selection-duplicate", &["<Ctrl>d"]);
        app.set_accels_for_action("win.selection-select-all", &["<Ctrl>a"]);
        app.set_accels_for_action("win.selection-deselect-all", &["<Ctrl><Shift>a"]);
        app.set_accels_for_action("win.clipboard-copy-selection", &["<Ctrl>c"]);
        app.set_accels_for_action("win.clipboard-paste", &["<Ctrl>v"]);

        // shortcuts for devel builds
        if config::PROFILE.to_lowercase().as_str() == "devel" {
            app.set_accels_for_action("win.visual-debug", &["<Ctrl><Shift>v"]);
        }
    }
}
