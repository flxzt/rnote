// gtk4::Dialog is deprecated, but the replacement adw::ToolbarView is not suitable for a async flow
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
    gio, glib, glib::clone, Builder, Button, CheckButton, ColorDialogButton, FileDialog, Label,
    MenuButton, ShortcutsWindow, StringList,
};

// About Dialog
pub(crate) fn dialog_about(appwindow: &RnAppWindow) {
    let app_icon_name = if config::PROFILE == "devel" {
        config::APP_NAME.to_string() + "-devel"
    } else {
        config::APP_NAME.to_string()
    };

    let aboutdialog = adw::AboutDialog::builder()
        .application_name(config::APP_NAME_CAPITALIZED)
        .application_icon(app_icon_name)
        .comments(gettext("Sketch and take handwritten notes"))
        .website(config::APP_WEBSITE)
        .issue_url(config::APP_ISSUES_URL)
        .support_url(config::APP_SUPPORT_URL)
        .developer_name(config::APP_AUTHOR_NAME)
        .developers(config::APP_AUTHORS.lines().collect::<Vec<&str>>())
        // TRANSLATORS: 'Name <email@domain.com>' or 'Name https://website.example'
        .translator_credits(gettext("translator-credits"))
        .license_type(globals::APP_LICENSE)
        .version((String::from(config::APP_VERSION) + config::APP_VERSION_SUFFIX).as_str())
        .build();

    if config::PROFILE == "devel" {
        aboutdialog.add_css_class("devel");
    }

    aboutdialog.present(appwindow);
}

pub(crate) fn dialog_keyboard_shortcuts(appwindow: &RnAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog: ShortcutsWindow = builder.object("shortcuts_window").unwrap();
    dialog.set_transient_for(Some(appwindow));
    dialog.present();
}

pub(crate) async fn dialog_clear_doc(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::AlertDialog = builder.object("dialog_clear_doc").unwrap();

    match dialog.choose_future(appwindow).await.as_str() {
        "clear" => {
            let prev_empty = canvas.empty();

            let widget_flags = canvas.engine_mut().clear();
            appwindow.handle_widget_flags(widget_flags, canvas);

            if !prev_empty {
                canvas.set_unsaved_changes(true);
                canvas.set_empty(true);
            }
        }
        _ => {
            // Cancel
        }
    }
}

#[allow(unused)]
pub(crate) async fn dialog_new_doc(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::AlertDialog = builder.object("dialog_new_doc").unwrap();

    let new_doc = |appwindow: &RnAppWindow, canvas: &RnCanvas| {
        let widget_flags = canvas.engine_mut().clear();
        appwindow.handle_widget_flags(widget_flags, canvas);

        canvas.set_unsaved_changes(false);
        canvas.set_empty(true);
        canvas.set_output_file(None);
    };

    if !canvas.unsaved_changes() {
        new_doc(appwindow, canvas);
        return;
    }

    match dialog.choose_future(appwindow).await.as_str() {
        "discard" => {
            new_doc(appwindow, canvas);
        }
        "save" => {
            glib::spawn_future_local(clone!(@weak canvas, @weak appwindow => async move {
                if let Some(output_file) = canvas.output_file() {
                    appwindow.overlays().progressbar_start_pulsing();

                    if let Err(e) = canvas.save_document_to_file(&output_file).await {
                        tracing::error!("Saving document failed before creating new document, Err: {e:?}");

                        canvas.set_output_file(None);
                        appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed"));
                        appwindow.overlays().progressbar_abort();
                    } else {
                        appwindow.overlays().progressbar_finish();
                    }
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
        }
        _ => {
            // Cancel
        }
    }
}

/// Only to be called from the tabview close-page handler
///
/// Returns `close_finish_confirm` that should be passed into close_page_finish() and indicates if the tab should be
/// actually closed or closing should be aborted.
#[must_use]
pub(crate) async fn dialog_close_tab(appwindow: &RnAppWindow, tab_page: &adw::TabPage) -> bool {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::AlertDialog = builder.object("dialog_close_tab").unwrap();
    let file_group: adw::PreferencesGroup = builder.object("close_tab_file_group").unwrap();
    let canvas = tab_page
        .child()
        .downcast::<RnCanvasWrapper>()
        .unwrap()
        .canvas();
    let canvas_output_file = canvas.output_file();

    let mut save_file = canvas_output_file.clone();
    let save_folder_path = if let Some(p) = canvas
        .output_file()
        .as_ref()
        .and_then(|f| f.parent()?.path())
    {
        Some(p)
    } else {
        appwindow.sidebar().workspacebrowser().dir_list_dir()
    };

    // Handle possible file collisions for new files
    if save_file.is_none() {
        if let Some(save_folder_path) = save_folder_path.as_ref() {
            let base_title = canvas.doc_title_display();
            let mut test_save_file =
                gio::File::for_path(save_folder_path.join(base_title.clone() + ".rnote"));
            let mut doc_postfix = 0;

            // increment as long as as files with same name exist
            while gio::File::query_exists(&test_save_file, gio::Cancellable::NONE) {
                doc_postfix += 1;
                test_save_file = gio::File::for_path(save_folder_path.join(
                    base_title.clone()
                        + crate::utils::FILE_DUP_SUFFIX_DELIM
                        + &doc_postfix.to_string()
                        + ".rnote",
                ));
            }
            save_file = Some(test_save_file);
        }
    }

    let save_file_display_name = save_file
        .as_ref()
        .and_then(|f| Some(f.path()?.file_stem()?.to_string_lossy().to_string()))
        .unwrap_or_else(|| gettext("- invalid file name -"));
    let save_folder_display_name = save_file
        .as_ref()
        .and_then(|f| {
            f.parent()
                .and_then(|p| Some(p.path()?.display().to_string()))
        })
        .unwrap_or_else(|| gettext("- invalid save folder name -"));

    let row = adw::ActionRow::builder()
        .title(save_file_display_name)
        .subtitle(save_folder_display_name)
        .subtitle_lines(2)
        .build();

    let check = CheckButton::builder().active(true).build();
    // Lock checkbox to active state as user can click "discard" if they choose
    check.set_sensitive(false);
    let prefix_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    prefix_box.append(&check);
    if canvas_output_file.is_some() {
        // Indicate that a new existing file will be saved
        let icon_image = gtk4::Image::from_icon_name("doc-save-symbolic");
        icon_image.set_tooltip_text(Some(&gettext("The changes will be saved")));
        prefix_box.append(&icon_image);
    } else {
        // Indicate that a new file will be created
        let icon_image = gtk4::Image::from_icon_name("doc-create-symbolic");
        icon_image.set_tooltip_text(Some(&gettext("A new file will be created")));
        prefix_box.append(&icon_image);
    }
    row.add_prefix(&prefix_box);
    if save_file.is_none() {
        // Indicate that the file cannot be saved
        check.set_active(false);
        row.set_sensitive(false);
    }
    file_group.add(&row);

    // Returns close_finish_confirm, a boolean that indicates if the tab should actually be closed or closing
    // should be aborted.
    match dialog.choose_future(appwindow).await.as_str() {
        "discard" => true,
        "save" => {
            if let Some(save_file) = save_file {
                appwindow.overlays().progressbar_start_pulsing();

                if let Err(e) = canvas.save_document_to_file(&save_file).await {
                    canvas.set_output_file(None);

                    tracing::error!("Saving document failed before closing tab, Err: {e:?}");
                    appwindow
                        .overlays()
                        .dispatch_toast_error(&gettext("Saving document failed"));
                    appwindow.overlays().progressbar_abort();
                } else {
                    appwindow.overlays().progressbar_finish();
                }

                appwindow.overlays().progressbar_finish();
                // No success toast on saving without dialog, success is already indicated in the header title
            }

            // only close if saving was successful
            !canvas.unsaved_changes()
        }
        _ => {
            // Cancel
            false
        }
    }
}

pub(crate) async fn dialog_close_window(appwindow: &RnAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::AlertDialog = builder.object("dialog_close_window").unwrap();
    let files_group: adw::PreferencesGroup = builder.object("close_window_files_group").unwrap();

    let tabs = appwindow.tabs_snapshot();
    let mut rows = Vec::new();
    let mut doc_postfix = 0;
    for (i, tab) in tabs.iter().enumerate() {
        let canvas = tab.child().downcast::<RnCanvasWrapper>().unwrap().canvas();
        let canvas_output_file = canvas.output_file();

        if !canvas.unsaved_changes() {
            continue;
        }

        let mut save_file = canvas_output_file.clone();
        let save_folder_path = if let Some(p) = canvas
            .output_file()
            .as_ref()
            .and_then(|f| f.parent()?.path())
        {
            Some(p)
        } else {
            appwindow.sidebar().workspacebrowser().dir_list_dir()
        };

        // Handle possible file collisions for new files
        if canvas_output_file.is_none() {
            if let Some(save_folder_path) = save_folder_path.as_ref() {
                let base_title = canvas.doc_title_display();
                let mut test_save_file = if doc_postfix == 0 {
                    gio::File::for_path(save_folder_path.join(base_title.clone() + ".rnote"))
                } else {
                    gio::File::for_path(save_folder_path.join(
                        base_title.clone()
                            + crate::utils::FILE_DUP_SUFFIX_DELIM
                            + &doc_postfix.to_string()
                            + ".rnote",
                    ))
                };

                // increment as long as as files with same name exist
                while gio::File::query_exists(&test_save_file, gio::Cancellable::NONE) {
                    doc_postfix += 1;
                    test_save_file = gio::File::for_path(save_folder_path.join(
                        base_title.clone()
                            + crate::utils::FILE_DUP_SUFFIX_DELIM
                            + &doc_postfix.to_string()
                            + ".rnote",
                    ));
                }
                save_file = Some(test_save_file);
                // increment for next iteration
                doc_postfix += 1;
            }
        }

        let save_file_display_name = save_file
            .as_ref()
            .and_then(|f| Some(f.path()?.file_stem()?.to_string_lossy().to_string()))
            .unwrap_or_else(|| gettext("- invalid file name -"));
        let save_folder_display_name = save_file
            .as_ref()
            .and_then(|f| {
                f.parent()
                    .and_then(|p| Some(p.path()?.display().to_string()))
            })
            .unwrap_or_else(|| gettext("- invalid save folder name -"));

        let row = adw::ActionRow::builder()
            .title(save_file_display_name)
            .subtitle(save_folder_display_name)
            .subtitle_lines(2)
            .build();

        // Checkbox active by default
        let check = CheckButton::builder().active(true).build();
        let prefix_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        prefix_box.append(&check);
        if canvas_output_file.is_some() {
            // Indicate that a new existing file will be saved
            let icon_image = gtk4::Image::from_icon_name("doc-save-symbolic");
            icon_image.set_tooltip_text(Some(&gettext("The changes will be saved")));
            prefix_box.append(&icon_image);
        } else {
            // Indicate that a new file will be created
            let icon_image = gtk4::Image::from_icon_name("doc-create-symbolic");
            icon_image.set_tooltip_text(Some(&gettext("A new file will be created")));
            prefix_box.append(&icon_image);
        }
        row.add_prefix(&prefix_box);
        if save_file.is_none() {
            // Indicate that the file cannot be saved
            check.set_active(false);
            row.set_sensitive(false);
        }
        files_group.add(&row);

        rows.push((i, check, save_file));
    }

    let close = match dialog.choose_future(appwindow).await.as_str() {
        "discard" => {
            // do nothing and close
            true
        }
        "save" => {
            let mut close = true;
            appwindow.overlays().progressbar_start_pulsing();

            for (i, check, save_file) in rows {
                if !check.is_active() {
                    continue;
                }
                let Some(save_file) = save_file else {
                    continue;
                };
                let canvas = tabs[i]
                    .child()
                    .downcast::<RnCanvasWrapper>()
                    .unwrap()
                    .canvas();

                if let Err(e) = canvas.save_document_to_file(&save_file).await {
                    tracing::error!("Saving document failed before closing window, Err: `{e:?}`");

                    close = false;
                    canvas.set_output_file(None);
                    appwindow
                        .overlays()
                        .dispatch_toast_error(&gettext("Saving document failed"));
                }
            }

            appwindow.overlays().progressbar_finish();
            close
        }
        _ => {
            // Cancel
            false
        }
    };

    if close {
        appwindow.close_force();
    }
}

pub(crate) async fn dialog_edit_selected_workspace(appwindow: &RnAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::Dialog = builder.object("dialog_edit_selected_workspace").unwrap();
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
    let edit_selected_workspace_button_cancel: Button = builder
        .object("edit_selected_workspace_button_cancel")
        .unwrap();
    let edit_selected_workspace_button_apply: Button = builder
        .object("edit_selected_workspace_button_apply")
        .unwrap();

    preview_row.init(appwindow);

    // Sets the icons
    icon_picker.set_list(
        StringList::new(WORKSPACELISTENTRY_ICONS_LIST),
        Some(workspacelistentry_icons_list_to_display_name),
        false,
    );

    let Some(initial_entry) = appwindow
        .sidebar()
        .workspacebrowser()
        .workspacesbar()
        .selected_workspacelistentry()
    else {
        tracing::warn!("Tried to edit workspace entry in dialog, but no workspace is selected.");
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
        clone!(@weak preview_row, @weak dir_label, @weak name_entryrow, @weak dialog, @weak appwindow => move |_| {
            glib::spawn_future_local(clone!(@weak preview_row, @weak dir_label, @weak name_entryrow, @weak dialog, @weak appwindow => async move {
                dialog.set_sensitive(false);

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
                        tracing::debug!("Did not select new folder for workspacerow (Error or dialog dismissed by user), Err: {e:?}");
                    }
                }

                dialog.set_sensitive(true);
            }));
        }),
    );

    // Listen to responses

    edit_selected_workspace_button_cancel.connect_clicked(clone!(@weak dialog => move |_| {
        dialog.close();
    }));

    edit_selected_workspace_button_apply.connect_clicked(
        clone!(@weak preview_row, @weak dialog, @weak appwindow => move |_| {
            dialog.close();

            // update the actual selected entry
            appwindow
                .sidebar()
                .workspacebrowser()
                .workspacesbar()
                .replace_selected_workspacelistentry(preview_row.entry());
            // refreshing the files list
            appwindow
                .sidebar()
                .workspacebrowser()
                .refresh_dir_list_selected_workspace();
        }),
    );

    dialog.present(appwindow);
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
