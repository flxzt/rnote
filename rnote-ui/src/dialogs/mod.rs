pub(crate) mod export;
pub(crate) mod import;

use adw::prelude::*;
use gettextrs::gettext;
use gtk4::CheckButton;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, ColorButton, Dialog, FileChooserAction,
    FileChooserNative, Label, MenuButton, ResponseType, ShortcutsWindow, StringList,
};

use crate::appwindow::RnoteAppWindow;
use crate::canvaswrapper::RnoteCanvasWrapper;
use crate::config;
use crate::workspacebrowser::workspacesbar::WorkspaceRow;
use crate::{globals, IconPicker};

// About Dialog
pub(crate) fn dialog_about(appwindow: &RnoteAppWindow) {
    let aboutdialog = adw::AboutWindow::builder()
        .modal(true)
        .transient_for(appwindow)
        .application_name(config::APP_NAME_CAPITALIZED)
        .application_icon(config::APP_ID)
        .comments(&gettext("Sketch and take handwritten notes"))
        .website(config::APP_WEBSITE)
        .issue_url(config::APP_ISSUES_URL)
        .support_url(config::APP_SUPPORT_URL)
        .developer_name(config::APP_AUTHOR_NAME)
        .developers(
            config::APP_AUTHORS
                .iter()
                .map(|&s| String::from(s))
                .collect(),
        )
        // TRANSLATORS: 'Name <email@domain.com>' or 'Name https://website.example'
        .translator_credits(&gettext("translator-credits"))
        .license_type(globals::APP_LICENSE)
        .version((String::from(config::APP_VERSION) + config::APP_VERSION_SUFFIX).as_str())
        .build();

    if config::PROFILE == "devel" {
        aboutdialog.add_css_class("devel");
    }

    aboutdialog.show();
}

pub(crate) fn dialog_keyboard_shortcuts(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog_shortcuts: ShortcutsWindow = builder.object("shortcuts_window").unwrap();

    if config::PROFILE == "devel" {
        dialog_shortcuts.add_css_class("devel");
    }

    dialog_shortcuts.set_transient_for(Some(appwindow));
    dialog_shortcuts.show();
}

pub(crate) fn dialog_clear_doc(appwindow: &RnoteAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog_clear_doc: adw::MessageDialog = builder.object("dialog_clear_doc").unwrap();

    dialog_clear_doc.set_transient_for(Some(appwindow));

    dialog_clear_doc.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_clear_doc, response| {
            match response {
                "clear" => {
                    let prev_empty = appwindow.active_tab().canvas().empty();

                    let widget_flags = appwindow.active_tab().canvas().engine().borrow_mut().clear();
                    appwindow.handle_widget_flags(widget_flags);

                    appwindow.active_tab().canvas().return_to_origin_page();
                    appwindow.active_tab().canvas().engine().borrow_mut().resize_autoexpand();

                    if !prev_empty {
                        appwindow.active_tab().canvas().set_unsaved_changes(true);
                    }
                    appwindow.active_tab().canvas().set_empty(true);
                    appwindow.active_tab().canvas().update_engine_rendering();
                },
                _ => {
                // Cancel
                }
            }
        }),
    );

    dialog_clear_doc.show();
}

pub(crate) fn dialog_new_doc(appwindow: &RnoteAppWindow) {
    let new_doc = |appwindow: &RnoteAppWindow| {
        let widget_flags = appwindow
            .active_tab()
            .canvas()
            .engine()
            .borrow_mut()
            .clear();
        appwindow.handle_widget_flags(widget_flags);

        appwindow.active_tab().canvas().return_to_origin_page();
        appwindow
            .active_tab()
            .canvas()
            .engine()
            .borrow_mut()
            .resize_autoexpand();
        appwindow.active_tab().canvas().update_engine_rendering();

        appwindow.active_tab().canvas().set_unsaved_changes(false);
        appwindow.active_tab().canvas().set_empty(true);
        appwindow.active_tab().canvas().set_output_file(None);
    };

    if !appwindow.active_tab().canvas().unsaved_changes() {
        return new_doc(appwindow);
    }

    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog_new_doc: adw::MessageDialog = builder.object("dialog_new_doc").unwrap();

    dialog_new_doc.set_transient_for(Some(appwindow));
    dialog_new_doc.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_new_doc, response| {
        match response {
            "discard" => {
                new_doc(&appwindow)
            },
            "save" => {
                glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                    if let Some(output_file) = appwindow.active_tab().canvas().output_file() {
                        appwindow.overlays().start_pulsing_progressbar();

                        if let Err(e) = appwindow.active_tab().canvas().save_document_to_file(&output_file).await {
                            appwindow.active_tab().canvas().set_output_file(None);

                            log::error!("saving document failed with error `{e:?}`");
                            appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed."));
                        }

                        appwindow.overlays().finish_progressbar();
                        // No success toast on saving without dialog, success is already indicated in the header title

                        // only create new document if saving was successful
                        if !appwindow.active_tab().canvas().unsaved_changes() {
                            new_doc(&appwindow)
                        }
                    } else {
                        // Open a dialog to choose a save location
                        export::filechooser_save_doc_as(&appwindow);
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
pub(crate) fn dialog_close_tab(appwindow: &RnoteAppWindow, active_page: &adw::TabPage) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::MessageDialog = builder.object("dialog_close_tab").unwrap();

    dialog.set_transient_for(Some(appwindow));

    dialog.connect_response(
        None,
        clone!(@weak active_page, @weak appwindow => move |_dialog_quit_save, response| {
            let active_tab = active_page.child().downcast::<RnoteCanvasWrapper>().unwrap();

            match response {
                "discard" => {
                    appwindow.overlays().tabview().close_page_finish(&active_page, true);
                },
                "save" => {
                    glib::MainContext::default().spawn_local(clone!(@weak active_page, @weak appwindow => async move {
                        if let Some(output_file) = active_tab.canvas().output_file() {
                            appwindow.overlays().start_pulsing_progressbar();

                            if let Err(e) = active_tab.canvas().save_document_to_file(&output_file).await {
                                active_tab.canvas().set_output_file(None);

                                log::error!("saving document failed with error `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed."));
                            }

                            appwindow.overlays().finish_progressbar();
                            // No success toast on saving without dialog, success is already indicated in the header title
                        } else {
                            // Open a dialog to choose a save location
                            export::filechooser_save_doc_as(&appwindow);
                        }

                        // only close if saving was successful
                        appwindow
                            .overlays()
                            .tabview()
                            .close_page_finish(
                                &active_page,
                                !active_tab.canvas().unsaved_changes()
                            );
                    }));
                },
                _ => {
                // Cancel
                    appwindow.overlays().tabview().close_page_finish(&active_page, false);
                }
            }
        }),
    );

    dialog.show();
}

pub(crate) async fn dialog_quit_save(appwindow: &RnoteAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::MessageDialog = builder.object("dialog_quit_save").unwrap();
    let files_group: adw::PreferencesGroup = builder.object("quit_save_files_group").unwrap();
    dialog.set_transient_for(Some(appwindow));

    let tabs = appwindow.tab_pages_snapshot();
    let mut rows = Vec::new();
    let mut close = false;
    let mut prev_doc_title = String::new();

    for (i, tab) in tabs.iter().enumerate() {
        let c = tab
            .child()
            .downcast::<RnoteCanvasWrapper>()
            .unwrap()
            .canvas();

        if c.unsaved_changes() {
            let save_folder_path = if let Some(p) = c.output_file().and_then(|f| f.parent()?.path())
            {
                Some(p)
            } else {
                xdg_user::documents().ok().flatten()
            };

            let mut doc_title = c.doc_title_display();
            // Ensuring we don't save with same file names by suffixing with a running index if it already exists
            let mut suff_i = 1;
            while &doc_title == &prev_doc_title {
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

    match dialog.run_future().await.as_str() {
        "discard" => {
            // do nothing and close
            close = true;
        }
        "save" => {
            for (i, check, save_folder_path, doc_title) in rows {
                if check.is_active() {
                    let c = tabs[i]
                        .child()
                        .downcast::<RnoteCanvasWrapper>()
                        .unwrap()
                        .canvas();

                    if let Some(export_folder_path) = save_folder_path {
                        appwindow.overlays().start_pulsing_progressbar();

                        let save_file =
                            gio::File::for_path(export_folder_path.join(doc_title + ".rnote"));

                        if let Err(e) = c.save_document_to_file(&save_file).await {
                            appwindow.active_tab().canvas().set_output_file(None);

                            log::error!("saving document failed with error `{e:?}`");
                            appwindow
                                .overlays()
                                .dispatch_toast_error(&gettext("Saving document failed."));
                        }

                        // No success toast on saving without dialog, success is already indicated in the header title
                        appwindow.overlays().finish_progressbar();
                    }
                }
            }
            close = true;
        }
        _ => {
            // Cancel
        }
    }

    if close {
        appwindow.close_force();
    }
}

pub(crate) fn dialog_edit_selected_workspace(appwindow: &RnoteAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_edit_selected_workspace").unwrap();
    let preview_row: WorkspaceRow = builder
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
    let icon_picker: IconPicker = builder
        .object("edit_selected_workspace_icon_picker")
        .unwrap();

    preview_row.init(appwindow);
    dialog.set_transient_for(Some(appwindow));

    // Sets the icons
    icon_picker.set_list(StringList::new(globals::WORKSPACELISTENTRY_ICONS_LIST));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Change workspace directory"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
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
        clone!(@weak dialog, @weak filechooser, @weak appwindow => move |_| {
            dialog.hide();
            filechooser.show();
        }),
    );

    dialog.show();
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}
