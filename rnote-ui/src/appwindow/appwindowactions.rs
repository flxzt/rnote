use super::RnAppWindow;
use crate::canvas::RnCanvasLayout;
use crate::config;
use crate::{dialogs, RnCanvas};
use piet::RenderContext;
use rnote_compose::helpers::Vector2Helpers;
use rnote_compose::penevents::ShortcutKey;
use rnote_engine::document::Layout;
use rnote_engine::engine::RNOTE_STROKE_CONTENT_MIME_TYPE;
use rnote_engine::pens::PenStyle;
use rnote_engine::{render, Camera, DrawBehaviour, RnoteEngine};

use gettextrs::gettext;
use gtk4::{gdk, gio, glib, glib::clone, prelude::*, PrintOperation, PrintOperationAction, Unit};
use gtk4::{PrintStatus, Window};
use std::path::PathBuf;
use std::time::Instant;

const CLIPBOARD_INPUT_STREAM_BUFSIZE: usize = 4096;

impl RnAppWindow {
    /// Boolean actions have no target, and a boolean state. They have a default implementation for the activate signal, which requests the state to be inverted, and the default implementation for change_state, which sets the state to the request.
    /// We generally want to connect to the change_state signal. (but then have to set the state with action.set_state() )
    /// We can then either toggle the state through activating the action, or set the state explicitly through action.change_state(<request>)
    pub(crate) fn setup_actions(&self) {
        let action_fullscreen = gio::PropertyAction::new("fullscreen", self, "fullscreened");
        self.add_action(&action_fullscreen);
        let action_open_settings = gio::SimpleAction::new("open-settings", None);
        self.add_action(&action_open_settings);
        let action_about = gio::SimpleAction::new("about", None);
        self.add_action(&action_about);
        let action_donate = gio::SimpleAction::new("donate", None);
        self.add_action(&action_donate);
        let action_keyboard_shortcuts_dialog = gio::SimpleAction::new("keyboard-shortcuts", None);
        self.add_action(&action_keyboard_shortcuts_dialog);
        let action_open_canvasmenu = gio::SimpleAction::new("open-canvasmenu", None);
        self.add_action(&action_open_canvasmenu);
        let action_open_appmenu = gio::SimpleAction::new("open-appmenu", None);
        self.add_action(&action_open_appmenu);
        let action_devel_mode =
            gio::SimpleAction::new_stateful("devel-mode", None, &false.to_variant());
        self.add_action(&action_devel_mode);
        let action_devel_menu = gio::SimpleAction::new("devel-menu", None);
        self.add_action(&action_devel_menu);
        let action_new_tab = gio::SimpleAction::new("new-tab", None);
        self.add_action(&action_new_tab);
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
        let action_touch_drawing = gio::PropertyAction::new("touch-drawing", self, "touch-drawing");
        self.add_action(&action_touch_drawing);

        let action_pen_sounds =
            gio::SimpleAction::new_stateful("pen-sounds", None, &false.to_variant());
        self.add_action(&action_pen_sounds);
        let action_format_borders =
            gio::SimpleAction::new_stateful("format-borders", None, &true.to_variant());
        self.add_action(&action_format_borders);
        // Couldn't make it work with enums as state together with activating from menu model, so using strings instead
        let action_doc_layout = gio::SimpleAction::new_stateful(
            "doc-layout",
            Some(&String::static_variant_type()),
            &String::from("infinite").to_variant(),
        );
        self.add_action(&action_doc_layout);
        let action_pen_style = gio::SimpleAction::new_stateful(
            "pen-style",
            Some(&PenStyle::static_variant_type()),
            &PenStyle::Brush.to_variant(),
        );
        self.add_action(&action_pen_style);
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
        let action_print_doc = gio::SimpleAction::new("print-doc", None);
        self.add_action(&action_print_doc);
        let action_import_file = gio::SimpleAction::new("import-file", None);
        self.add_action(&action_import_file);
        let action_export_doc = gio::SimpleAction::new("export-doc", None);
        self.add_action(&action_export_doc);
        let action_export_doc_pages = gio::SimpleAction::new("export-doc-pages", None);
        self.add_action(&action_export_doc_pages);
        let action_export_selection = gio::SimpleAction::new("export-selection", None);
        self.add_action(&action_export_selection);
        let action_clipboard_copy = gio::SimpleAction::new("clipboard-copy", None);
        self.add_action(&action_clipboard_copy);
        let action_clipboard_cut = gio::SimpleAction::new("clipboard-cut", None);
        self.add_action(&action_clipboard_cut);
        let action_clipboard_paste = gio::SimpleAction::new("clipboard-paste", None);
        self.add_action(&action_clipboard_paste);
        let action_active_tab_move_left = gio::SimpleAction::new("active-tab-move-left", None);
        self.add_action(&action_active_tab_move_left);
        let action_active_tab_move_right = gio::SimpleAction::new("active-tab-move-right", None);
        self.add_action(&action_active_tab_move_right);
        let action_active_tab_close = gio::SimpleAction::new("active-tab-close", None);
        self.add_action(&action_active_tab_close);

        let action_drawing_pad_pressed_button_0 =
            gio::SimpleAction::new("drawing-pad-pressed-button-0", None);
        self.add_action(&action_drawing_pad_pressed_button_0);
        let action_drawing_pad_pressed_button_1 =
            gio::SimpleAction::new("drawing-pad-pressed-button-1", None);
        self.add_action(&action_drawing_pad_pressed_button_1);
        let action_drawing_pad_pressed_button_2 =
            gio::SimpleAction::new("drawing-pad-pressed-button-2", None);
        self.add_action(&action_drawing_pad_pressed_button_2);
        let action_drawing_pad_pressed_button_3 =
            gio::SimpleAction::new("drawing-pad-pressed-button-3", None);
        self.add_action(&action_drawing_pad_pressed_button_3);

        // Open settings
        action_open_settings.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            appwindow.flap_stack().set_visible_child_name("settings_page");
            appwindow.flap().set_reveal_flap(true);
        }));

        // About Dialog
        action_about.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_about(&appwindow);
        }));

        // Donate
        action_donate.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            gtk4::show_uri(None::<&Window>, config::APP_DONATE_URL, 0);
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
        // Its enabled state toggles the visibility of the developer settings menu entry. Is only modified inside action_devel_mode
        action_devel_menu.set_enabled(false);

        // Visual debugging
        action_visual_debug.connect_change_state(
            clone!(@weak self as appwindow => move |action_visual_debug, state_request| {
                let requested_state = state_request.unwrap().get::<bool>().unwrap();
                let canvas = appwindow.active_tab().canvas();

                canvas.engine().borrow_mut().visual_debug = requested_state;
                canvas.queue_draw();
                action_visual_debug.set_state(&requested_state.to_variant());
            }),
        );

        // Create page
        action_new_tab.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            appwindow.new_tab();
        }));

        // Export engine state
        action_debug_export_engine_state.connect_activate(
            clone!(@weak self as appwindow => move |_action_debug_export_engine_state, _target| {
                dialogs::export::filechooser_export_engine_state(&appwindow, &appwindow.active_tab().canvas());
            }),
        );

        // Export engine config
        action_debug_export_engine_config.connect_activate(
            clone!(@weak self as appwindow => move |_action_debug_export_engine_config, _target| {
                dialogs::export::filechooser_export_engine_config(&appwindow, &appwindow.active_tab().canvas());
            }),
        );

        // Doc layout
        action_doc_layout.connect_activate(
            clone!(@weak self as appwindow => move |action_doc_layout, target| {
                let doc_layout = target.unwrap().str().unwrap();
                let canvas = appwindow.active_tab().canvas();
                let prev_layout = canvas.engine().borrow().document.layout;
                action_doc_layout.set_state(&doc_layout.to_variant());

                let doc_layout = match doc_layout {
                    "fixed-size" => {
                        Layout::FixedSize
                    },
                    "continuous-vertical" => {
                        Layout::ContinuousVertical
                    },
                    "semi-infinite" => {
                        Layout::SemiInfinite
                    },
                    "infinite" => {
                        Layout::Infinite
                    },
                    other => {
                        log::error!("doc-layout action activated with invalid target string: {other}");
                        unimplemented!()
                    }
                };

                appwindow.mainheader().fixedsize_quickactions_box().set_visible(doc_layout == Layout::FixedSize);

                if prev_layout != doc_layout {
                    canvas.engine().borrow_mut().document.layout = doc_layout;
                    canvas.engine().borrow_mut().resize_to_fit_strokes();
                } else {
                    canvas.engine().borrow_mut().resize_autoexpand();
                }
                canvas.update_engine_rendering();
            }));

        // Pen sounds
        action_pen_sounds.connect_change_state(
            clone!(@weak self as appwindow => move |action_pen_sounds, state_request| {
                let pen_sounds = state_request.unwrap().get::<bool>().unwrap();

                appwindow.active_tab().canvas().engine().borrow_mut().set_pen_sounds(pen_sounds, Some(PathBuf::from(config::PKGDATADIR)));

                action_pen_sounds.set_state(&pen_sounds.to_variant());
            }),
        );

        // Format borders
        action_format_borders.connect_change_state(
            clone!(@weak self as appwindow => move |action_format_borders, state_request| {
                let format_borders = state_request.unwrap().get::<bool>().unwrap();
                let canvas = appwindow.active_tab().canvas();

                canvas.engine().borrow_mut().document.format.show_borders = format_borders;
                canvas.queue_draw();

                action_format_borders.set_state(&format_borders.to_variant());
            }),
        );

        // Pen style
        action_pen_style.connect_activate(
            clone!(@weak self as appwindow => move |action, target| {
                let new_pen_style = target.unwrap().get::<PenStyle>().unwrap();
                action.set_state(&new_pen_style.to_variant());
                let canvas = appwindow.active_tab().canvas();

                // don't change the style if the current style with override is already the same (e.g. when switched to from the pen button, not by clicking the pen page)
                if new_pen_style != canvas.engine().borrow().penholder.current_pen_style_w_override() {
                    let mut widget_flags = canvas.engine().borrow_mut().change_pen_style(
                        new_pen_style,
                    );
                    widget_flags.merge(canvas.engine().borrow_mut().change_pen_style_override(
                        None,
                    ));

                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            }),
        );

        // Tab actions
        action_active_tab_move_left.connect_activate(
            clone!(@weak self as appwindow => move |_, _| {
                let active_tab_page = appwindow.active_tab_page();
                appwindow.overlays().tabview().reorder_backward(&active_tab_page);
            }),
        );
        action_active_tab_move_right.connect_activate(
            clone!(@weak self as appwindow => move |_, _| {
                let active_tab_page = appwindow.active_tab_page();
                appwindow.overlays().tabview().reorder_forward(&active_tab_page);
            }),
        );
        action_active_tab_close.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            let active_tab_page = appwindow.active_tab_page();
            if appwindow.overlays().tabview().n_pages() <= 1 {
                // If there is only one tab left, request to close the entire window.
                appwindow.close();
            } else {
                appwindow.overlays().tabview().close_page(&active_tab_page);
            }
        }));

        // Drawing pad buttons
        action_drawing_pad_pressed_button_0.connect_activate(
            clone!(@weak self as appwindow => move |_, _| {
                log::debug!("drawing pad pressed button 0");
                let canvas = appwindow.active_tab().canvas();
                let widget_flags = canvas.engine().borrow_mut().handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton0, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        action_drawing_pad_pressed_button_1.connect_activate(
            clone!(@weak self as appwindow => move |_, _| {
                log::debug!("drawing pad pressed button 1");
                let canvas = appwindow.active_tab().canvas();
                let widget_flags = canvas.engine().borrow_mut().handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton1, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        action_drawing_pad_pressed_button_2.connect_activate(
            clone!(@weak self as appwindow => move |_, _| {
                log::debug!("drawing pad pressed button 2");
                let canvas = appwindow.active_tab().canvas();
                let widget_flags = canvas.engine().borrow_mut().handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton2, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        action_drawing_pad_pressed_button_3.connect_activate(
            clone!(@weak self as appwindow => move |_, _| {
                log::debug!("drawing pad pressed button 3");
                let canvas = appwindow.active_tab().canvas();
                let widget_flags = canvas.engine().borrow_mut().handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton3, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        // Trash Selection
        action_selection_trash.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_trash, _| {
                let canvas = appwindow.active_tab().canvas();

                let mut widget_flags = canvas.engine().borrow_mut().record(Instant::now());
                let selection_keys = canvas.engine().borrow().store.selection_keys_as_rendered();
                canvas.engine().borrow_mut().store.set_trashed_keys(&selection_keys, true);
                widget_flags.merge(canvas.engine().borrow_mut().update_state_current_pen());
                canvas.engine().borrow_mut().resize_autoexpand();
                canvas.update_engine_rendering();

                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        // Duplicate Selection
        action_selection_duplicate.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_duplicate, _| {
                let canvas = appwindow.active_tab().canvas();

                let mut widget_flags = canvas.engine().borrow_mut().record(Instant::now());
                let new_selected = canvas.engine().borrow_mut().store.duplicate_selection();
                canvas.engine().borrow_mut().store.update_geometry_for_strokes(&new_selected);
                widget_flags.merge(canvas.engine().borrow_mut().update_state_current_pen());
                canvas.engine().borrow_mut().resize_autoexpand();
                canvas.update_engine_rendering();

                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        // select all strokes
        action_selection_select_all.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_select_all, _| {
                let canvas = appwindow.active_tab().canvas();

                let mut widget_flags = canvas.engine().borrow_mut().record(Instant::now());
                let all_strokes = canvas.engine().borrow().store.stroke_keys_as_rendered();
                canvas.engine().borrow_mut().store.set_selected_keys(&all_strokes, true);
                widget_flags.merge(canvas.engine().borrow_mut().change_pen_style(PenStyle::Selector));
                widget_flags.merge(canvas.engine().borrow_mut().update_state_current_pen());
                canvas.engine().borrow_mut().resize_autoexpand();
                canvas.update_engine_rendering();

                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        // deselect all strokes
        action_selection_deselect_all.connect_activate(
            clone!(@weak self as appwindow => move |_action_selection_deselect_all, _| {
                let canvas = appwindow.active_tab().canvas();

                let mut widget_flags = canvas.engine().borrow_mut().record(Instant::now());
                let all_strokes = canvas.engine().borrow().store.selection_keys_as_rendered();
                canvas.engine().borrow_mut().store.set_selected_keys(&all_strokes, false);
                widget_flags.merge(canvas.engine().borrow_mut().change_pen_style(PenStyle::Selector));
                widget_flags.merge(canvas.engine().borrow_mut().update_state_current_pen());
                canvas.engine().borrow_mut().resize_autoexpand();
                canvas.update_engine_rendering();

                appwindow.handle_widget_flags(widget_flags, &canvas);
            }),
        );

        // Clear doc
        action_clear_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_clear_doc(&appwindow, &appwindow.active_tab().canvas());
        }));

        // Undo stroke
        action_undo_stroke.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvas = appwindow.active_tab().canvas();

            let widget_flags = canvas.engine().borrow_mut().undo(Instant::now());
            canvas.update_engine_rendering();

            appwindow.handle_widget_flags(widget_flags, &canvas);
        }));

        // Redo stroke
        action_redo_stroke.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvas = appwindow.active_tab().canvas();

            let widget_flags = canvas.engine().borrow_mut().redo(Instant::now());
            canvas.update_engine_rendering();

            appwindow.handle_widget_flags(widget_flags, &canvas);
        }));

        // Zoom reset
        action_zoom_reset.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvas = appwindow.active_tab().canvas();

            let new_zoom = Camera::ZOOM_DEFAULT;
            let current_doc_center = canvas.current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            canvas.center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom fit to width
        action_zoom_fit_width.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvaswrapper = appwindow.active_tab();

            let new_zoom = f64::from(canvaswrapper.scroller().width()) / (canvaswrapper.canvas().engine().borrow().document.format.width + 2.0 * RnCanvasLayout::OVERSHOOT_HORIZONTAL);
            let current_doc_center = canvaswrapper.canvas().current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            canvaswrapper.canvas().center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom in
        action_zoomin.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvas = appwindow.active_tab().canvas();

            let new_zoom = canvas.engine().borrow().camera.total_zoom() * (1.0 + RnCanvas::ZOOM_STEP);
            let current_doc_center = canvas.current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            canvas.center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom out
        action_zoomout.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvas = appwindow.active_tab().canvas();

            let new_zoom = canvas.engine().borrow().camera.total_zoom() * (1.0 - RnCanvas::ZOOM_STEP);
            let current_doc_center = canvas.current_center_on_doc();
            adw::prelude::ActionGroupExt::activate_action(&appwindow, "zoom-to-value", Some(&new_zoom.to_variant()));
            canvas.center_around_coord_on_doc(current_doc_center);
        }));

        // Zoom to value
        action_zoom_to_value.connect_activate(
            clone!(@weak self as appwindow => move |_action_zoom_to_value, target| {
                let new_zoom = target.unwrap().get::<f64>().unwrap().clamp(Camera::ZOOM_MIN, Camera::ZOOM_MAX);
                appwindow.active_tab().canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom);
            }));

        // Add page to doc in fixed size mode
        action_add_page_to_doc.connect_activate(
            clone!(@weak self as appwindow => move |_action_add_page_to_doc, _target| {
                let canvas = appwindow.active_tab().canvas();

                let format_height = canvas.engine().borrow().document.format.height;
                let new_doc_height = canvas.engine().borrow().document.height + format_height;
                canvas.engine().borrow_mut().document.height = new_doc_height;

                canvas.update_engine_rendering();
            }),
        );

        // Resize to fit strokes
        action_resize_to_fit_strokes.connect_activate(
            clone!(@weak self as appwindow => move |_action_resize_to_fit_strokes, _target| {
                let canvas = appwindow.active_tab().canvas();

                canvas.engine().borrow_mut().resize_to_fit_strokes();
                canvas.update_engine_rendering();
            }),
        );

        // Return to the origin page
        action_return_origin_page.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvas = appwindow.active_tab().canvas();

            canvas.return_to_origin_page();
            canvas.engine().borrow_mut().resize_autoexpand();
            canvas.update_engine_rendering();
        }));

        // New doc
        action_new_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::dialog_new_doc(&appwindow, &appwindow.active_tab().canvas());
        }));

        // Open doc
        action_open_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::import::filechooser_open_doc(&appwindow);
        }));

        // Save doc
        action_save_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                let canvas = appwindow.active_tab().canvas();

                if let Some(output_file) = canvas.output_file() {
                    appwindow.overlays().start_pulsing_progressbar();

                    if let Err(e) = canvas.save_document_to_file(&output_file).await {
                        canvas.set_output_file(None);

                        log::error!("saving document failed, Error: `{e:?}`");
                        appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed"));
                    }

                    appwindow.overlays().finish_progressbar();
                    // No success toast on saving without dialog, success is already indicated in the header title
                } else {
                    // Open a dialog to choose a save location
                    dialogs::export::filechooser_save_doc_as(&appwindow, &canvas);
                }
            }));
        }));

        // Save doc as
        action_save_doc_as.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            dialogs::export::filechooser_save_doc_as(&appwindow, &appwindow.active_tab().canvas());
        }));

        // Print doc
        action_print_doc.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            let canvas = appwindow.active_tab().canvas();

            let pages_bounds = canvas.engine().borrow().pages_bounds_w_content();
            let n_pages = pages_bounds.len();
            let engine_snapshot = canvas.engine().borrow().take_snapshot();
            let doc_bounds = engine_snapshot.document.bounds();
            let format_size = na::vector![engine_snapshot.document.format.width, engine_snapshot.document.format.height];

            // TODO: Exposing this as a setting in the print dialog
            let with_background = true;

            appwindow.overlays().start_pulsing_progressbar();

            let background_svg = if with_background {
                canvas.engine().borrow().document
                    .background
                    .gen_svg(doc_bounds, true)
                    .map_err(|e| {
                        log::error!(
                            "background.gen_svg() failed in in the print document action, with Err: {e:?}"
                        )
                    })
                    .ok()
            } else {
                None
            };

            let print_op = PrintOperation::builder()
                .unit(Unit::None)
                .build();

            print_op.connect_begin_print(clone!(@weak appwindow => move |print_op, _print_cx| {
                print_op.set_n_pages(n_pages as i32);
            }));


            print_op.connect_draw_page(clone!(@weak appwindow, @weak canvas => move |_print_op, print_cx, page_no| {
                if let Err(e) = || -> anyhow::Result<()> {
                    let page_bounds = pages_bounds[page_no as usize];

                    let page_strokes = canvas.engine().borrow()
                        .store
                        .stroke_keys_as_rendered_intersecting_bounds(page_bounds);

                    let print_zoom = {
                        let width_scale = print_cx.width() / format_size[0];
                        let height_scale = print_cx.height() / format_size[1];

                        width_scale.min(height_scale)
                    };

                    let cairo_cx = print_cx.cairo_context();

                    cairo_cx.scale(print_zoom, print_zoom);

                    // Clip everything outside page bounds
                    cairo_cx.rectangle(
                        0.0,
                        0.0,
                        page_bounds.extents()[0],
                        page_bounds.extents()[1]
                    );
                    cairo_cx.clip();

                    cairo_cx.save()?;
                    cairo_cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);

                    // We can't render the background svg with piet, so we have to do it with cairo.
                    if let Some(background_svg) = background_svg.to_owned() {
                        render::Svg::draw_svgs_to_cairo_context(
                            &[background_svg],
                            &cairo_cx,
                        )?;
                    }
                    cairo_cx.restore()?;

                    // Draw the strokes with piet
                    let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

                    piet_cx.transform(kurbo::Affine::translate(
                        -page_bounds.mins.coords.to_kurbo_vec(),
                    ));

                    for stroke in page_strokes.into_iter() {
                        if let Some(stroke) = engine_snapshot.stroke_components.get(stroke) {
                            stroke.draw(&mut piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                        }
                    }

                    piet_cx.finish().map_err(|e| anyhow::anyhow!("piet_cx finish() failed while printing page {page_no}, with Err: {e:?}"))
                }() {
                    log::error!("draw_page() failed while printing page: {page_no}, Err: {e:?}");
                }
            }));

            print_op.connect_status_changed(clone!(@weak appwindow => move |print_op| {
                log::debug!("{:?}", print_op.status());
                match print_op.status() {
                    PrintStatus::Finished => {
                        appwindow.overlays().dispatch_toast_text(&gettext("Printed document successfully"));
                    }
                    _ => {}
                }
            }));

            // Run the print op
            if let Err(e) = print_op.run(PrintOperationAction::PrintDialog, Some(&appwindow)){
                log::error!("print_op.run() failed with Err, {e:?}");
                appwindow.overlays().dispatch_toast_error(&gettext("Printing document failed"));
            }

            appwindow.overlays().finish_progressbar();
        }));

        // Import
        action_import_file.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::import::filechooser_import_file(&appwindow);
        }));

        // Export document
        action_export_doc.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::export::dialog_export_doc_w_prefs(&appwindow, &appwindow.active_tab().canvas());
        }));

        // Export document pages
        action_export_doc_pages.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            dialogs::export::dialog_export_doc_pages_w_prefs(&appwindow, &appwindow.active_tab().canvas());
        }));

        // Export selection
        action_export_selection.connect_activate(clone!(@weak self as appwindow => move |_,_| {
            let canvas = appwindow.active_tab().canvas();

            if !canvas.engine().borrow().store.selection_keys_unordered().is_empty() {
                dialogs::export::dialog_export_selection_w_prefs(&appwindow, &appwindow.active_tab().canvas());
            } else {
                appwindow.overlays().dispatch_toast_error(&gettext("Exporting selection failed, nothing selected"));
            }
        }));

        // Clipboard copy
        action_clipboard_copy.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            let canvas = appwindow.active_tab().canvas();

            let (content, widget_flags) = match canvas.engine().borrow().fetch_clipboard_content() {
                Ok((content, widget_flags)) => (content,widget_flags),
                Err(e) => {
                    log::error!("fetch_clipboard_content() failed in clipboard-copy action, Err: {e:?}");
                    return;
                }
            };

            match content {
                Some((data, mime_type)) => {
                    //log::debug!("set clipboard with data: {:02x?}, mime-type: {}", data, mime_type);

                    let content = gdk::ContentProvider::for_bytes(mime_type.as_str(), &glib::Bytes::from_owned(data));

                    if let Err(e) = appwindow.clipboard().set_content(Some(&content)) {
                        log::error!("clipboard set_content() failed in clipboard-copy action, Err: {e:?}");
                    }
                }
                None => {
                    log::debug!("no data available to copy into clipboard");
                }
            }

            appwindow.handle_widget_flags(widget_flags, &canvas);
        }));

        // Clipboard cut
        action_clipboard_cut.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            let canvas = appwindow.active_tab().canvas();

            let (content, widget_flags) = match canvas.engine().borrow_mut().cut_clipboard_content() {
                Ok((content, widget_flags)) => (content,widget_flags),
                Err(e) => {
                    log::error!("cut_clipboard_content() failed in clipboard-cut action, Err: {e:?}");
                    return;
                }
            };

            match content {
                Some((data, mime_type)) => {
                    let content = gdk::ContentProvider::for_bytes(mime_type.as_str(), &glib::Bytes::from_owned(data));

                    if let Err(e) = appwindow.clipboard().set_content(Some(&content)) {
                        log::error!("clipboard set_content() failed in clipboard-cut action, Err: {e:?}");
                    }
                }
                None => {
                    log::debug!("no data available to cut into clipboard");
                }
            }

            appwindow.handle_widget_flags(widget_flags, &canvas);
        }));

        // Clipboard paste
        action_clipboard_paste.connect_activate(clone!(@weak self as appwindow => move |_, _| {
            let canvas = appwindow.active_tab().canvas();
            let content_formats = appwindow.clipboard().formats();

            // Order matters here, we want to go from specific -> generic, mostly because `text/plain` is contained in other text based formats
             if content_formats.contain_mime_type("text/uri-list") {
                glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                    log::debug!("recognized clipboard content format: files list");
                    match appwindow.clipboard().read_text_future().await {
                        Ok(Some(text)) => {
                            let file_paths = text.lines().filter_map(|line| {
                                let file_path = if let Ok(path_uri) = url::Url::parse(line) {
                                    path_uri.to_file_path().ok()?
                                } else {
                                    PathBuf::from(&line)
                                };

                                if file_path.exists() {
                                    Some(file_path)
                                } else {
                                    None
                                }
                            }).collect::<Vec<PathBuf>>();

                            for file_path in file_paths {
                                log::debug!("pasting from path: {:?}", file_path);

                                appwindow.open_file_w_dialogs(gio::File::for_path(&file_path), None, true);
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("failed to paste clipboard from path, read_text() failed with Err: {e:?}");

                        }
                    }
                }));
            } else if content_formats.contain_mime_type(RNOTE_STROKE_CONTENT_MIME_TYPE) {
                glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                    log::debug!("recognized clipboard content format: {RNOTE_STROKE_CONTENT_MIME_TYPE}");
                    match appwindow.clipboard().read_future(&[RNOTE_STROKE_CONTENT_MIME_TYPE], glib::PRIORITY_DEFAULT).await {
                        Ok((input_stream, _)) => {
                            let mut acc = Vec::new();
                            loop {
                                match input_stream.read_future(vec![0; CLIPBOARD_INPUT_STREAM_BUFSIZE], glib::PRIORITY_DEFAULT).await {
                                    Ok((mut bytes, n)) => {
                                        if n == 0 {
                                            break;
                                        }
                                        acc.append(&mut bytes);
                                    }
                                    Err(e) => {
                                        log::error!("failed to read clipboard input stream, Err: {e:?}");
                                        acc.clear();
                                        break;
                                    }
                                }
                            }

                            if !acc.is_empty() {
                                match crate::utils::str_from_u8_nul_utf8(&acc) {
                                    Ok(json_string) => {
                                        if let Err(e) = canvas.insert_stroke_content(json_string.to_string()).await {
                                            log::error!("failed to paste clipboard, Err: {e:?}");
                                        }
                                    }
                                    Err(e) => log::error!("failed to read &str from clipboard data, Err: {e:?}"),
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("failed to paste clipboard as {RNOTE_STROKE_CONTENT_MIME_TYPE}, read_future() failed with Err: {e:?}");
                        }
                    };
                }));
            } else if content_formats.contain_mime_type("image/svg+xml") {
                glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                    log::debug!("recognized clipboard content: svg image");
                    match appwindow.clipboard().read_future(&["image/svg+xml"], glib::PRIORITY_DEFAULT).await {
                        Ok((input_stream, _)) => {
                            let mut acc = Vec::new();
                            loop {
                                match input_stream.read_future(vec![0; CLIPBOARD_INPUT_STREAM_BUFSIZE], glib::PRIORITY_DEFAULT).await {
                                    Ok((mut bytes, n)) => {
                                        if n == 0 {
                                            break;
                                        }
                                        acc.append(&mut bytes);
                                    }
                                    Err(e) => {
                                        log::error!("failed to read clipboard input stream, Err: {e:?}");
                                        acc.clear();
                                        break;
                                    }
                                }
                            }

                            if !acc.is_empty() {
                                match crate::utils::str_from_u8_nul_utf8(&acc) {
                                    Ok(text) => {
                                        if let Err(e) = canvas.load_in_vectorimage_bytes(text.as_bytes().to_vec(), None).await {
                                            log::error!("failed to paste clipboard as vector image, load_in_vectorimage_bytes() returned Err: {e:?}");
                                        };
                                    }
                                    Err(e) => log::error!("failed to read &str from clipboard data, Err: {e:?}"),
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("failed to paste clipboard as vector image, read_future() failed with Err: {e:?}");
                        }
                    };
                }));
            } else if content_formats.contain_mime_type("image/png")  ||
                      content_formats.contain_mime_type("image/jpeg") ||
                      content_formats.contain_mime_type("image/jpg")  ||
                      content_formats.contain_mime_type("image/tiff") ||
                      content_formats.contain_mime_type("image/bmp") {
                const MIMES: [&str; 5] = [
                    "image/png",
                    "image/jpeg",
                    "image/jpg",
                    "image/tiff",
                    "image/bmp",
                ];
                if let Some(mime_type) = MIMES.into_iter().find(|&mime| content_formats.contain_mime_type(mime)) {
                    glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                        log::debug!("recognized clipboard content: bitmap image");
                        match appwindow.clipboard().read_texture_future().await {
                            Ok(Some(texture)) => {
                                if let Err(e) = canvas.load_in_bitmapimage_bytes(texture.save_to_png_bytes().to_vec(), None).await {
                                    log::error!("failed to paste clipboard as {mime_type}, load_in_bitmapimage_bytes() returned Err: {e:?}");
                                };
                            }
                            Ok(None) => {}
                            Err(e) => {
                                log::error!("failed to paste clipboard as {mime_type}, read_texture_future() failed with Err: {e:?}");
                            }
                        };
                    }));
                }
            } else if content_formats.contain_mime_type("text/plain") || content_formats.contain_mime_type("text/plain;charset=utf-8"){
                glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                    log::debug!("recognized clipboard content: plain text");
                    match appwindow.clipboard().read_text_future().await {
                        Ok(Some(text)) => {
                            if let Err(e) = canvas.load_in_text(text.to_string(), None) {
                                log::error!("failed to paste clipboard text, Err: {e:?}");
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("failed to paste clipboard text, read_text() failed with Err: {e:?}");

                        }
                    }
                }));
            } else {
                log::debug!("failed to paste clipboard, unsupported mime-types: {:?}", content_formats.mime_types());
            }
        }));
    }

    pub(crate) fn setup_action_accels(&self) {
        let app = self.app();

        app.set_accels_for_action("win.active-tab-close", &["<Ctrl>w"]);
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
        app.set_accels_for_action("win.zoom-in", &["<Ctrl>plus"]);
        app.set_accels_for_action("win.zoom-out", &["<Ctrl>minus"]);
        app.set_accels_for_action("win.import-file", &["<Ctrl>i"]);
        app.set_accels_for_action("win.undo", &["<Ctrl>z"]);
        app.set_accels_for_action("win.redo", &["<Ctrl><Shift>z"]);
        app.set_accels_for_action("win.clipboard-copy", &["<Ctrl>c"]);
        app.set_accels_for_action("win.clipboard-cut", &["<Ctrl>x"]);
        app.set_accels_for_action("win.clipboard-paste", &["<Ctrl>v"]);
        app.set_accels_for_action("win.pen-style::brush", &["<Ctrl>1"]);
        app.set_accels_for_action("win.pen-style::shaper", &["<Ctrl>2"]);
        app.set_accels_for_action("win.pen-style::typewriter", &["<Ctrl>3"]);
        app.set_accels_for_action("win.pen-style::eraser", &["<Ctrl>4"]);
        app.set_accels_for_action("win.pen-style::selector", &["<Ctrl>5"]);
        app.set_accels_for_action("win.pen-style::tools", &["<Ctrl>6"]);

        // shortcuts for devel builds
        if config::PROFILE.to_lowercase().as_str() == "devel" {
            app.set_accels_for_action("win.visual-debug", &["<Ctrl><Shift>v"]);
        }
    }
}
