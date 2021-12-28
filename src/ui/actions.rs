use std::{cell::Cell, rc::Rc};

use crate::{
    app::RnoteApp,
    pens::{selector, shaper, PenStyle},
    render,
    ui::appwindow::RnoteAppWindow,
    ui::{canvas::Canvas, dialogs},
};
use gtk4::{
    gdk, gio, glib, glib::clone, prelude::*, ArrowType, CornerType, PackType, PositionType,
    PrintOperation, PrintOperationAction, Unit,
};

/* Actions follow this principle:
without any state: the activation triggers the callback
with boolean state: They have a boolean parameter, and a boolean state. activating the action can be done with activate_action() with the desired state.
    A state change can also be directly requested with change_action_state( somebool ).
for other stateful actions: They have the same values as their state as their parameters. Activating the action with a parameter is equivalent to changing its state directly
*/
pub fn setup_actions(appwindow: &RnoteAppWindow) {
    let app = appwindow
        .application()
        .unwrap()
        .downcast::<RnoteApp>()
        .unwrap();

    let action_quit = gio::SimpleAction::new("quit", None);
    app.add_action(&action_quit);
    let action_close_active = gio::SimpleAction::new("close-active", None);
    appwindow.add_action(&action_close_active);
    let action_about = gio::SimpleAction::new("about", None);
    appwindow.add_action(&action_about);
    let action_keyboard_shortcuts_dialog = gio::SimpleAction::new("keyboard-shortcuts", None);
    appwindow.add_action(&action_keyboard_shortcuts_dialog);
    let action_open_canvasmenu = gio::SimpleAction::new("open-canvasmenu", None);
    appwindow.add_action(&action_open_canvasmenu);
    let action_open_appmenu = gio::SimpleAction::new("open-appmenu", None);
    appwindow.add_action(&action_open_appmenu);
    let action_warning =
        gio::SimpleAction::new("warning", Some(&glib::VariantType::new("s").unwrap()));
    appwindow.add_action(&action_warning);
    let action_error = gio::SimpleAction::new("error", Some(&glib::VariantType::new("s").unwrap()));
    appwindow.add_action(&action_error);
    let action_devel_settings = gio::SimpleAction::new("devel-settings", None);
    app.add_action(&action_devel_settings);
    let action_clear_sheet = gio::SimpleAction::new("clear-sheet", None);
    appwindow.add_action(&action_clear_sheet);
    let action_undo_stroke = gio::SimpleAction::new("undo-stroke", None);
    appwindow.add_action(&action_undo_stroke);
    let action_redo_stroke = gio::SimpleAction::new("redo-stroke", None);
    appwindow.add_action(&action_redo_stroke);
    let action_zoom_reset = gio::SimpleAction::new("zoom-reset", None);
    appwindow.add_action(&action_zoom_reset);
    let action_zoom_fit_width = gio::SimpleAction::new("zoom-fit-width", None);
    appwindow.add_action(&action_zoom_fit_width);
    let action_zoomin = gio::SimpleAction::new("zoom-in", None);
    appwindow.add_action(&action_zoomin);
    let action_zoomout = gio::SimpleAction::new("zoom-out", None);
    appwindow.add_action(&action_zoomout);
    let action_delete_selection = gio::SimpleAction::new("delete-selection", None);
    appwindow.add_action(&action_delete_selection);
    let action_duplicate_selection = gio::SimpleAction::new("duplicate-selection", None);
    appwindow.add_action(&action_duplicate_selection);
    let action_new_sheet = gio::SimpleAction::new("new-sheet", None);
    appwindow.add_action(&action_new_sheet);
    let action_save_sheet = gio::SimpleAction::new("save-sheet", None);
    appwindow.add_action(&action_save_sheet);
    let action_save_sheet_as = gio::SimpleAction::new("save-sheet-as", None);
    appwindow.add_action(&action_save_sheet_as);
    let action_open_sheet = gio::SimpleAction::new("open-sheet", None);
    appwindow.add_action(&action_open_sheet);
    let action_open_workspace = gio::SimpleAction::new("open-workspace", None);
    appwindow.add_action(&action_open_workspace);
    let action_print_sheet = gio::SimpleAction::new("print-sheet", None);
    appwindow.add_action(&action_print_sheet);
    let action_import_file = gio::SimpleAction::new("import-file", None);
    appwindow.add_action(&action_import_file);
    let action_export_selection_as_svg = gio::SimpleAction::new("export-selection-as-svg", None);
    appwindow.add_action(&action_export_selection_as_svg);
    let action_export_sheet_as_svg = gio::SimpleAction::new("export-sheet-as-svg", None);
    appwindow.add_action(&action_export_sheet_as_svg);
    let action_clipboard_copy_selection = gio::SimpleAction::new("clipboard-copy-selection", None);
    appwindow.add_action(&action_clipboard_copy_selection);
    let action_clipboard_paste_selection =
        gio::SimpleAction::new("clipboard-paste-selection", None);
    appwindow.add_action(&action_clipboard_paste_selection);

    let action_tmperaser = gio::SimpleAction::new_stateful(
        "tmperaser",
        Some(&glib::VariantType::new("b").unwrap()),
        &false.to_variant(),
    );
    appwindow.add_action(&action_tmperaser);
    let action_current_pen = gio::SimpleAction::new_stateful(
        "current-pen",
        Some(&glib::VariantType::new("s").unwrap()),
        &"marker".to_variant(),
    );
    appwindow.add_action(&action_current_pen);
    let action_current_shape = gio::SimpleAction::new_stateful(
        "current-shape",
        Some(&glib::VariantType::new("s").unwrap()),
        &"rectangle".to_variant(),
    );
    appwindow.add_action(&action_current_shape);
    let action_shaper_drawstyle = gio::SimpleAction::new_stateful(
        "shaper-drawstyle",
        Some(&glib::VariantType::new("s").unwrap()),
        &"smooth".to_variant(),
    );
    appwindow.add_action(&action_shaper_drawstyle);
    let action_selector_style = gio::SimpleAction::new_stateful(
        "selector-style",
        Some(&glib::VariantType::new("s").unwrap()),
        &"polygon".to_variant(),
    );
    appwindow.add_action(&action_selector_style);

    let action_devel = appwindow.app_settings().create_action("devel");
    app.add_action(&action_devel);
    let action_visual_debug = appwindow.app_settings().create_action("visual-debug");
    app.add_action(&action_visual_debug);
    let action_renderer_backend = appwindow.app_settings().create_action("renderer-backend");
    app.add_action(&action_renderer_backend);
    let action_touch_drawing = appwindow.app_settings().create_action("touch-drawing");
    app.add_action(&action_touch_drawing);
    let action_sheet_format_borders = appwindow.app_settings().create_action("format-borders");
    app.add_action(&action_sheet_format_borders);
    let action_endless_sheet = appwindow.app_settings().create_action("endless-sheet");
    app.add_action(&action_endless_sheet);
    let action_righthanded = appwindow.app_settings().create_action("righthanded");
    app.add_action(&action_righthanded);

    // Quit App
    action_quit.connect_activate(clone!(@weak appwindow => move |_, _| {
        appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().quit();
    }));

    // Close active window
    action_close_active.connect_activate(clone!(@weak appwindow => move |_, _| {
        appwindow.close();
    }));

    // About Dialog
    action_about.connect_activate(clone!(@weak appwindow => move |_, _| {
        dialogs::dialog_about(&appwindow);
    }));

    // Keyboard shortcuts
    action_keyboard_shortcuts_dialog.connect_activate(
        clone!(@weak appwindow => move |_action_keyboard_shortcuts_dialog, _parameter| {
            dialogs::dialog_keyboard_shortcuts(&appwindow);
        }),
    );

    // Open Canvas Menu
    action_open_canvasmenu.connect_activate(clone!(@weak appwindow => move |_,_| {
        appwindow.mainheader().canvasmenu().popovermenu().popup();
    }));

    // Open App Menu
    action_open_appmenu.connect_activate(clone!(@weak appwindow => move |_,_| {
        appwindow.mainheader().appmenu().popovermenu().popup();
    }));

    // Warning
    action_warning.connect_activate(
        clone!(@weak appwindow => move |_action_warning, parameter| {
             let warning = parameter.unwrap().get::<String>().unwrap();
            log::warn!("{}", warning);
        }),
    );

    // Error
    action_error.connect_activate(clone!(@weak appwindow => move |_action_error, parameter| {
         let error = parameter.unwrap().get::<String>().unwrap();
            log::error!("{}", error);
    }));

    // Developer mode
    action_devel.connect_state_notify(
        clone!(@weak appwindow, @weak action_devel_settings => move |action_devel| {
            let state = action_devel.state().unwrap().get::<bool>().unwrap();

            action_devel_settings.set_enabled(state);
            appwindow.devel_actions_revealer().set_reveal_child(state);

            if !state {
                appwindow.application().unwrap().change_action_state("visual-debug", &false.to_variant());
            }
        }),
    );

    // Renderer Backend
    action_renderer_backend.connect_state_notify(clone!(@weak appwindow => move |action_renderer_backend| {
        let state = action_renderer_backend.state().unwrap().get::<String>().expect("wrong type for state of 'action_renderer_backend' must be of type String");
        match state.as_str() {
            "librsvg" => {
                appwindow.canvas().sheet().strokes_state().borrow_mut().renderer.write().unwrap().backend = render::RendererBackend::Librsvg;
            },
            "resvg" => {
                appwindow.canvas().sheet().strokes_state().borrow_mut().renderer.write().unwrap().backend = render::RendererBackend::Resvg;
            },
            _ => {
                log::error!("invalid state of action_renderer_backend");
            }
        }

        appwindow.canvas().regenerate_background(false);
        appwindow.canvas().regenerate_content(true, true);
    }));

    // Current Pen
    action_current_pen.connect_activate(move |action_current_pen, parameter| {
        if action_current_pen.state().unwrap().str().unwrap() != parameter.unwrap().str().unwrap() {
            action_current_pen.change_state(parameter.unwrap());
        }
    });
    action_current_pen.connect_change_state(
        clone!(@weak appwindow => move |action_current_pen, value| {
            action_current_pen.set_state(value.unwrap());

            match action_current_pen.state().unwrap().str().unwrap() {
                "marker" => {
                    appwindow.mainheader().marker_toggle().set_active(true);
                    appwindow.canvas().current_pen().set(PenStyle::Marker);
                    appwindow.penssidebar().sidebar_stack().set_visible_child_name("marker_page");
                },
                "brush" => {
                    appwindow.mainheader().brush_toggle().set_active(true);
                    appwindow.canvas().current_pen().set(PenStyle::Brush);
                    appwindow.penssidebar().sidebar_stack().set_visible_child_name("brush_page");
                },
                "shaper" => {
                    appwindow.mainheader().shaper_toggle().set_active(true);
                    appwindow.canvas().current_pen().set(PenStyle::Shaper);
                    appwindow.penssidebar().sidebar_stack().set_visible_child_name("shaper_page");
                },
                "eraser" => {
                    appwindow.mainheader().eraser_toggle().set_active(true);
                    appwindow.canvas().current_pen().set(PenStyle::Eraser);
                    appwindow.penssidebar().sidebar_stack().set_visible_child_name("eraser_page");
                },
                "selector" => {
                    appwindow.mainheader().selector_toggle().set_active(true);
                    appwindow.canvas().current_pen().set(PenStyle::Selector);
                    appwindow.penssidebar().sidebar_stack().set_visible_child_name("selector_page");
                },
                _ => { log::error!("set invalid state of action `current-pen`")}
            }
        }),
    );

    // Shaper drawstyle
    action_shaper_drawstyle.connect_activate(move |action_shaper_drawstyle, parameter| {
        if action_shaper_drawstyle.state().unwrap().str().unwrap()
            != parameter.unwrap().str().unwrap()
        {
            action_shaper_drawstyle.change_state(parameter.unwrap());
        }
    });
    action_shaper_drawstyle.connect_change_state(
        clone!(@weak appwindow => move |action_shaper_drawstyle, value| {
            action_shaper_drawstyle.set_state(value.unwrap());

            match action_shaper_drawstyle.state().unwrap().str().unwrap() {
                "smooth" => {
                    appwindow.penssidebar().shaper_page().drawstyle_smooth_toggle().set_active(true);
                    appwindow.canvas().pens().borrow_mut().shaper.drawstyle = shaper::DrawStyle::Smooth;
                },
                "rough" => {
                    appwindow.penssidebar().shaper_page().drawstyle_rough_toggle().set_active(true);
                    appwindow.canvas().pens().borrow_mut().shaper.drawstyle = shaper::DrawStyle::Rough;
                },
                _ => { log::error!("set invalid state of action `shaper-drawstye`")}
            }
        }),
    );

    // Current Shape
    action_current_shape.connect_activate(move |action_current_shape, parameter| {
        if action_current_shape.state().unwrap().str().unwrap() != parameter.unwrap().str().unwrap()
        {
            action_current_shape.change_state(parameter.unwrap());
        }
    });
    action_current_shape.connect_change_state(
        clone!(@weak appwindow => move |action_current_shape, value| {
            action_current_shape.set_state(value.unwrap());

            match action_current_shape.state().unwrap().str().unwrap() {
                "line" => {
                    appwindow.penssidebar().shaper_page().line_toggle().set_active(true);
                    appwindow.canvas().pens().borrow_mut().shaper.current_shape = shaper::CurrentShape::Line;
                    appwindow.penssidebar().shaper_page().fill_revealer().set_reveal_child(false);
                },
                "rectangle" => {
                    appwindow.penssidebar().shaper_page().rectangle_toggle().set_active(true);
                    appwindow.canvas().pens().borrow_mut().shaper.current_shape = shaper::CurrentShape::Rectangle;
                    appwindow.penssidebar().shaper_page().fill_revealer().set_reveal_child(true);
                },
                "ellipse" => {
                    appwindow.penssidebar().shaper_page().ellipse_toggle().set_active(true);
                    appwindow.canvas().pens().borrow_mut().shaper.current_shape = shaper::CurrentShape::Ellipse;
                    appwindow.penssidebar().shaper_page().fill_revealer().set_reveal_child(true);
                },
                _ => { log::error!("set invalid state of action `current-shape`")}
            }
        }),
    );

    // Selector Style
    action_selector_style.connect_activate(move |action_selector_style, parameter| {
        if action_selector_style.state().unwrap().str().unwrap()
            != parameter.unwrap().str().unwrap()
        {
            action_selector_style.change_state(parameter.unwrap());
        }
    });
    action_selector_style.connect_change_state(
        clone!(@weak appwindow => move |action_selector_style, value| {
            action_selector_style.set_state(value.unwrap());

            match action_selector_style.state().unwrap().str().unwrap() {
                "polygon" => {
                    appwindow.penssidebar().selector_page().selectorstyle_polygon_toggle().set_active(true);
                    appwindow.canvas().pens().borrow_mut().selector.set_style(selector::SelectorStyle::Polygon);
                },
                "rectangle" => {
                    appwindow.penssidebar().selector_page().selectorstyle_rect_toggle().set_active(true);
                    appwindow.canvas().pens().borrow_mut().selector.set_style(selector::SelectorStyle::Rectangle);
                },
                _ => { log::error!("set invalid state of action `shaper-drawstye`")}
            }
        }),
    );

    // Trash Selection
    action_delete_selection.connect_activate(
        clone!(@weak appwindow => move |_action_delete_selection, _| {
            appwindow.canvas().sheet().strokes_state().borrow_mut().trash_selection();
            appwindow.canvas().selection_modifier().set_visible(false);

            appwindow.canvas().queue_draw();
        }),
    );

    // Duplicate Selection
    action_duplicate_selection.connect_activate(
        clone!(@weak appwindow => move |_action_duplicate_selection, _| {
            appwindow.canvas().sheet().strokes_state().borrow_mut().duplicate_selection();

            appwindow.canvas().regenerate_content(false, true);
        }),
    );

    // Format borders
    action_sheet_format_borders.connect_state_notify(
        clone!(@weak appwindow => move |action_sheet_format_borders| {
            let state = action_sheet_format_borders.state().unwrap().get::<bool>().unwrap();
                appwindow.canvas().sheet().set_format_borders(state);
                appwindow.canvas().queue_draw();
        }),
    );

    // Endless Sheet
    action_endless_sheet.connect_state_notify(
        clone!(@weak appwindow => move |action_endless_sheet| {
            let state = action_endless_sheet.state().unwrap().get::<bool>().unwrap();

            appwindow.canvas().sheet().set_endless_sheet(state);
            appwindow.mainheader().pageedit_revealer().set_reveal_child(!state);

            appwindow.canvas().update_background_rendernode();
        }),
    );

    // Righthanded
    action_righthanded.connect_state_notify(clone!(@weak appwindow => move |action_righthanded| {

        if action_righthanded.state().unwrap().get::<bool>().unwrap() {
            appwindow.mainheader().canvasmenu().righthanded_toggle().set_active(true);

            appwindow.main_grid().remove(&appwindow.sidebar_grid());
            appwindow.main_grid().remove(&appwindow.sidebar_sep());
            appwindow.main_grid().remove(&appwindow.devel_actions_revealer());
            appwindow.main_grid().remove(&appwindow.canvas_scroller());
            appwindow.main_grid().attach(&appwindow.sidebar_grid(), 0, 1 ,1, 2);
            appwindow.main_grid().attach(&appwindow.sidebar_sep(), 1, 1 ,1, 2);
            appwindow.main_grid().attach(&appwindow.devel_actions_revealer(), 2, 1 ,1, 1);
            appwindow.main_grid().attach(&appwindow.canvas_scroller(), 2, 2 ,1, 1);

            appwindow.mainheader().headerbar().remove::<gtk4::Box>(&appwindow.mainheader().pens_togglebox());
            appwindow.mainheader().headerbar().remove::<gtk4::Box>(&appwindow.mainheader().quickactions_box());
            appwindow.mainheader().headerbar().pack_end::<gtk4::Box>(&appwindow.mainheader().quickactions_box());
            appwindow.mainheader().headerbar().pack_start::<gtk4::Box>(&appwindow.mainheader().pens_togglebox());

            appwindow.canvas_scroller().set_window_placement(CornerType::BottomRight);

            appwindow.sidebar_scroller().set_window_placement(CornerType::TopRight);
            appwindow.penssidebar().marker_page().colorpicker().set_property("position", PositionType::Left.to_value()).unwrap();
            appwindow.penssidebar().brush_page().templatechooser().help_button().set_direction(ArrowType::Right);
            appwindow.penssidebar().brush_page().templatechooser().chooser_button().set_direction(ArrowType::Right);
            appwindow.penssidebar().brush_page().colorpicker().set_property("position", PositionType::Left.to_value()).unwrap();
            appwindow.penssidebar().shaper_page().stroke_colorpicker().set_property("position", PositionType::Left.to_value()).unwrap();
            appwindow.penssidebar().shaper_page().fill_colorpicker().set_property("position", PositionType::Left.to_value()).unwrap();
            appwindow.penssidebar().shaper_page().roughconfig_menubutton().set_direction(ArrowType::Right);

            appwindow.flap().set_flap_position(PackType::Start);
        } else {
            appwindow.mainheader().canvasmenu().lefthanded_toggle().set_active(true);

            appwindow.main_grid().remove(&appwindow.devel_actions_revealer());
            appwindow.main_grid().remove(&appwindow.canvas_scroller());
            appwindow.main_grid().remove(&appwindow.sidebar_sep());
            appwindow.main_grid().remove(&appwindow.sidebar_grid());
            appwindow.main_grid().attach(&appwindow.devel_actions_revealer(), 0, 1 ,1, 1);
            appwindow.main_grid().attach(&appwindow.canvas_scroller(), 0, 2 ,1, 1);
            appwindow.main_grid().attach(&appwindow.sidebar_sep(), 1, 1 ,1, 2);
            appwindow.main_grid().attach(&appwindow.sidebar_grid(), 2, 1 ,1, 2);

            appwindow.mainheader().headerbar().remove::<gtk4::Box>(&appwindow.mainheader().pens_togglebox());
            appwindow.mainheader().headerbar().remove::<gtk4::Box>(&appwindow.mainheader().quickactions_box());
            appwindow.mainheader().headerbar().pack_start::<gtk4::Box>(&appwindow.mainheader().quickactions_box());
            appwindow.mainheader().headerbar().pack_end::<gtk4::Box>(&appwindow.mainheader().pens_togglebox());

            appwindow.canvas_scroller().set_window_placement(CornerType::BottomLeft);

            appwindow.sidebar_scroller().set_window_placement(CornerType::TopLeft);
            appwindow.penssidebar().marker_page().colorpicker().set_property("position", PositionType::Right.to_value()).unwrap();
            appwindow.penssidebar().brush_page().templatechooser().help_button().set_direction(ArrowType::Left);
            appwindow.penssidebar().brush_page().templatechooser().chooser_button().set_direction(ArrowType::Left);
            appwindow.penssidebar().brush_page().colorpicker().set_property("position", PositionType::Right.to_value()).unwrap();
            appwindow.penssidebar().shaper_page().stroke_colorpicker().set_property("position", PositionType::Right.to_value()).unwrap();
            appwindow.penssidebar().shaper_page().fill_colorpicker().set_property("position", PositionType::Right.to_value()).unwrap();
            appwindow.penssidebar().shaper_page().roughconfig_menubutton().set_direction(ArrowType::Left);

            appwindow.flap().set_flap_position(PackType::End);
        }
    }));

    // Clear sheet
    action_clear_sheet.connect_activate(clone!(@weak appwindow => move |_, _| {
        dialogs::dialog_clear_sheet(&appwindow);
    }));

    // Undo stroke
    action_undo_stroke.connect_activate(clone!(@weak appwindow => move |_,_| {
        appwindow.canvas().sheet().strokes_state().borrow_mut().undo_last_stroke();
        appwindow.canvas().sheet().resize_to_format();
        appwindow.canvas().update_background_rendernode();
    }));

    // Redo stroke
    action_redo_stroke.connect_activate(clone!(@weak appwindow => move |_,_| {
        appwindow.canvas().sheet().strokes_state().borrow_mut().redo_last_stroke();
        appwindow.canvas().sheet().resize_to_format();
        appwindow.canvas().update_background_rendernode();
    }));

    // Zoom reset
    action_zoom_reset.connect_activate(clone!(@weak appwindow => move |_,_| {
        appwindow.canvas().zoom_to(Canvas::ZOOM_DEFAULT);
    }));

    // Zoom fit to width
    action_zoom_fit_width.connect_activate(clone!(@weak appwindow => move |_,_| {
        let mut new_zoom = appwindow.canvas().zoom();

        for _ in 0..2 {
            new_zoom = (f64::from(appwindow.canvas_scroller().width()) - 2.0 * appwindow.canvas().sheet_margin() * new_zoom) / appwindow.canvas().sheet().format().width() as f64;
        }
        appwindow.canvas().zoom_to(new_zoom);
    }));

    // Zoom in
    action_zoomin.connect_activate(clone!(@weak appwindow => move |_,_| {
        let new_zoom = ((appwindow.canvas().zoom() + Canvas::ZOOM_ACTION_DELTA) * 10.0).floor() / 10.0;
        appwindow.canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom, Canvas::ZOOM_TIMEOUT_TIME);
    }));

    // Zoom out
    action_zoomout.connect_activate(clone!(@weak appwindow => move |_,_| {
        let new_zoom = ((appwindow.canvas().zoom() - Canvas::ZOOM_ACTION_DELTA) * 10.0).ceil() / 10.0;
        appwindow.canvas().zoom_temporarily_then_scale_to_after_timeout(new_zoom, Canvas::ZOOM_TIMEOUT_TIME);
    }));

    // Temporary Eraser
    let pen_tmp = Rc::new(Cell::new(PenStyle::default()));

    action_tmperaser.connect_activate(move |action_tmperaser, parameter| {
        if let Some(parameter) = parameter {
            if let Some(state) = parameter.get::<bool>() {
                action_tmperaser.change_state(&state.to_variant());
            }
        } else {
            let state = action_tmperaser.state().unwrap().get::<bool>().unwrap();
            action_tmperaser.change_state(&(!state).to_variant());
        }
    });
    action_tmperaser.connect_change_state(
        clone!(@strong pen_tmp, @weak appwindow => move |action_tmperaser, value| {
            if value.unwrap().get::<bool>().unwrap() != action_tmperaser.state().unwrap().get::<bool>().unwrap() {
                action_tmperaser.set_state(value.unwrap());

                if action_tmperaser.state().unwrap().get::<bool>().unwrap() {
                    pen_tmp.set(appwindow.canvas().current_pen().get());
                    appwindow.canvas().current_pen().set(PenStyle::Eraser);
                } else {
                    appwindow.canvas().current_pen().set(pen_tmp.get());
                };
            }
        }),
    );

    // New sheet
    action_new_sheet.connect_activate(clone!(@weak appwindow => move |_, _| {
        dialogs::dialog_new_sheet(&appwindow);
    }));

    // Open workspace
    action_open_workspace.connect_activate(clone!(@weak appwindow => move |_, _| {
        dialogs::dialog_open_workspace(&appwindow);
    }));

    // Open sheet
    action_open_sheet.connect_activate(clone!(@weak appwindow => move |_, _| {
        dialogs::dialog_open_sheet(&appwindow);
    }));

    // Save sheet
    action_save_sheet.connect_activate(clone!(@weak appwindow => move |_, _| {
        if appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().is_none() {
            dialogs::dialog_save_sheet_as(&appwindow);
        }

        if let Some(output_file) = appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().to_owned() {
            if let Err(e) = appwindow.canvas().sheet().save_sheet_to_file(&output_file) {
                log::error!("failed to save sheet, {}", e);
                appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_output_file(None, &appwindow);
            } else {
                appwindow.canvas().set_unsaved_changes(false);
            }
        }
    }));

    // Save sheet as
    action_save_sheet_as.connect_activate(clone!(@weak appwindow => move |_, _| {
        dialogs::dialog_save_sheet_as(&appwindow);
    }));

    // Print sheet
    action_print_sheet.connect_activate(clone!(@weak appwindow => move |_, _| {
        let print_op = PrintOperation::builder()
            .unit(Unit::Points)
            .n_pages(appwindow.canvas().sheet().calc_n_pages())
            .allow_async(true)
            .build();

            let sheet_bounds= appwindow.canvas().sheet().bounds();
            match appwindow.canvas().sheet().gen_svg() {
                Ok(svg_data) => {
                    let svg = render::Svg {
                        bounds: sheet_bounds,
                        svg_data,
                    };
                    print_op.connect_draw_page(clone!(@weak appwindow => move |_print_op, print_cx, page_nr| {
                        let cx = match print_cx.cairo_context() {
                            None => {
                                log::error!("failed to get cairo context in print_op.connect_draw_page().");
                                return;
                            }
                            Some(cx) => { cx }
                        };

                        let (margin_top, margin_bottom, margin_left, margin_right) = print_cx.hard_margins().unwrap_or( (0.0, 0.0, 0.0, 0.0) );

                        let width_scale = (print_cx.width() + margin_left + margin_right) / f64::from(appwindow.canvas().sheet().format().width());
                        let height_scale = (print_cx.height() + margin_top + margin_bottom) / f64::from(appwindow.canvas().sheet().format().height());
                        let print_zoom = width_scale.min(height_scale);
                        let y_offset = - (f64::from(page_nr * appwindow.canvas().sheet().format().height()) * print_zoom);

                        let format_bounds_scaled = p2d::bounding_volume::AABB::new(
                            na::point![0.0, 0.0],
                            na::point![f64::from(appwindow.canvas().sheet().format().width()) * print_zoom,f64::from(appwindow.canvas().sheet().format().height()) * print_zoom]
                        );

                        cx.rectangle(
                            format_bounds_scaled.mins[0],
                            format_bounds_scaled.mins[1],
                            format_bounds_scaled.extents()[0],
                            format_bounds_scaled.extents()[1]
                        );
                        cx.clip();
                        cx.translate(0.0, y_offset);

                        if let Err(e) = render::draw_svgs_to_cairo_context(print_zoom, &vec![svg.clone()], &cx) {
                            log::error!("render::draw_svgs_to_cairo_context() failed in draw_page() callback while printing page: {}, {}", page_nr, e);

                        }
                }));
            },
                Err(e) => {
                    log::error!("gen_svg() failed in print-sheet action with Err {}", e);
                }
            }

        if let Err(e) = print_op.run(PrintOperationAction::PrintDialog, Some(&appwindow)){
            log::error!("print_op.run() failed with Err, {}", e);
        };

    }));

    // Import
    action_import_file.connect_activate(clone!(@weak appwindow => move |_,_| {
        dialogs::dialog_import_file(&appwindow);
    }));

    // Export selection as SVG
    action_export_selection_as_svg.connect_activate(clone!(@weak appwindow => move |_,_| {
        dialogs::dialog_export_selection(&appwindow);
    }));

    // Export sheet as SVG
    action_export_sheet_as_svg.connect_activate(clone!(@weak appwindow => move |_,_| {
        dialogs::dialog_export_sheet(&appwindow);
    }));

    // Clipboard copy selection
    action_clipboard_copy_selection.connect_activate(clone!(@weak appwindow => move |_, _| {
        match appwindow.canvas().sheet().strokes_state().borrow().gen_svg_selection() {
            Ok(Some(selection_svg)) => {
                let svg_content_provider = gdk::ContentProvider::for_bytes("image/svg+xml", &glib::Bytes::from(selection_svg.as_bytes()));
                match appwindow.clipboard().set_content(Some(&svg_content_provider)) {
                    Ok(_) => {
                    }
                    Err(e) => {
                        log::error!("copy selection into clipboard failed in clipboard().set_content(), {}", e);
                    }
                }
            }
            Ok(None) => {

            }
            Err(e) => {
                log::error!("copy selection into clipboard failed in gen_svg_selection(), {}", e);
            }
        }
    }));

    // Clipboard paste as selection
    action_clipboard_paste_selection.connect_activate(clone!(@weak appwindow => move |_, _| {
        let clipboard = appwindow.clipboard();
        if let Some(formats) = clipboard.formats() {
            for mime_type in formats.mime_types() {
                    match mime_type.as_str() {
                        "image/svg+xml" => {
                            appwindow.clipboard().read_text_async(gio::NONE_CANCELLABLE, clone!(@weak appwindow => move |text_res| {
                                match text_res {
                                    Ok(Some(text)) => {
                                        appwindow.load_in_vectorimage_bytes(text.as_bytes(), None).unwrap_or_else(|e| {
                                            log::error!("failed to paste clipboard as VectorImage, load_in_vectorimage_bytes() returned Err, {}", e);
                                        });
                                    }
                                    Ok(None) => {
                                    }
                                    Err(e) => {
                                        log::error!("failed to paste clipboard as VectorImage, text in callback is Err, {}", e);

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
        }
    }));
}

// ### Accelerators / Keyboard Shortcuts
pub fn setup_accels(appwindow: &RnoteAppWindow) {
    let app = appwindow
        .application()
        .unwrap()
        .downcast::<RnoteApp>()
        .unwrap();

    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
    app.set_accels_for_action("win.close-active", &["<Ctrl>w"]);
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
    app.set_accels_for_action("win.undo-stroke", &["<Ctrl>z"]);
    app.set_accels_for_action("win.redo-stroke", &["<Ctrl><Shift>z"]);
    app.set_accels_for_action("win.zoomin", &["plus"]);
    app.set_accels_for_action("win.zoomout", &["minus"]);
    app.set_accels_for_action("win.delete-selection", &["Delete"]);
    app.set_accels_for_action("win.duplicate-selection", &["<Ctrl>d"]);
    app.set_accels_for_action("win.tmperaser(true)", &["d"]);
    app.set_accels_for_action("win.clipboard-copy-selection", &["<Ctrl>c"]);
    app.set_accels_for_action("win.clipboard-paste-selection", &["<Ctrl>v"]);
}
