// Imports
use crate::{RnAppWindow, RnCanvas, config, dialogs};
use gettextrs::gettext;
use gtk4::gio::InputStream;
use gtk4::graphene;
use gtk4::{
    PrintOperation, PrintOperationAction, Unit, UriLauncher, Window, gdk, gio, glib, glib::clone,
    prelude::*,
};
use p2d::bounding_volume::BoundingVolume;
use rnote_compose::SplitOrder;
use rnote_compose::penevent::ShortcutKey;
use rnote_engine::engine::StrokeContent;
use rnote_engine::ext::GraphenePointExt;
use rnote_engine::strokes::resize::{ImageSizeOption, Resize};
use rnote_engine::{Camera, Engine};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, error};

const CLIPBOARD_INPUT_STREAM_BUFSIZE: usize = 4096;

impl RnAppWindow {
    /// Boolean actions have no target, and a boolean state. They have a default implementation for the activate signal,
    /// which requests the state to be inverted, and the default implementation for change_state, which sets the state
    /// to the request.
    ///
    /// We generally want to connect to the change_state signal. (but then have to set the state with
    /// `action.set_state()`)
    ///
    /// We can then either toggle the state through activating the action, or set the state explicitly through
    /// `action.change_state(<request>)`
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
        let action_toggle_overview = gio::SimpleAction::new("toggle-overview", None);
        self.add_action(&action_toggle_overview);
        let action_devel_menu = gio::SimpleAction::new("devel-menu", None);
        self.add_action(&action_devel_menu);
        let action_new_tab = gio::SimpleAction::new("new-tab", None);
        self.add_action(&action_new_tab);
        let action_debug_export_engine_state =
            gio::SimpleAction::new("debug-export-engine-state", None);
        self.add_action(&action_debug_export_engine_state);
        let action_debug_export_engine_config =
            gio::SimpleAction::new("debug-export-engine-config", None);
        self.add_action(&action_debug_export_engine_config);
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
        let action_add_page_to_doc = gio::SimpleAction::new("add-page-to-doc", None);
        self.add_action(&action_add_page_to_doc);
        let action_remove_page_from_doc = gio::SimpleAction::new("remove-page-from-doc", None);
        self.add_action(&action_remove_page_from_doc);
        let action_resize_to_fit_content = gio::SimpleAction::new("resize-to-fit-content", None);
        self.add_action(&action_resize_to_fit_content);
        let action_return_origin_page = gio::SimpleAction::new("return-origin-page", None);
        self.add_action(&action_return_origin_page);
        let action_selection_trash = gio::SimpleAction::new("selection-trash", None);
        self.add_action(&action_selection_trash);
        let action_selection_duplicate = gio::SimpleAction::new("selection-duplicate", None);
        self.add_action(&action_selection_duplicate);
        let action_selection_invert_color = gio::SimpleAction::new("selection-invert-color", None);
        self.add_action(&action_selection_invert_color);
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
        let action_clipboard_paste_contextmenu =
            gio::SimpleAction::new("clipboard-paste-contextmenu", None);
        self.add_action(&action_clipboard_paste_contextmenu);
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

        // Property Actions
        let action_righthanded = gio::PropertyAction::new("righthanded", self, "righthanded");
        self.add_action(&action_righthanded);
        let action_touch_drawing = gio::PropertyAction::new("touch-drawing", self, "touch-drawing");
        self.add_action(&action_touch_drawing);
        let action_focus_mode = gio::PropertyAction::new("focus-mode", self, "focus-mode");
        self.add_action(&action_focus_mode);
        let action_pen_sounds = gio::PropertyAction::new("pen-sounds", self, "pen-sounds");
        self.add_action(&action_pen_sounds);
        let action_snap_positions =
            gio::PropertyAction::new("snap-positions", self, "snap-positions");
        self.add_action(&action_snap_positions);
        let action_block_pinch_zoom =
            gio::PropertyAction::new("block-pinch-zoom", self, "block-pinch-zoom");
        self.add_action(&action_block_pinch_zoom);
        let action_respect_borders =
            gio::PropertyAction::new("respect-borders", self, "respect-borders");
        self.add_action(&action_respect_borders);
        let action_autosave = gio::PropertyAction::new("autosave", self, "autosave");
        self.add_action(&action_autosave);
        let action_devel_mode = gio::PropertyAction::new("devel-mode", self, "devel-mode");
        self.add_action(&action_devel_mode);
        let action_visual_debug = gio::PropertyAction::new("visual-debug", self, "visual-debug");
        self.add_action(&action_visual_debug);

        // Open settings
        action_open_settings.connect_activate(clone!(
            #[weak(rename_to = appwindow)]
            self,
            move |_, _| {
                appwindow
                    .sidebar()
                    .sidebar_stack()
                    .set_visible_child_name("settings_page");
                appwindow.split_view().set_show_sidebar(true);
            }
        ));

        // About Dialog
        action_about.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                dialogs::dialog_about(&appwindow);
            }
        ));

        // Donate
        action_donate.connect_activate(clone!(move |_, _| {
            UriLauncher::new(config::APP_DONATE_URL).launch(
                None::<&Window>,
                gio::Cancellable::NONE,
                |res| {
                    if let Err(e) = res {
                        error!("Launching donate URL failed, Err: {e:?}");
                    }
                },
            )
        }));

        // Keyboard shortcuts
        action_keyboard_shortcuts_dialog.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                dialogs::dialog_keyboard_shortcuts(&appwindow);
            }
        ));

        // Open Canvas Menu
        action_open_canvasmenu.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                if appwindow.split_view().shows_sidebar() && appwindow.split_view().is_collapsed() {
                    appwindow.split_view().set_show_sidebar(false);
                }
                appwindow.main_header().canvasmenu().popovermenu().popup();
            }
        ));

        // Open App Menu
        action_open_appmenu.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                if !appwindow.split_view().shows_sidebar() {
                    appwindow.main_header().appmenu().popovermenu().popup();
                    return;
                }
                if appwindow.split_view().is_collapsed() {
                    appwindow.split_view().set_show_sidebar(false);
                    appwindow.main_header().appmenu().popovermenu().popup();
                } else {
                    appwindow.sidebar().appmenu().popovermenu().popup();
                }
            }
        ));

        // Toggle Tabs Overview
        action_toggle_overview.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let overview = appwindow.overview();
                overview.set_open(!overview.is_open());
            }
        ));

        // Developer settings
        // Its enabled state toggles the visibility of the developer settings menu entry.
        action_devel_menu.set_enabled(false);

        // Create page
        action_new_tab.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let wrapper = appwindow.new_canvas_wrapper();
                appwindow.append_wrapper_new_tab(&wrapper);
            }
        ));

        // Export engine state
        action_debug_export_engine_state.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        dialogs::export::filechooser_export_engine_state(&appwindow, &canvas).await;
                    }
                ));
            }
        ));

        // Export engine config
        action_debug_export_engine_config.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        dialogs::export::filechooser_export_engine_config(&appwindow, &canvas)
                            .await;
                    }
                ));
            }
        ));

        // Tab actions
        action_active_tab_move_left.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(active_tab_page) = appwindow.active_tab_page() else {
                    return;
                };
                appwindow
                    .overlays()
                    .tabview()
                    .reorder_backward(&active_tab_page);
            }
        ));
        action_active_tab_move_right.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(active_tab_page) = appwindow.active_tab_page() else {
                    return;
                };
                appwindow
                    .overlays()
                    .tabview()
                    .reorder_forward(&active_tab_page);
            }
        ));
        action_active_tab_close.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(active_tab_page) = appwindow.active_tab_page() else {
                    return;
                };
                if appwindow.overlays().tabview().n_pages() <= 1 {
                    // If there is only one tab left, request to close the entire window.
                    appwindow.close();
                } else {
                    appwindow.close_tab_request(&active_tab_page);
                }
            }
        ));

        // Drawing pad buttons
        action_drawing_pad_pressed_button_0.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                debug!("Pressed drawing pad button 0");
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let (_, widget_flags) = canvas
                    .engine_mut()
                    .handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton0, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        action_drawing_pad_pressed_button_1.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                debug!("Pressed drawing pad button 1");
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let (_, widget_flags) = canvas
                    .engine_mut()
                    .handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton1, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        action_drawing_pad_pressed_button_2.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                debug!("Pressed drawing pad button 2");
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let (_, widget_flags) = canvas
                    .engine_mut()
                    .handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton2, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        action_drawing_pad_pressed_button_3.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                debug!("Pressed drawing pad button 3");
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let (_, widget_flags) = canvas
                    .engine_mut()
                    .handle_pressed_shortcut_key(ShortcutKey::DrawingPadButton3, Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Trash Selection
        action_selection_trash.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().trash_selection();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Duplicate Selection
        action_selection_duplicate.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().duplicate_selection();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // invert color brightness of selection
        action_selection_invert_color.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().invert_selection_colors();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // select all strokes
        action_selection_select_all.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().select_all_strokes();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // deselect all strokes
        action_selection_deselect_all.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().deselect_all_strokes();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Clear doc
        action_clear_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        dialogs::dialog_clear_doc(&appwindow, &canvas).await;
                    }
                ));
            }
        ));

        // Undo stroke
        action_undo_stroke.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().undo(Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Redo stroke
        action_redo_stroke.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().redo(Instant::now());
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Zoom reset
        action_zoom_reset.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let viewport_center = canvas.engine_ref().camera.viewport_center();
                let new_zoom = Camera::ZOOM_DEFAULT;
                let mut widget_flags = canvas.engine_mut().zoom_w_timeout(new_zoom);
                widget_flags |= canvas
                    .engine_mut()
                    .camera
                    .set_viewport_center(viewport_center);
                appwindow.handle_widget_flags(widget_flags, &canvas)
            }
        ));

        // Zoom fit to width
        action_zoom_fit_width.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(wrapper) = appwindow.active_tab_wrapper() else {
                    return;
                };
                let canvas = wrapper.canvas();
                let viewport_center = canvas.engine_ref().camera.viewport_center();
                let new_zoom = f64::from(wrapper.scroller().width())
                    / (wrapper.canvas().engine_ref().document.config.format.width()
                        + 2.0 * Camera::OVERSHOOT_HORIZONTAL);
                let mut widget_flags = canvas.engine_mut().zoom_w_timeout(new_zoom);
                widget_flags |= canvas
                    .engine_mut()
                    .camera
                    .set_viewport_center(viewport_center);
                appwindow.handle_widget_flags(widget_flags, &canvas)
            }
        ));

        // Zoom in
        action_zoomin.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let viewport_center = canvas.engine_ref().camera.viewport_center();
                let new_zoom =
                    canvas.engine_ref().camera.total_zoom() * (1.0 + RnCanvas::ZOOM_SCROLL_STEP);
                let mut widget_flags = canvas.engine_mut().zoom_w_timeout(new_zoom);
                widget_flags |= canvas
                    .engine_mut()
                    .camera
                    .set_viewport_center(viewport_center);
                appwindow.handle_widget_flags(widget_flags, &canvas)
            }
        ));

        // Zoom out
        action_zoomout.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let viewport_center = canvas.engine_ref().camera.viewport_center();
                let new_zoom = canvas.engine_ref().camera.total_zoom()
                    * (1.0 / (1.0 + RnCanvas::ZOOM_SCROLL_STEP));
                let mut widget_flags = canvas.engine_mut().zoom_w_timeout(new_zoom);
                widget_flags |= canvas
                    .engine_mut()
                    .camera
                    .set_viewport_center(viewport_center);
                appwindow.handle_widget_flags(widget_flags, &canvas)
            }
        ));

        // Add page to doc in fixed size mode
        action_add_page_to_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_action_add_page_to_doc, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().doc_add_page_fixed_size();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Remove page from doc in fixed size mode
        action_remove_page_from_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().doc_remove_page_fixed_size();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Resize to fit content
        action_resize_to_fit_content.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas.engine_mut().doc_resize_to_fit_content();
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // Return to the origin page
        action_return_origin_page.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let widget_flags = canvas
                    .engine_mut()
                    .return_to_origin(canvas.parent().map(|p| p.width() as f64));
                appwindow.handle_widget_flags(widget_flags, &canvas);
            }
        ));

        // New doc
        action_new_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        dialogs::dialog_new_doc(&appwindow, &canvas).await;
                    }
                ));
            }
        ));

        // Open doc
        action_open_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        dialogs::import::filedialog_open_doc(&appwindow).await;
                    }
                ));
            }
        ));

        // Save doc
        action_save_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };

                        if let Some(output_file) = canvas.output_file() {
                            appwindow.overlays().progressbar_start_pulsing();

                            if let Err(e) = canvas.save_document_to_file(&output_file).await {
                                error!("Saving document failed, Err: `{e:?}`");
                                canvas.set_output_file(None);
                                appwindow
                                    .overlays()
                                    .dispatch_toast_error(&gettext("Saving document failed"));
                                appwindow.overlays().progressbar_abort();
                            } else {
                                appwindow.overlays().progressbar_finish();
                                // No success toast on saving without dialog, success is already indicated in the header title
                            }
                        } else {
                            // Open a dialog to choose a save location
                            dialogs::export::dialog_save_doc_as(&appwindow, &canvas).await;
                        }
                    }
                ));
            }
        ));

        // Save doc as
        action_save_doc_as.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        dialogs::export::dialog_save_doc_as(&appwindow, &canvas).await;
                    }
                ));
            }
        ));

        // Print doc
        action_print_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                // TODO: Expose these variables as options in the print dialog
                let draw_background = true;
                let draw_pattern = true;
                let optimize_printing = false;
                let page_order = SplitOrder::default();
                let margin = 0.0;
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let pages_content = canvas.engine_ref().extract_pages_content(page_order);
                let n_pages = pages_content.len();

                appwindow.overlays().progressbar_start_pulsing();

                let print_op = PrintOperation::builder().unit(Unit::None).build();

                print_op.connect_begin_print(clone!(move |print_op, _print_cx| {
                    print_op.set_n_pages(n_pages as i32);
                }));

                print_op.connect_draw_page(clone!(move |_print_op, print_cx, page_no| {
                    let page_content = &pages_content[page_no as usize];
                    let page_bounds = page_content.bounds.unwrap().loosened(margin);
                    let print_scale = (print_cx.width() / page_bounds.extents()[0])
                        .min(print_cx.height() / page_bounds.extents()[1]);
                    let cairo_cx = print_cx.cairo_context();

                    cairo_cx.scale(print_scale, print_scale);
                    cairo_cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);
                    if let Err(e) = page_content.draw_to_cairo(
                        &cairo_cx,
                        draw_background,
                        draw_pattern,
                        optimize_printing,
                        margin,
                        Engine::STROKE_EXPORT_IMAGE_SCALE,
                    ) {
                        error!("Drawing page no: {page_no} while printing failed, Err: {e:?}");
                    }
                }));

                print_op.connect_status_changed(clone!(move |print_op| {
                    debug!(
                        "Print operation status has changed to: {:?}",
                        print_op.status()
                    );
                }));

                // Run the print op
                if let Err(e) = print_op.run(PrintOperationAction::PrintDialog, Some(&appwindow)) {
                    error!("Running print operation failed , Err: {e:?}");
                    appwindow
                        .overlays()
                        .dispatch_toast_error(&gettext("Printing document failed"));
                    appwindow.overlays().progressbar_abort();
                } else {
                    appwindow.overlays().progressbar_finish();
                }
            }
        ));

        // Import
        action_import_file.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        dialogs::import::filedialog_import_file(&appwindow).await;
                    }
                ));
            }
        ));

        // Export document
        action_export_doc.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        dialogs::export::dialog_export_doc_w_prefs(&appwindow, &canvas).await;
                    }
                ));
            }
        ));

        // Export document pages
        action_export_doc_pages.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };
                        dialogs::export::dialog_export_doc_pages_w_prefs(&appwindow, &canvas).await;
                    }
                ));
            }
        ));

        // Export selection
        action_export_selection.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    appwindow,
                    async move {
                        let Some(canvas) = appwindow.active_tab_canvas() else {
                            return;
                        };

                        if !canvas.engine_ref().nothing_selected() {
                            dialogs::export::dialog_export_selection_w_prefs(&appwindow, &canvas)
                                .await;
                        } else {
                            appwindow.overlays().dispatch_toast_error(&gettext(
                                "Exporting selection failed, nothing selected",
                            ));
                        }
                    }
                ));
            }
        ));

        // Clipboard copy
        action_clipboard_copy.connect_activate(clone!(#[weak(rename_to=appwindow)] self, move |_, _| {
            glib::spawn_future_local(clone!(#[weak] appwindow, async move {
                let Some(canvas) = appwindow.active_tab_canvas()
                else {
                    return;
                };
                let receiver = canvas.engine_ref().fetch_clipboard_content();
                let (content, widget_flags) = match receiver.await {
                    Ok(Ok((content, widget_flags))) => (content,widget_flags),
                    Ok(Err(e)) => {
                        error!("Fetching clipboard content failed in clipboard-copy action, Err: {e:?}");
                        return;
                    }
                    Err(e) => {
                        error!("Awaiting fetched clipboard content failed in clipboard-copy action, Err: {e:?}");
                        return;
                    }
                };

                let gdk_content_provider = gdk::ContentProvider::new_union(content.into_iter().map(|(data, mime_type)| {
                    gdk::ContentProvider::for_bytes(mime_type.as_str(), &glib::Bytes::from_owned(data))
                }).collect::<Vec<gdk::ContentProvider>>().as_slice());

                if let Err(e) = appwindow.clipboard().set_content(Some(&gdk_content_provider)) {
                    error!("Set appwindow clipboard content failed in clipboard-copy action, Err: {e:?}");
                }

                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));
        }));

        // Clipboard cut
        action_clipboard_cut.connect_activate(clone!(#[weak(rename_to=appwindow)] self, move |_, _| {
            glib::spawn_future_local(clone!(#[weak] appwindow, async move {
                let Some(canvas) = appwindow.active_tab_canvas()
                else {
                    return;
                };
                let receiver = canvas.engine_mut().cut_clipboard_content();
                let (content, widget_flags) = match receiver.await {
                    Ok(Ok((content, widget_flags))) => (content,widget_flags),
                    Ok(Err(e)) => {
                        error!("Cutting clipboard content failed in clipboard-cut action, Err: {e:?}");
                        return;
                    }
                    Err(e) => {
                        error!("Awaiting cut clipboard content failed in clipboard-cut action, Err: {e:?}");
                        return;
                    }
                };
                let gdk_content_provider = gdk::ContentProvider::new_union(content.into_iter().map(|(data, mime_type)| {
                    gdk::ContentProvider::for_bytes(mime_type.as_str(), &glib::Bytes::from_owned(data))
                }).collect::<Vec<gdk::ContentProvider>>().as_slice());

                if let Err(e) = appwindow.clipboard().set_content(Some(&gdk_content_provider)) {
                    error!("Set appwindow clipboard content failed in clipboard-cut action, Err: {e:?}");
                }

                appwindow.handle_widget_flags(widget_flags, &canvas);
            }));
        }));

        // Clipboard paste
        action_clipboard_paste.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(wrapper) = appwindow.active_tab_wrapper() else {
                    return;
                };
                let canvas = wrapper.canvas();

                let pointer_pos = wrapper.pointer_pos().and_then(|wrapper_point| {
                    let canvas_point = wrapper
                        .compute_point(&canvas, &graphene::Point::from_na_vec(wrapper_point));

                    if let Some(point) = canvas_point {
                        let x = point.x() as f64;
                        let y = point.y() as f64;

                        if canvas.contains(x, y) {
                            let transformed_point =
                                (canvas.engine_ref().camera.transform().inverse()
                                    * na::point![x, y])
                                .coords;

                            return Some(transformed_point);
                        }
                    }

                    None
                });

                appwindow.clipboard_paste(pointer_pos);
            }
        ));

        action_clipboard_paste_contextmenu.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(wrapper) = appwindow.active_tab_wrapper() else {
                    return;
                };
                let canvas = wrapper.canvas();

                let last_contextmenu_pos = wrapper.last_contextmenu_pos().map(|vec2| {
                    let p = graphene::Point::new(vec2.x as f32, vec2.y as f32);
                    (canvas.engine_ref().camera.transform().inverse()
                        * na::point![p.x() as f64, p.y() as f64])
                    .coords
                });

                appwindow.clipboard_paste(last_contextmenu_pos);
            }
        ));
    }

    pub(crate) fn setup_action_accels(&self) {
        let app = self.app();

        app.set_accels_for_action("win.active-tab-close", &["<Ctrl>w"]);
        app.set_accels_for_action("win.fullscreen", &["F11"]);
        app.set_accels_for_action("win.keyboard-shortcuts", &["<Ctrl>question"]);
        app.set_accels_for_action("win.toggle-overview", &["<Ctrl><Shift>o"]);
        app.set_accels_for_action("win.open-canvasmenu", &["F9"]);
        app.set_accels_for_action("win.open-appmenu", &["F10"]);
        app.set_accels_for_action("win.open-doc", &["<Ctrl>o"]);
        app.set_accels_for_action("win.save-doc", &["<Ctrl>s"]);
        app.set_accels_for_action("win.save-doc-as", &["<Ctrl><Shift>s"]);
        app.set_accels_for_action("win.new-tab", &["<Ctrl>t"]);
        app.set_accels_for_action("win.snap-positions", &["<Ctrl><Shift>p"]);
        app.set_accels_for_action("win.clear-doc", &["<Ctrl>l"]);
        app.set_accels_for_action("win.print-doc", &["<Ctrl>p"]);
        app.set_accels_for_action("win.add-page-to-doc", &["<Ctrl><Shift>a"]);
        app.set_accels_for_action("win.remove-page-from-doc", &["<Ctrl><Shift>r"]);
        app.set_accels_for_action(
            "win.zoom-in",
            &["<Ctrl>plus", "<Ctrl>equal", "<Ctrl>KP_Add"],
        );
        app.set_accels_for_action("win.zoom-reset", &["<Ctrl>0", "<Ctrl>KP_0"]);
        app.set_accels_for_action("win.zoom-out", &["<Ctrl>minus", "<Ctrl>KP_Subtract"]);
        app.set_accels_for_action("win.import-file", &["<Ctrl>i"]);
        app.set_accels_for_action("win.undo", &["<Ctrl>z"]);
        app.set_accels_for_action("win.redo", &["<Ctrl><Shift>z"]);
        app.set_accels_for_action("win.clipboard-copy", &["<Ctrl>c"]);
        app.set_accels_for_action("win.clipboard-cut", &["<Ctrl>x"]);
        app.set_accels_for_action("win.clipboard-paste", &["<Ctrl>v"]);
        app.set_accels_for_action("win.pen-style::brush", &["<Ctrl>1", "<Ctrl>KP_1"]);
        app.set_accels_for_action("win.pen-style::shaper", &["<Ctrl>2", "<Ctrl>KP_2"]);
        app.set_accels_for_action("win.pen-style::typewriter", &["<Ctrl>3", "<Ctrl>KP_3"]);
        app.set_accels_for_action("win.pen-style::eraser", &["<Ctrl>4", "<Ctrl>KP_4"]);
        app.set_accels_for_action("win.pen-style::selector", &["<Ctrl>5", "<Ctrl>KP_5"]);
        app.set_accels_for_action("win.pen-style::tools", &["<Ctrl>6", "<Ctrl>KP_6"]);

        // shortcuts for devel build
        if config::PROFILE.to_lowercase().as_str() == "devel" {
            app.set_accels_for_action("win.visual-debug", &["<Ctrl><Shift>v"]);
        }
    }

    fn clipboard_paste(&self, target_pos: Option<na::Vector2<f64>>) {
        let content_formats = self.clipboard().formats();
        let Some(canvas) = self.active_tab_canvas() else {
            return;
        };

        // Order matters here, we want to go from specific -> generic, mostly because `text/plain` is contained in other text based formats
        if content_formats.contain_mime_type("text/uri-list") {
            glib::spawn_future_local(clone!(
                #[weak(rename_to=appwindow)]
                self,
                async move {
                    debug!("Recognized clipboard content format: files list");

                    match appwindow
                        .clipboard()
                        .read_future(&["text/uri-list"], glib::source::Priority::DEFAULT)
                        .await
                    {
                        Ok((input_stream, _)) => {
                            let acc = collect_clipboard_data(input_stream).await;
                            if !acc.is_empty() {
                                match crate::utils::str_from_u8_nul_utf8(&acc) {
                                    Ok(text) => {
                                        debug!("files uri list : {:?}", text);
                                        let file_paths = text
                                            .lines()
                                            .filter_map(|line| {
                                                let file_path =
                                                    if let Ok(path_uri) = url::Url::parse(line) {
                                                        path_uri.to_file_path().ok()?
                                                    } else {
                                                        PathBuf::from(&line)
                                                    };

                                                if file_path.exists() {
                                                    Some(file_path)
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect::<Vec<PathBuf>>();

                                        for file_path in file_paths {
                                            appwindow
                                                .open_file_w_dialogs(
                                                    gio::File::for_path(&file_path),
                                                    target_pos,
                                                    true,
                                                )
                                                .await;
                                        }
                                    }
                                    Err(e) => error!(
                                        "Failed to read `text/uri-list` from clipboard data, Err: {e:?}"
                                    ),
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Reading clipboard failed while pasting as `text/uri-list`, Err: {e:?}",
                            );
                        }
                    }
                }
            ));
        } else if content_formats.contain_mime_type(StrokeContent::MIME_TYPE) {
            glib::spawn_future_local(clone!(
                #[weak]
                canvas,
                #[weak(rename_to=appwindow)]
                self,
                async move {
                    debug!(
                        "Recognized clipboard content format: {}",
                        StrokeContent::MIME_TYPE
                    );

                    match appwindow
                        .clipboard()
                        .read_future(&[StrokeContent::MIME_TYPE], glib::source::Priority::DEFAULT)
                        .await
                    {
                        Ok((input_stream, _)) => {
                            let acc = collect_clipboard_data(input_stream).await;

                            if !acc.is_empty() {
                                match crate::utils::str_from_u8_nul_utf8(&acc) {
                                    Ok(json_string) => {
                                        let resize_argument =
                                            ImageSizeOption::ResizeImage(Resize {
                                                width: canvas
                                                    .engine_ref()
                                                    .document
                                                    .config
                                                    .format
                                                    .width(),
                                                height: canvas
                                                    .engine_ref()
                                                    .document
                                                    .config
                                                    .format
                                                    .height(),
                                                layout_fixed_width: canvas
                                                    .engine_ref()
                                                    .document
                                                    .config
                                                    .layout
                                                    .is_fixed_width(),
                                                max_viewpoint: None,
                                                restrain_to_viewport: false,
                                                respect_borders: appwindow.respect_borders(),
                                            });
                                        if let Err(e) = canvas
                                            .insert_stroke_content(
                                                json_string.to_string(),
                                                resize_argument,
                                                target_pos,
                                            )
                                            .await
                                        {
                                            error!(
                                                "Failed to insert stroke content while pasting as `{}`, Err: {e:?}",
                                                StrokeContent::MIME_TYPE
                                            );
                                        }
                                    }
                                    Err(e) => error!(
                                        "Failed to read stroke content &str from clipboard data, Err: {e:?}"
                                    ),
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Reading clipboard failed while pasting as `{}`, Err: {e:?}",
                                StrokeContent::MIME_TYPE
                            );
                        }
                    };
                }
            ));
        } else if content_formats.contain_mime_type("image/svg+xml") {
            glib::spawn_future_local(clone!(
                #[weak(rename_to=appwindow)]
                self,
                async move {
                    debug!("Recognized clipboard content: svg image");

                    match appwindow
                        .clipboard()
                        .read_future(&["image/svg+xml"], glib::source::Priority::DEFAULT)
                        .await
                    {
                        Ok((input_stream, _)) => {
                            let acc = collect_clipboard_data(input_stream).await;

                            if !acc.is_empty() {
                                match crate::utils::str_from_u8_nul_utf8(&acc) {
                                    Ok(text) => {
                                        if let Err(e) = canvas
                                            .load_in_vectorimage_bytes(
                                                text.as_bytes().to_vec(),
                                                target_pos,
                                                appwindow.respect_borders(),
                                            )
                                            .await
                                        {
                                            error!(
                                                "Loading VectorImage bytes failed while pasting as Svg failed, Err: {e:?}"
                                            );
                                        };
                                    }
                                    Err(e) => error!(
                                        "Failed to get string from clipboard data while pasting as Svg, Err: {e:?}"
                                    ),
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to read clipboard data while pasting as Svg, Err: {e:?}"
                            );
                        }
                    };
                }
            ));
        } else if content_formats.contain_mime_type("image/png")
            || content_formats.contain_mime_type("image/jpeg")
            || content_formats.contain_mime_type("image/jpg")
            || content_formats.contain_mime_type("image/tiff")
            || content_formats.contain_mime_type("image/bmp")
        {
            const MIMES: [&str; 5] = [
                "image/png",
                "image/jpeg",
                "image/jpg",
                "image/tiff",
                "image/bmp",
            ];
            if let Some(mime_type) = MIMES
                .into_iter()
                .find(|&mime| content_formats.contain_mime_type(mime))
            {
                glib::spawn_future_local(clone!(
                    #[weak]
                    canvas,
                    #[weak(rename_to=appwindow)]
                    self,
                    async move {
                        debug!("Recognized clipboard content: bitmap image");

                        match appwindow.clipboard().read_texture_future().await {
                            Ok(Some(texture)) => {
                                if let Err(e) = canvas
                                    .load_in_bitmapimage_bytes(
                                        texture.save_to_png_bytes().to_vec(),
                                        target_pos,
                                        appwindow.respect_borders(),
                                    )
                                    .await
                                {
                                    error!(
                                        "Loading bitmap image bytes failed while pasting clipboard as {mime_type}, Err: {e:?}"
                                    );
                                };
                            }
                            Ok(None) => {}
                            Err(e) => {
                                error!(
                                    "Reading clipboard text failed while pasting clipboard as {mime_type}, Err: {e:?}"
                                );
                            }
                        };
                    }
                ));
            }
        } else if content_formats.contain_mime_type("text/plain")
            || content_formats.contain_mime_type("text/plain;charset=utf-8")
        {
            glib::spawn_future_local(clone!(
                #[weak]
                canvas,
                #[weak(rename_to=appwindow)]
                self,
                async move {
                    debug!("Recognized clipboard content: plain text");

                    match appwindow.clipboard().read_text_future().await {
                        Ok(Some(text)) => {
                            if let Err(e) = canvas.load_in_text(text.to_string(), target_pos) {
                                error!("Failed to paste clipboard text, Err: {e:?}");
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            error!(
                                "Reading clipboard text failed while pasting clipboard as plain text, Err: {e:?}"
                            );
                        }
                    }
                }
            ));
        } else {
            debug!(
                "Failed to paste clipboard, unsupported MIME-type(s): {:?}",
                content_formats.mime_types()
            );
        }
    }
}

async fn collect_clipboard_data(input_stream: InputStream) -> Vec<u8> {
    let mut acc = Vec::new();
    loop {
        match input_stream
            .read_future(
                vec![0; CLIPBOARD_INPUT_STREAM_BUFSIZE],
                glib::source::Priority::DEFAULT,
            )
            .await
        {
            Ok((mut bytes, n)) => {
                if n == 0 {
                    break;
                }
                acc.append(&mut bytes);
            }
            Err(e) => {
                error!("Failed to read clipboard input stream, Err: {e:?}");
                acc.clear();
                break;
            }
        }
    }
    acc
}
