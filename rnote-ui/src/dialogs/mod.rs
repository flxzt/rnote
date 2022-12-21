pub(crate) mod export;
pub(crate) mod import;

use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, ColorButton, Dialog, FileChooserAction,
    FileChooserNative, Label, MenuButton, ResponseType, ShortcutsWindow, StringList,
};

use crate::appwindow::RnoteAppWindow;
use crate::config;
use crate::workspacebrowser::WorkspaceRow;
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
                    let prev_empty = appwindow.canvas().empty();

                    let widget_flags = appwindow.canvas().engine().borrow_mut().clear();
                    appwindow.handle_widget_flags(widget_flags);

                    appwindow.canvas().return_to_origin_page();
                    appwindow.canvas().engine().borrow_mut().resize_autoexpand();

                    if !prev_empty {
                        appwindow.canvas().set_unsaved_changes(true);
                    }
                    appwindow.canvas().set_empty(true);
                    appwindow.canvas().update_engine_rendering();
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
        let widget_flags = appwindow.canvas().engine().borrow_mut().clear();
        appwindow.handle_widget_flags(widget_flags);

        appwindow.canvas().return_to_origin_page();
        appwindow.canvas().engine().borrow_mut().resize_autoexpand();
        appwindow.canvas().update_engine_rendering();

        appwindow.canvas().set_unsaved_changes(false);
        appwindow.canvas().set_empty(true);
        appwindow.canvas().set_output_file(None);
    };

    if !appwindow.canvas().unsaved_changes() {
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
                    if let Some(output_file) = appwindow.canvas().output_file() {
                        appwindow.canvas_wrapper().start_pulsing_progressbar();

                        if let Err(e) = appwindow.save_document_to_file(&output_file).await {
                            appwindow.canvas().set_output_file(None);

                            log::error!("saving document failed with error `{e:?}`");
                            appwindow.canvas_wrapper().dispatch_toast_error(&gettext("Saving document failed."));
                        }

                        appwindow.canvas_wrapper().finish_progressbar();
                        // No success toast on saving without dialog, success is already indicated in the header title

                        // only create new document if saving was successful
                        if !appwindow.canvas().unsaved_changes() {
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

pub(crate) fn dialog_quit_save(appwindow: &RnoteAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog_quit_save: adw::MessageDialog = builder.object("dialog_quit_save").unwrap();

    dialog_quit_save.set_transient_for(Some(appwindow));

    dialog_quit_save.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_quit_save, response| {
            match response {
                "discard" => {
                    appwindow.close_force();
                },
                "save" => {
                    glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                        if let Some(output_file) = appwindow.canvas().output_file() {
                            appwindow.canvas_wrapper().start_pulsing_progressbar();

                            if let Err(e) = appwindow.save_document_to_file(&output_file).await {
                                appwindow.canvas().set_output_file(None);

                                log::error!("saving document failed with error `{e:?}`");
                                appwindow.canvas_wrapper().dispatch_toast_error(&gettext("Saving document failed."));
                            }

                            appwindow.canvas_wrapper().finish_progressbar();
                            // No success toast on saving without dialog, success is already indicated in the header title
                        } else {
                            // Open a dialog to choose a save location
                            export::filechooser_save_doc_as(&appwindow);
                        }

                        // only close if saving was successful
                        if !appwindow.canvas().unsaved_changes() {
                            appwindow.close_force();
                        }
                    }));
                },
                _ => {
                // Cancel
                }
            }
        }),
    );

    dialog_quit_save.show();
}

pub(crate) fn dialog_edit_workspace(appwindow: &RnoteAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog_edit_workspace: Dialog = builder.object("dialog_edit_workspace").unwrap();
    let edit_workspace_preview_row: WorkspaceRow =
        builder.object("edit_workspace_preview_row").unwrap();
    let change_workspace_name_entryrow: adw::EntryRow =
        builder.object("change_workspace_name_entryrow").unwrap();
    let change_workspace_color_button: ColorButton =
        builder.object("change_workspace_color_button").unwrap();
    let change_workspace_dir_label: Label = builder.object("change_workspace_dir_label").unwrap();
    let change_workspace_dir_button: Button =
        builder.object("change_workspace_dir_button").unwrap();
    let change_workspace_icon_menubutton: MenuButton =
        builder.object("change_workspace_icon_menubutton").unwrap();
    let change_workspace_icon_picker: IconPicker =
        builder.object("change_workspace_icon_picker").unwrap();

    edit_workspace_preview_row.init(appwindow);
    dialog_edit_workspace.set_transient_for(Some(appwindow));

    // Sets the icons
    change_workspace_icon_picker.set_list(StringList::new(globals::WORKSPACELISTENTRY_ICONS_LIST));

    let filechooser_change_workspace_dir: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Change workspace directory"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::SelectFolder)
        .select_multiple(false)
        .build();

    if let Some(row) = appwindow
        .workspacebrowser()
        .current_selected_workspace_row()
    {
        if let Err(e) =
            filechooser_change_workspace_dir.set_file(&gio::File::for_path(row.entry().dir()))
        {
            log::error!("set file in change workspace dialog failed with Err: {e:?}");
        }

        // set initial dialog UI on popup
        edit_workspace_preview_row
            .entry()
            .replace_data(&row.entry());
        change_workspace_name_entryrow.set_text(row.entry().name().as_str());
        change_workspace_icon_menubutton.set_icon_name(row.entry().icon().as_str());
        change_workspace_color_button.set_rgba(&row.entry().color());
        change_workspace_dir_label.set_label(row.entry().dir().as_str());
    }

    change_workspace_name_entryrow.connect_changed(
        clone!(@weak edit_workspace_preview_row => move |entry| {
            let text = entry.text().to_string();
            edit_workspace_preview_row.entry().set_name(text);
        }),
    );

    change_workspace_icon_picker.connect_local(
        "icon-picked",
        false,
        clone!(@weak change_workspace_icon_menubutton, @weak edit_workspace_preview_row, @weak appwindow =>@default-return None, move |args| {
            let picked = args[1].get::<String>().unwrap();

            change_workspace_icon_menubutton.set_icon_name(&picked);
            edit_workspace_preview_row.entry().set_icon(picked);
            None
        }),
    );

    change_workspace_color_button.connect_color_set(
        clone!(@weak edit_workspace_preview_row => move |button| {
            let color = button.rgba();
            edit_workspace_preview_row.entry().set_color(color);
        }),
    );

    filechooser_change_workspace_dir.connect_response(clone!(
        @weak edit_workspace_preview_row,
        @weak change_workspace_name_entryrow,
        @weak change_workspace_dir_label,
        @weak dialog_edit_workspace,
        @weak appwindow => move |filechooser, responsetype| {
        match responsetype {
            ResponseType::Accept => {
                if let Some(p) = filechooser.file().and_then(|f| f.path()) {
                    let path_string = p.to_string_lossy().to_string();
                    change_workspace_dir_label.set_label(&path_string);
                    edit_workspace_preview_row.entry().set_dir(path_string);

                    // Update the entry row with the file name of the new selected directory
                    if let Some(file_name) = p.file_name().map(|n| n.to_string_lossy()) {
                        change_workspace_name_entryrow.set_text(&file_name);
                    }
                } else {
                    change_workspace_dir_label.set_label(&gettext("- no directory selected -"));
                }
            }
            _ => {}
        }

        filechooser.hide();
        dialog_edit_workspace.show();
    }));

    dialog_edit_workspace.connect_response(
        clone!(@weak edit_workspace_preview_row, @weak appwindow => move |dialog_modify_workspace, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    // update the actual row
                    if let Some(current_row) = appwindow.workspacebrowser().current_selected_workspace_row() {
                        current_row.entry().replace_data(&edit_workspace_preview_row.entry());

                        // refreshing the files list
                        appwindow.workspacebrowser().refresh();
                        // And save the state
                        appwindow.workspacebrowser().save_workspaces_to_settings(&appwindow.app_settings());
                    }
                }
                _ => {}
            }

            dialog_modify_workspace.close();
        }));

    change_workspace_dir_button.connect_clicked(
        clone!(@weak dialog_edit_workspace, @weak filechooser_change_workspace_dir, @weak appwindow => move |_| {
            dialog_edit_workspace.hide();
            filechooser_change_workspace_dir.show();
        }),
    );

    dialog_edit_workspace.show();
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser_change_workspace_dir);
}
