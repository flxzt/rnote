pub(crate) mod export;
pub(crate) mod import;

use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use gtk4::CheckButton;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, ColorButton, Dialog, FileChooserAction,
    FileChooserNative, Label, MenuButton, ResponseType, ShortcutsWindow, StringList,
};

use crate::appwindow::RnAppWindow;
use crate::canvas::RnCanvas;
use crate::canvaswrapper::RnCanvasWrapper;
use crate::config;
use crate::workspacebrowser::workspacesbar::RnWorkspaceRow;
use crate::{globals, RnIconPicker};

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

    aboutdialog.show();
}

pub(crate) fn dialog_keyboard_shortcuts(appwindow: &RnAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog_shortcuts: ShortcutsWindow = builder.object("shortcuts_window").unwrap();

    if config::PROFILE == "devel" {
        dialog_shortcuts.add_css_class("devel");
    }

    dialog_shortcuts.set_transient_for(Some(appwindow));
    dialog_shortcuts.show();
}

pub(crate) fn dialog_clear_doc(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog_clear_doc: adw::MessageDialog = builder.object("dialog_clear_doc").unwrap();

    dialog_clear_doc.set_transient_for(Some(appwindow));

    dialog_clear_doc.connect_response(
        None,
        clone!(@weak canvas, @weak appwindow => move |_dialog_clear_doc, response| {
            match response {
                "clear" => {
                    let prev_empty = canvas.empty();

                    let widget_flags = canvas.engine().borrow_mut().clear();
                    appwindow.handle_widget_flags(widget_flags, &canvas);

                    canvas.return_to_origin_page();
                    canvas.engine().borrow_mut().resize_autoexpand();

                    if !prev_empty {
                        canvas.set_unsaved_changes(true);
                    }
                    canvas.set_empty(true);
                    canvas.update_engine_rendering();
                },
                _ => {
                // Cancel
                }
            }
        }),
    );

    dialog_clear_doc.show();
}

pub(crate) fn dialog_new_doc(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let new_doc = |appwindow: &RnAppWindow, canvas: &RnCanvas| {
        let widget_flags = canvas.engine().borrow_mut().clear();
        appwindow.handle_widget_flags(widget_flags, canvas);

        canvas.return_to_origin_page();
        canvas.engine().borrow_mut().resize_autoexpand();
        canvas.update_engine_rendering();

        canvas.set_unsaved_changes(false);
        canvas.set_empty(true);
        canvas.set_output_file(None);
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
                        export::filechooser_save_doc_as(&appwindow, &canvas);
                    }
                }));
            },
            _ => {
                // Cancel
            }
        }
        }),
    );

    dialog_new_doc.show();
}

/// Only to be called from the tabview close-page handler
pub(crate) fn dialog_close_tab(appwindow: &RnAppWindow, tab_page: &adw::TabPage) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::MessageDialog = builder.object("dialog_close_tab").unwrap();

    dialog.set_transient_for(Some(appwindow));

    dialog.connect_response(
        None,
        clone!(@weak tab_page, @weak appwindow => move |_, response| {
            let canvas = tab_page.child().downcast::<RnCanvasWrapper>().unwrap().canvas();

            match response {
                "discard" => {
                    appwindow.overlays().tabview().close_page_finish(&tab_page, true);
                },
                "save" => {
                    glib::MainContext::default().spawn_local(clone!(@weak tab_page, @weak canvas, @weak appwindow => async move {
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
                            export::filechooser_save_doc_as(&appwindow, &canvas);
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

    dialog.show();
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
    let mut prev_doc_title = String::new();

    for (i, tab) in tabs.iter().enumerate() {
        let canvas = tab.child().downcast::<RnCanvasWrapper>().unwrap().canvas();

        if canvas.unsaved_changes() {
            let save_folder_path = if let Some(p) =
                canvas.output_file().and_then(|f| f.parent()?.path())
            {
                Some(p)
            } else {
                directories::UserDirs::new().and_then(|u| u.document_dir().map(|p| p.to_path_buf()))
            };

            let mut doc_title = canvas.doc_title_display();
            // Ensuring we don't save with same file names by suffixing with a running index if it already exists
            let mut suff_i = 1;
            while doc_title == prev_doc_title {
                suff_i += 1;
                doc_title += &format!(" - {suff_i}");
            }
            prev_doc_title = doc_title.clone();

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

    dialog.show();
}

pub(crate) fn dialog_edit_selected_workspace(appwindow: &RnAppWindow) {
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
    let color_button: ColorButton = builder
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

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(gettext("Change Workspace Directory"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(gettext("Select"))
        .cancel_label(gettext("Cancel"))
        .action(FileChooserAction::SelectFolder)
        .select_multiple(false)
        .build();

    let Some(initial_entry) = appwindow
        .workspacebrowser()
        .workspacesbar()
        .selected_workspacelistentry() else {
            log::warn!("tried to edit workspace dialog, but no workspace was selected");
            return;
        };
    if let Err(e) = filechooser.set_file(&gio::File::for_path(initial_entry.dir())) {
        log::error!("set file in change workspace dialog failed with Err: {e:?}");
    }

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

    color_button.connect_color_set(clone!(@weak preview_row => move |button| {
        let color = button.rgba();
        preview_row.entry().set_color(color);
    }));

    filechooser.connect_response(clone!(
        @weak preview_row,
        @weak name_entryrow,
        @weak dir_label,
        @weak dialog,
        @weak appwindow => move |filechooser, responsetype| {
        match responsetype {
            ResponseType::Accept => {
                if let Some(p) = filechooser.file().and_then(|f| f.path()) {
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
            _ => {}
        }

        filechooser.hide();
        dialog.show();
    }));

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

    dir_button.connect_clicked(
        clone!(@weak dialog, @strong filechooser, @weak appwindow => move |_| {
            dialog.hide();
            filechooser.show();
        }),
    );

    dialog.show();
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
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
