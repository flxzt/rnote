use gtk4::{glib, glib::clone, prelude::*, Builder};
use gtk4::{
    AboutDialog, FileChooserAction, FileChooserNative, FileFilter, MessageDialog, ResponseType,
    ShortcutsWindow,
};

use crate::ui::appwindow::RnoteAppWindow;
use crate::utils;
use crate::{app::RnoteApp, config};

// About Dialog
pub fn dialog_about(appwindow: &RnoteAppWindow) {
    let aboutdialog = AboutDialog::builder()
        .modal(true)
        .transient_for(appwindow)
        .program_name("Rnote")
        .comments("Create handwritten notes")
        .logo_icon_name(config::APP_ID)
        .website(config::APP_WEBSITE)
        .authors(
            config::APP_AUTHORS
                .iter()
                .map(|&s| String::from(s))
                .collect(),
        )
        .license_type(config::APP_LICENSE)
        .version((String::from(config::APP_VERSION) + config::APP_VERSION_SUFFIX).as_str())
        .build();

    aboutdialog.show();
}

// Message Dialogs

pub fn dialog_keyboard_shortcuts(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog_shortcuts: ShortcutsWindow = builder.object("shortcuts_window").unwrap();

    dialog_shortcuts.set_transient_for(Some(appwindow));
    dialog_shortcuts.show();
}

pub fn dialog_clear_sheet(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_clear_sheet: MessageDialog = builder.object("dialog_clear_sheet").unwrap();

    dialog_clear_sheet.set_transient_for(Some(appwindow));

    dialog_clear_sheet.connect_response(
        clone!(@weak appwindow => move |dialog_clear_sheet, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    appwindow.canvas().sheet().clear();
                    appwindow.canvas().queue_resize();
                    appwindow.canvas().queue_draw();
                    appwindow.canvas().set_unsaved_changes(false);

                    dialog_clear_sheet.close();
                },
                _ => {
                    dialog_clear_sheet.close();
                }
            }
        }),
    );

    dialog_clear_sheet.show();
}

pub fn dialog_new_sheet(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_new_sheet: MessageDialog = builder.object("dialog_new_sheet").unwrap();

    dialog_new_sheet.set_transient_for(Some(appwindow));
    dialog_new_sheet.connect_response(clone!(@weak appwindow => move |dialog_new_sheet, responsetype| {
        match responsetype {
            ResponseType::Ok => {
                *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().borrow_mut() = None;
                *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().borrow_mut() = None;

                appwindow.canvas().sheet().clear();
                appwindow.canvas().queue_resize();
                appwindow.canvas().queue_draw();
                appwindow.canvas().sheet().selection().set_shown(false);
                appwindow.canvas().set_unsaved_changes(false);

                dialog_new_sheet.close();
            },
            ResponseType::Apply => {
                dialog_save_sheet_as(&appwindow);
            }
            _ => {
                dialog_new_sheet.close();
            }
        }
    }));

    dialog_new_sheet.show();
}

pub fn dialog_quit_save(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_quit_save: MessageDialog = builder.object("dialog_quit_save").unwrap();

    dialog_quit_save.set_transient_for(Some(appwindow));

    dialog_quit_save.connect_response(
        clone!(@weak appwindow => move |dialog_quit_save, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    dialog_quit_save.close();
                    appwindow.destroy();
                },
                ResponseType::Apply => {
                    dialog_save_sheet_as(&appwindow);
                }
                _ => {
                    dialog_quit_save.close();
                }
            }
        }),
    );

    dialog_quit_save.show();
}

pub fn dialog_open_overwrite(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_open_input_file: MessageDialog = builder.object("dialog_open_overwrite").unwrap();

    dialog_open_input_file.set_transient_for(Some(appwindow));

    dialog_open_input_file.connect_response(
        clone!(@weak appwindow => move |dialog_open_input_file, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    dialog_open_input_file.close();
                    if let Some(input_file) = appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().borrow().to_owned() {
                        if let Err(e) = appwindow.load_in_file(&input_file) {
                            log::error!("failed to load in input file, {}", e);
                        } else {
                            appwindow.canvas().set_unsaved_changes(false);
                        }
                    }
                },
                ResponseType::Apply => {
                    dialog_save_sheet_as(&appwindow);
                }
                _ => {
                    dialog_open_input_file.close();
                }
            }
        }),
    );

    dialog_open_input_file.show();
}

// FileChooserNative Dialogs

pub fn dialog_open_sheet(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_pattern("*.rnote");
    filter.set_name(Some(".rnote file"));

    let dialog_open_file: FileChooserNative = FileChooserNative::builder()
        .title("Open File")
        .modal(true)
        .transient_for(appwindow)
        .accept_label("Open")
        .cancel_label("Cancel")
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .build();

    dialog_open_file.add_filter(&filter);

    dialog_open_file.connect_response(clone!(@weak appwindow => move |dialog_open_file, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_open_file.file() {
                        *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().borrow_mut() = Some(file);
                        appwindow.canvas().set_unsaved_changes(false);

                        dialog_open_overwrite(&appwindow);
                    } else {
                        log::error!("Can't open file. No file selected.");
                    };
                },
                _ => {
                }
            }

        }));

    dialog_open_file.show();

    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_open_file);
}

pub fn dialog_open_workspace(appwindow: &RnoteAppWindow) {
    let dialog_open_workspace: FileChooserNative = FileChooserNative::builder()
        .title("Open Workspace")
        .modal(true)
        .transient_for(appwindow)
        .accept_label("Open")
        .cancel_label("Cancel")
        .action(FileChooserAction::SelectFolder)
        .select_multiple(false)
        .build();

    dialog_open_workspace.connect_response(
        clone!(@weak appwindow => move |dialog_open_workspace, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_open_workspace.file() {
                        Some(file) => {
                            if let Some(workspace_path) = file.path() {
                                appwindow.workspacebrowser().set_primary_path(&workspace_path);
                            } else {
                                log::error!("Can't open workspace. not a valid path.")
                            }
                        },
                        None => { log::error!("Can't open workspace. Nothing selected.")},
                    }

                }
                _ => {
                }
            }
        }),
    );

    dialog_open_workspace.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_open_workspace);
}

pub fn dialog_save_sheet_as(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_pattern("*.rnote");
    filter.set_name(Some(".rnote file"));

    let dialog_save_sheet_as: FileChooserNative = FileChooserNative::builder()
        .title("Save Sheet As")
        .modal(true)
        .transient_for(appwindow)
        .accept_label("Save As")
        .cancel_label("Cancel")
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    dialog_save_sheet_as.add_filter(&filter);

    dialog_save_sheet_as
        .set_current_name(format!("{}_{}_sheet.rnote", utils::now(), config::APP_NAME).as_str());

    dialog_save_sheet_as.connect_response(clone!(@weak appwindow => move |dialog_save_sheet_as, responsetype| {
        match responsetype {
            ResponseType::Accept => {
                match dialog_save_sheet_as.file() {
                    Some(file) => {
                        if let Err(e) = appwindow.canvas().sheet().save_sheet(&file) {
                            log::error!("failed to save sheet as, {}", e);
                            *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().borrow_mut() = None;
                        } else {
                            *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().borrow_mut() = Some(file.clone());
                            appwindow.canvas().set_unsaved_changes(false);
                        }
                    },
                    None => { log::error!("Can't save file as. No file selected.")},
                }
            }
            _ => {
            }
        }
    }));

    dialog_save_sheet_as.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_save_sheet_as);
}

pub fn dialog_import_file(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    filter.add_pattern("*.svg");
    filter.add_pattern("*.png");
    filter.add_pattern("*.jpg");
    filter.set_name(Some("PNG / SVG / JPG file"));

    let dialog_import_file: FileChooserNative = FileChooserNative::builder()
        .title("Import File")
        .modal(true)
        .transient_for(appwindow)
        .accept_label("Import")
        .cancel_label("Cancel")
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .build();

    dialog_import_file.add_filter(&filter);

    dialog_import_file.connect_response(
        clone!(@weak appwindow => move |dialog_import_file, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_import_file.file() {
                        Some(file) => {
                            if appwindow.load_in_file(&file).is_err() {
                                log::error!("failed to load_in_file() on import");
                            }
                        },
                        None => { log::error!("unable to import file. No file selected.")},
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_import_file.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_import_file);
}

pub fn dialog_export_selection(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");
    filter.set_name(Some("SVG file"));

    let dialog_export_selection: FileChooserNative = FileChooserNative::builder()
        .title("Export Selection")
        .modal(true)
        .transient_for(appwindow)
        .accept_label("Export")
        .cancel_label("Cancel")
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_selection.add_filter(&filter);

    dialog_export_selection
        .set_current_name(format!("{}_{}_selection.svg", utils::now(), config::APP_NAME).as_str());

    dialog_export_selection.connect_response(clone!(@weak appwindow => move |dialog_export_selection, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_export_selection.file() {
                        Some(file) => {
                            if let Err(e) = appwindow.canvas().sheet().selection().export_selection_as_svg(file) {
                                log::error!("exporting selection failed with error `{}`", e);
                            }
                        },
                        None => { log::error!("Unable to export selection. No file selected.")},
                    }
                }
                _ => {
                }
            }
        }));

    dialog_export_selection.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_selection);
}

pub fn dialog_export_sheet(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");
    filter.set_name(Some("SVG file"));

    let dialog_export_sheet: FileChooserNative = FileChooserNative::builder()
        .title("Export Sheet")
        .modal(true)
        .transient_for(appwindow)
        .accept_label("Export")
        .cancel_label("Cancel")
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_sheet.add_filter(&filter);

    dialog_export_sheet
        .set_current_name(format!("{}_{}_sheet.svg", utils::now(), config::APP_NAME).as_str());

    dialog_export_sheet.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_export_sheet.file() {
                        Some(file) => {
                            if let Err(e) = appwindow.canvas().sheet().export_sheet_as_svg(file) {
                                log::error!("exporting sheet failed with error `{}`", e);
                            }
                        },
                        None => { log::error!("Can't export sheet. No file selected.")},
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_export_sheet.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_sheet);
}
