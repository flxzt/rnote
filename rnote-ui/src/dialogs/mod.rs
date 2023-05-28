// gtk4::Dialog is deprecated, but the replacement adw::ToolbarView is not yet stable
#![allow(deprecated)]

// Modules
pub(crate) mod export;
pub(crate) mod import;

// Imports
use crate::appwindow::RnAppWindow;
use crate::canvas::RnCanvas;
use crate::canvaswrapper::RnCanvasWrapper;
use crate::config;
use crate::workspacebrowser::workspacesbar::RnWorkspaceRow;
use crate::{globals, RnIconPicker};
use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use gtk4::{
    gio, glib, glib::clone, Builder, Button, CheckButton, ColorDialogButton, Dialog, FileDialog,
    Label, MenuButton, ResponseType, ShortcutsWindow, StringList,
};

// About Dialog
pub(crate) fn dialog_about(appwindow: &RnAppWindow) {
    let app_icon_name = if config::PROFILE == "devel" {
        config::APP_NAME.to_string() + "-devel"
    } else {
        config::APP_NAME.to_string()
    };

    let aboutdialog = adw::AboutWindow::builder()
        .modal(true)
        .transient_for(appwindow)
        .application_name(config::APP_NAME_CAPITALIZED)
        .application_icon(app_icon_name)
        .comments(gettext("Sketch and take handwritten notes"))
        .website(config::APP_WEBSITE)
        .issue_url(config::APP_ISSUES_URL)
        .support_url(config::APP_SUPPORT_URL)
        .developer_name(config::APP_AUTHOR_NAME)
        .developers(glib::StrV::from(
            config::APP_AUTHORS
                .iter()
                .map(|&s| String::from(s))
                .collect::<Vec<String>>(),
        ))
        // TRANSLATORS: 'Name <email@domain.com>' or 'Name https://website.example'
        .translator_credits(gettext("translator-credits"))
        .license_type(globals::APP_LICENSE)
        .version((String::from(config::APP_VERSION) + config::APP_VERSION_SUFFIX).as_str())
        .build();

    if config::PROFILE == "devel" {
        aboutdialog.add_css_class("devel");
    }

    aboutdialog.present();
}

pub(crate) fn dialog_keyboard_shortcuts(appwindow: &RnAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog: ShortcutsWindow = builder.object("shortcuts_window").unwrap();
    dialog.set_transient_for(Some(appwindow));
    dialog.present();
}

pub(crate) fn dialog_clear_doc(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::MessageDialog = builder.object("dialog_clear_doc").unwrap();
    dialog.set_transient_for(Some(appwindow));

    dialog.connect_response(
        None,
        clone!(@weak canvas, @weak appwindow => move |_dialog_clear_doc, response| {
            match response {
                "clear" => {
                    let prev_empty = canvas.empty();

                    let mut widget_flags = canvas.engine().borrow_mut().clear();
                    canvas.return_to_origin_page();
                    widget_flags.merge(canvas.engine().borrow_mut().doc_resize_autoexpand());
                    if !prev_empty {
                        canvas.set_unsaved_changes(true);
                    }
                    canvas.set_empty(true);
                    canvas.update_rendering_current_viewport();
                    appwindow.handle_widget_flags(widget_flags, &canvas);
                },
                _ => {
                // Cancel
                }
            }
        }),
    );

    dialog.present();
}

pub(crate) fn dialog_new_doc(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let new_doc = |appwindow: &RnAppWindow, canvas: &RnCanvas| {
        let mut widget_flags = canvas.engine().borrow_mut().clear();
        canvas.return_to_origin_page();
        widget_flags.merge(canvas.engine().borrow_mut().doc_resize_autoexpand());
        canvas.update_rendering_current_viewport();
        canvas.set_unsaved_changes(false);
        canvas.set_empty(true);
        canvas.set_output_file(None);
        appwindow.handle_widget_flags(widget_flags, canvas);
    };

    if !canvas.unsaved_changes() {
        new_doc(appwindow, canvas);
        return;
    }

    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog_new_doc: adw::MessageDialog = builder.object("dialog_new_doc").unwrap();

    dialog_new_doc.set_transient_for(Some(appwindow));
    dialog_new_doc.connect_response(
        None,
        clone!(@weak canvas, @weak appwindow => move |_dialog_new_doc, response| {
        match response {
            "discard" => {
                new_doc(&appwindow, &canvas);
            },
            "save" => {
                glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                    if let Some(output_file) = canvas.output_file() {
                        appwindow.overlays().start_pulsing_progressbar();

                        if let Err(e) = canvas.save_document_to_file(&output_file).await {
                            canvas.set_output_file(None);

                            log::error!("saving document failed, Error: `{e:?}`");
                            appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed"));
                        }

                        appwindow.overlays().finish_progressbar();
                        // No success toast on saving without dialog, success is already indicated in the header title

                        // only create new document if saving was successful
                        if !canvas.unsaved_changes() {
                            new_doc(&appwindow, &canvas);
                        }
                    } else {
                        // Open a dialog to choose a save location
                        export::dialog_save_doc_as(&appwindow, &canvas).await;
                    }
                }));
            },
            _ => {
                // Cancel
            }
        }
        }),
    );

    dialog_new_doc.present();
}

/// Only to be called from the tabview close-page handler
pub(crate) fn dialog_close_tab(appwindow: &RnAppWindow, tab_page: &adw::TabPage) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::MessageDialog = builder.object("dialog_close_tab").unwrap();
    let file_group: adw::PreferencesGroup = builder.object("close_tab_file_group").unwrap();
    dialog.set_transient_for(Some(appwindow));
    let canvas = tab_page
        .child()
        .downcast::<RnCanvasWrapper>()
        .unwrap()
        .canvas();

    let mut doc_title = canvas.doc_title_display();
    let save_folder_path = if let Some(p) = canvas.output_file().and_then(|f| f.parent()?.path()) {
        Some(p)
    } else {
        appwindow.workspacebrowser().dirlist_dir()
    };

    let check = CheckButton::builder().active(true).build();
    // Lock checkbox to active state as user can discard document if they choose
    check.set_sensitive(false);

    // Handle possible file collisions
    if let Some(save_folder_path) = save_folder_path.clone() {
        let mut doc_file = gio::File::for_path(save_folder_path.join(doc_title.clone() + ".rnote"));

        if gio::File::query_exists(&doc_file, None::<&gio::Cancellable>) {
            let mut postfix = 0;
            while gio::File::query_exists(&doc_file, None::<&gio::Cancellable>) {
                postfix += 1;
                doc_file = gio::File::for_path(
                    save_folder_path
                        .join(doc_title.clone() + " - " + &postfix.to_string() + ".rnote"),
                );
            }
            doc_title = doc_title + " - " + &postfix.to_string();
        }
    }

    let row = adw::ActionRow::builder()
        .title(doc_title.clone() + ".rnote")
        .subtitle(
            save_folder_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| gettext("- unable to find a valid save folder -")),
        )
        .build();
    row.add_prefix(&check);

    if save_folder_path.is_none() {
        // Indicate that the file cannot be saved
        row.set_sensitive(false);
        check.set_active(false);
    }

    file_group.add(&row);
    dialog.connect_response(
        None,
        clone!(@weak tab_page, @weak canvas, @weak appwindow => move |_, response| {
            let output_folder_path = save_folder_path.clone();
            let doc_title = doc_title.clone();
            match response {
                "discard" => {
                    appwindow.overlays().tabview().close_page_finish(&tab_page, true);
                },
                "save" => {
                    glib::MainContext::default().spawn_local(clone!(@weak tab_page, @weak canvas, @weak appwindow => async move {
                        if let Some(output_folder_path) = output_folder_path {
                            let save_file =
                                    gio::File::for_path(output_folder_path.join(doc_title + ".rnote"));
                            appwindow.overlays().start_pulsing_progressbar();

                            if let Err(e) = canvas.save_document_to_file(&save_file).await {
                                canvas.set_output_file(None);

                                log::error!("saving document failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed"));
                            }

                            appwindow.overlays().finish_progressbar();
                            // No success toast on saving without dialog, success is already indicated in the header title
                        }
                        // only close if saving was successful
                        appwindow
                            .overlays()
                            .tabview()
                            .close_page_finish(
                                &tab_page,
                                !canvas.unsaved_changes()
                            );
                    }));
                },
                _ => {
                // Cancel
                    appwindow.overlays().tabview().close_page_finish(&tab_page, false);
                }
            }
        }),
    );

    dialog.present();
}

pub(crate) async fn dialog_close_window(appwindow: &RnAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::MessageDialog = builder.object("dialog_close_window").unwrap();
    let files_group: adw::PreferencesGroup = builder.object("close_window_files_group").unwrap();
    dialog.set_transient_for(Some(appwindow));

    let tabs = appwindow.tab_pages_snapshot();
    let mut rows = Vec::new();
    let mut postfix = 0;
    for (i, tab) in tabs.iter().enumerate() {
        let canvas = tab.child().downcast::<RnCanvasWrapper>().unwrap().canvas();

        if !canvas.unsaved_changes() {
            continue;
        }

        let save_folder_path =
            if let Some(p) = canvas.output_file().and_then(|f| f.parent()?.path()) {
                Some(p)
            } else {
                appwindow.workspacebrowser().dirlist_dir()
            };

        let mut doc_title = canvas.doc_title_display();

        // Handle possible file collisions
        if let Some(save_folder_path) = save_folder_path.clone() {
            let mut doc_file = if postfix == 0 {
                gio::File::for_path(save_folder_path.join(doc_title.clone() + ".rnote"))
            } else {
                gio::File::for_path(
                    save_folder_path
                        .join(doc_title.clone() + " - " + &postfix.to_string() + ".rnote"),
                )
            };
            while gio::File::query_exists(&doc_file, None::<&gio::Cancellable>) {
                postfix += 1;
                doc_file = gio::File::for_path(
                    save_folder_path
                        .join(doc_title.clone() + " - " + &postfix.to_string() + ".rnote"),
                );
            }
            if postfix != 0 {
                doc_title = doc_title + " - " + &postfix.to_string();
            }
            postfix += 1;
        }

        // Active by default
        let check = CheckButton::builder().active(true).build();

        let row = adw::ActionRow::builder()
            .title(&(doc_title.clone() + ".rnote"))
            .subtitle(
                &save_folder_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| gettext("- unable to find a valid save folder -")),
            )
            .build();

        row.add_prefix(&check);

        if save_folder_path.is_none() {
            // Indicate that the file cannot be saved
            check.set_active(false);
            row.set_sensitive(false);
        }

        files_group.add(&row);

        rows.push((i, check, save_folder_path, doc_title));
    }

    // TODO: as soon as libadwaita v1.3 is out, this can be replaced by choose_future()
    dialog.connect_response(
        None,
        clone!(@strong appwindow => move |_, response| {
         let response = response.to_string();
         let rows = rows.clone();
         let tabs = tabs.clone();
         glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
            let mut close = false;

            match response.as_str() {
                "discard"
                    => {
                    // do nothing and close
                    close = true;
                }
                "save" => {
                    appwindow.overlays().start_pulsing_progressbar();

                    for (i, check, save_folder_path, doc_title) in rows {
                        if check.is_active() {
                            let canvas = tabs[i]
                                .child()
                                .downcast::<RnCanvasWrapper>()
                                .unwrap()
                                .canvas();

                            if let Some(export_folder_path) = save_folder_path {
                                let save_file =
                                    gio::File::for_path(export_folder_path.join(doc_title + ".rnote"));

                                if let Err(e) = canvas.save_document_to_file(&save_file).await {
                                    canvas.set_output_file(None);

                                    log::error!("saving document failed, Error: `{e:?}`");
                                    appwindow
                                        .overlays()
                                        .dispatch_toast_error(&gettext("Saving document failed"));
                                }

                                // No success toast on saving without dialog, success is already indicated in the header title
                            }
                        }
                    }

                    appwindow.overlays().finish_progressbar();
                    close = true;
                }
                _ => {
                    // Cancel
                }
            }

            if close {
                appwindow.close_force();
            }
         }));
        }),
    );

    dialog.present();
}

pub(crate) async fn dialog_edit_selected_workspace(appwindow: &RnAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_edit_selected_workspace").unwrap();
    let preview_row: RnWorkspaceRow = builder
        .object("edit_selected_workspace_preview_row")
        .unwrap();
    let name_entryrow: adw::EntryRow = builder
        .object("edit_selected_workspace_name_entryrow")
        .unwrap();
    let color_button: ColorDialogButton = builder
        .object("edit_selected_workspace_color_button")
        .unwrap();
    let dir_label: Label = builder.object("edit_selected_workspace_dir_label").unwrap();
    let dir_button: Button = builder
        .object("edit_selected_workspace_dir_button")
        .unwrap();
    let icon_menubutton: MenuButton = builder
        .object("edit_selected_workspace_icon_menubutton")
        .unwrap();
    let icon_picker: RnIconPicker = builder
        .object("edit_selected_workspace_icon_picker")
        .unwrap();

    preview_row.init(appwindow);
    dialog.set_transient_for(Some(appwindow));

    // Sets the icons
    icon_picker.set_list(
        StringList::new(WORKSPACELISTENTRY_ICONS_LIST),
        Some(workspacelistentry_icons_list_to_display_name),
        false,
    );

    let Some(initial_entry) = appwindow
        .workspacebrowser()
        .workspacesbar()
        .selected_workspacelistentry() else {
            log::warn!("tried to edit workspace entry in dialog, but no workspace is selected");
            return;
        };

    // set initial dialog UI on popup
    preview_row.entry().replace_data(&initial_entry);
    name_entryrow.set_text(initial_entry.name().as_str());
    icon_menubutton.set_icon_name(initial_entry.icon().as_str());
    color_button.set_rgba(&initial_entry.color());
    dir_label.set_label(initial_entry.dir().as_str());

    name_entryrow.connect_changed(clone!(@weak preview_row => move |entry| {
        let text = entry.text().to_string();
        preview_row.entry().set_name(text);
    }));

    icon_picker.connect_notify_local(
        Some("picked"),
        clone!(@weak icon_menubutton, @weak preview_row, @weak appwindow => move |iconpicker, _| {
            if let Some(picked) = iconpicker.picked() {
                icon_menubutton.set_icon_name(&picked);
                preview_row.entry().set_icon(picked);
            }
        }),
    );

    color_button.connect_rgba_notify(clone!(@weak preview_row => move |button| {
        let color = button.rgba();
        preview_row.entry().set_color(color);
    }));

    dir_button.connect_clicked(
        clone!(@strong preview_row, @weak dir_label, @weak name_entryrow, @weak dialog, @weak appwindow => move |_| {
            glib::MainContext::default().spawn_local(clone!(@strong preview_row, @weak dir_label, @weak name_entryrow, @weak dialog, @weak appwindow => async move {
                dialog.hide();

                let filedialog = FileDialog::builder()
                    .title(gettext("Change Workspace Directory"))
                    .modal(true)
                    .accept_label(gettext("Select"))
                    .initial_file(&gio::File::for_path(preview_row.entry().dir()))
                    .build();

                match filedialog.select_folder_future(Some(&appwindow)).await {
                    Ok(selected_file) => {
                        if let Some(p) = selected_file
                            .path()
                        {
                            let path_string = p.to_string_lossy().to_string();
                            dir_label.set_label(&path_string);
                            preview_row.entry().set_dir(path_string);

                            // Update the entry row with the file name of the new selected directory
                            if let Some(file_name) = p.file_name().map(|n| n.to_string_lossy()) {
                                name_entryrow.set_text(&file_name);
                            }
                        } else {
                            dir_label.set_label(&gettext("- no directory selected -"));
                        }
                    }
                    Err(e) => {
                        log::debug!("did not select new folder for workspacerow (Error or dialog dismissed by user), {e:?}");
                    }
                }
                dialog.present();
            }));
        }),
    );

    dialog.connect_response(
        clone!(@weak preview_row, @weak appwindow => move |dialog, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    // update the actual selected entry
                    appwindow.workspacebrowser().workspacesbar().replace_selected_workspacelistentry(preview_row.entry());
                    // refreshing the files list
                    appwindow.workspacebrowser().refresh_dirlist_selected_workspace();
                    // And save the state
                    appwindow.workspacebrowser().workspacesbar().save_to_settings(&appwindow.app_settings());
                }
                _ => {}
            }

            dialog.close();
        }));

    dialog.present();
}

const WORKSPACELISTENTRY_ICONS_LIST: &[&str] = &[
    "workspacelistentryicon-bandaid-symbolic",
    "workspacelistentryicon-bank-symbolic",
    "workspacelistentryicon-bookmark-symbolic",
    "workspacelistentryicon-book-symbolic",
    "workspacelistentryicon-bread-symbolic",
    "workspacelistentryicon-calendar-symbolic",
    "workspacelistentryicon-camera-symbolic",
    "workspacelistentryicon-chip-symbolic",
    "workspacelistentryicon-clock-symbolic",
    "workspacelistentryicon-code-symbolic",
    "workspacelistentryicon-compose-symbolic",
    "workspacelistentryicon-crop-symbolic",
    "workspacelistentryicon-dictionary-symbolic",
    "workspacelistentryicon-document-symbolic",
    "workspacelistentryicon-drinks-symbolic",
    "workspacelistentryicon-flag-symbolic",
    "workspacelistentryicon-folder-symbolic",
    "workspacelistentryicon-footprints-symbolic",
    "workspacelistentryicon-gamepad-symbolic",
    "workspacelistentryicon-gear-symbolic",
    "workspacelistentryicon-globe-symbolic",
    "workspacelistentryicon-hammer-symbolic",
    "workspacelistentryicon-heart-symbolic",
    "workspacelistentryicon-hourglass-symbolic",
    "workspacelistentryicon-key-symbolic",
    "workspacelistentryicon-language-symbolic",
    "workspacelistentryicon-library-symbolic",
    "workspacelistentryicon-lightbulb-symbolic",
    "workspacelistentryicon-math-symbolic",
    "workspacelistentryicon-meeting-symbolic",
    "workspacelistentryicon-money-symbolic",
    "workspacelistentryicon-musicnote-symbolic",
    "workspacelistentryicon-nature-symbolic",
    "workspacelistentryicon-open-book-symbolic",
    "workspacelistentryicon-paintbrush-symbolic",
    "workspacelistentryicon-pencilandpaper-symbolic",
    "workspacelistentryicon-people-symbolic",
    "workspacelistentryicon-person-symbolic",
    "workspacelistentryicon-projector-symbolic",
    "workspacelistentryicon-science-symbolic",
    "workspacelistentryicon-scratchpad-symbolic",
    "workspacelistentryicon-shapes-symbolic",
    "workspacelistentryicon-shopping-symbolic",
    "workspacelistentryicon-speechbubble-symbolic",
    "workspacelistentryicon-speedometer-symbolic",
    "workspacelistentryicon-star-symbolic",
    "workspacelistentryicon-terminal-symbolic",
    "workspacelistentryicon-text-symbolic",
    "workspacelistentryicon-travel-symbolic",
    "workspacelistentryicon-weather-symbolic",
    "workspacelistentryicon-weight-symbolic",
];

fn workspacelistentry_icons_list_to_display_name(icon_name: &str) -> String {
    match icon_name {
        "workspacelistentryicon-bandaid-symbolic" => gettext("Band-Aid"),
        "workspacelistentryicon-bank-symbolic" => gettext("Bank"),
        "workspacelistentryicon-bookmark-symbolic" => gettext("Bookmark"),
        "workspacelistentryicon-book-symbolic" => gettext("Book"),
        "workspacelistentryicon-bread-symbolic" => gettext("Bread"),
        "workspacelistentryicon-calendar-symbolic" => gettext("Calendar"),
        "workspacelistentryicon-camera-symbolic" => gettext("Camera"),
        "workspacelistentryicon-chip-symbolic" => pgettext("as in computer chip", "Chip"),
        "workspacelistentryicon-clock-symbolic" => gettext("Clock"),
        "workspacelistentryicon-code-symbolic" => gettext("Code"),
        "workspacelistentryicon-compose-symbolic" => gettext("Compose"),
        "workspacelistentryicon-crop-symbolic" => pgettext("as in plant", "Crop"),
        "workspacelistentryicon-dictionary-symbolic" => gettext("Dictionary"),
        "workspacelistentryicon-document-symbolic" => gettext("Document"),
        "workspacelistentryicon-drinks-symbolic" => gettext("Drinks"),
        "workspacelistentryicon-flag-symbolic" => gettext("Flag"),
        "workspacelistentryicon-folder-symbolic" => gettext("Folder"),
        "workspacelistentryicon-footprints-symbolic" => gettext("Footprints"),
        "workspacelistentryicon-gamepad-symbolic" => gettext("Gamepad"),
        "workspacelistentryicon-gear-symbolic" => gettext("Gear"),
        "workspacelistentryicon-globe-symbolic" => gettext("Globe"),
        "workspacelistentryicon-hammer-symbolic" => gettext("Hammer"),
        "workspacelistentryicon-heart-symbolic" => gettext("Heart"),
        "workspacelistentryicon-hourglass-symbolic" => gettext("Hourglass"),
        "workspacelistentryicon-key-symbolic" => gettext("Key"),
        "workspacelistentryicon-language-symbolic" => gettext("Language"),
        "workspacelistentryicon-library-symbolic" => gettext("Library"),
        "workspacelistentryicon-lightbulb-symbolic" => gettext("Lightbulb"),
        "workspacelistentryicon-math-symbolic" => gettext("Mathematics"),
        "workspacelistentryicon-meeting-symbolic" => gettext("Meeting"),
        "workspacelistentryicon-money-symbolic" => gettext("Money"),
        "workspacelistentryicon-musicnote-symbolic" => gettext("Musical Note"),
        "workspacelistentryicon-nature-symbolic" => gettext("Nature"),
        "workspacelistentryicon-open-book-symbolic" => gettext("Open Book"),
        "workspacelistentryicon-paintbrush-symbolic" => gettext("Paintbrush"),
        "workspacelistentryicon-pencilandpaper-symbolic" => gettext("Pencil and Paper"),
        "workspacelistentryicon-people-symbolic" => gettext("People"),
        "workspacelistentryicon-person-symbolic" => gettext("Person"),
        "workspacelistentryicon-projector-symbolic" => gettext("Projector"),
        "workspacelistentryicon-science-symbolic" => gettext("Science"),
        "workspacelistentryicon-scratchpad-symbolic" => gettext("Scratchpad"),
        "workspacelistentryicon-shapes-symbolic" => gettext("Shapes"),
        "workspacelistentryicon-shopping-symbolic" => gettext("Shopping"),
        "workspacelistentryicon-speechbubble-symbolic" => gettext("Speech Bubble"),
        "workspacelistentryicon-speedometer-symbolic" => gettext("Speedometer"),
        "workspacelistentryicon-star-symbolic" => gettext("Star"),
        "workspacelistentryicon-terminal-symbolic" => {
            pgettext("as in terminal software", "Terminal")
        }
        "workspacelistentryicon-text-symbolic" => gettext("Text"),
        "workspacelistentryicon-travel-symbolic" => gettext("Travel"),
        "workspacelistentryicon-weather-symbolic" => gettext("Weather"),
        "workspacelistentryicon-weight-symbolic" => gettext("Weight"),
        _ => unimplemented!(),
    }
}
