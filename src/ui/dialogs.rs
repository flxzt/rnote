use gtk4::{glib, glib::clone, prelude::*, Builder};
use gtk4::{FileChooserDialog, FileFilter, MessageDialog, ResponseType, ShortcutsWindow};

use crate::ui::appwindow::RnoteAppWindow;
use crate::utils;
use crate::{app::RnoteApp, config};

pub fn dialog_shortcuts(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog_shortcuts: ShortcutsWindow = builder.object("dialog_shortcuts").unwrap();

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

pub fn dialog_open_sheet(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_open_file: FileChooserDialog = builder.object("dialog_open_file").unwrap();
    let filter = FileFilter::new();
    filter.add_pattern("*.rnote");

    dialog_open_file.set_transient_for(Some(appwindow));
    dialog_open_file.set_filter(&filter);

    dialog_open_file.connect_response(clone!(@weak appwindow => move |dialog_open_file, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    match dialog_open_file.file() {
                        Some(file) => {
                            *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().borrow_mut() = Some(file);
                            dialog_open_overwrite(&appwindow);
                        },
                        None => { log::error!("Can't open file. No file selected.")},
                    }

                    dialog_open_file.close();
                },
                _ => {
                dialog_open_file.close();
                }
            }

        }));

    dialog_open_file.show();
}

pub fn dialog_open_overwrite(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_open_input_file: MessageDialog = builder.object("dialog_open_input_file").unwrap();

    dialog_open_input_file.set_transient_for(Some(appwindow));

    dialog_open_input_file.connect_response(
        clone!(@weak appwindow => move |dialog_open_input_file, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    dialog_open_input_file.close();
                    if let Some(input_file) = appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().borrow().to_owned() {
                        if let Err(e) = appwindow.load_in_file(&input_file) {
                            log::error!("failed to load in input file, {}", e);
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

pub fn dialog_open_workspace(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_open_workspace: FileChooserDialog = builder.object("dialog_open_workspace").unwrap();

    dialog_open_workspace.set_transient_for(Some(appwindow));

    dialog_open_workspace.connect_response(
        clone!(@weak appwindow => move |dialog_open_workspace, responsetype| {
            match responsetype {
                ResponseType::Ok => {
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

                    dialog_open_workspace.close();
                }
                _ => {
                    dialog_open_workspace.close()
                }
            }
        }),
    );

    dialog_open_workspace.show();
}

pub fn dialog_save_sheet_as(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_save_sheet_as: FileChooserDialog = builder.object("dialog_save_sheet_as").unwrap();

    dialog_save_sheet_as.set_transient_for(Some(appwindow));

    dialog_save_sheet_as
        .set_current_name(format!("{}_{}_sheet.rnote", utils::now(), config::APP_NAME).as_str());

    dialog_save_sheet_as.connect_response(clone!(@weak appwindow => move |dialog_save_sheet_as, responsetype| {
        match responsetype {
            ResponseType::Ok => {
                match dialog_save_sheet_as.file() {
                    Some(file) => {
                        if let Err(e) = appwindow.canvas().sheet().save_sheet(&file) {
                            log::error!("failed to save sheet as, {}", e);
                            *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().borrow_mut() = None;
                        } else {
                            *appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().output_file().borrow_mut() = Some(file.clone());
                        }
                    },
                    None => { log::error!("Can't save file as. No file selected.")},
                }

                dialog_save_sheet_as.close();
            }
            _ => {
                dialog_save_sheet_as.close()
            }
        }
    }));

    dialog_save_sheet_as.show();
}

pub fn dialog_import_file(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_import_file: FileChooserDialog = builder.object("dialog_import_file").unwrap();
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_mime_type("image/png");
    filter.add_pattern("*.svg");
    filter.add_pattern("*.png");

    dialog_import_file.set_transient_for(Some(appwindow));
    dialog_import_file.set_filter(&filter);

    dialog_import_file.connect_response(
        clone!(@weak appwindow => move |dialog_import_file, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    match dialog_import_file.file() {
                        Some(file) => {
                            if appwindow.load_in_file(&file).is_err() {
                                log::error!("failed to load_in_file() on import");
                            }
                        },
                        None => { log::error!("unable to import file. No file selected.")},
                    }

                    dialog_import_file.close();
                }
                _ => {
                    dialog_import_file.close()
                }
            }
        }),
    );

    dialog_import_file.show();
}

pub fn dialog_export_selection(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_export_selection: FileChooserDialog =
        builder.object("dialog_export_selection").unwrap();
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");

    dialog_export_selection.set_transient_for(Some(appwindow));
    dialog_export_selection.set_filter(&filter);

    dialog_export_selection
        .set_current_name(format!("{}_{}_selection.svg", utils::now(), config::APP_NAME).as_str());

    dialog_export_selection.connect_response(clone!(@weak appwindow => move |dialog_export_selection, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    match dialog_export_selection.file() {
                        Some(file) => {
                            if let Err(e) = appwindow.canvas().sheet().selection().export_selection_as_svg(file) {
                                log::error!("exporting selection failed with error `{}`", e);
                            }
                        },
                        None => { log::error!("Unable to export selection. No file selected.")},
                    }

                    dialog_export_selection.close();
                }
                _ => {
                    dialog_export_selection.close()
                }
            }
        }));

    dialog_export_selection.show();
}

pub fn dialog_export_sheet(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_export_sheet: FileChooserDialog = builder.object("dialog_export_sheet").unwrap();
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");

    dialog_export_sheet.set_transient_for(Some(appwindow));
    dialog_export_sheet.set_filter(&filter);

    dialog_export_sheet
        .set_current_name(format!("{}_{}_sheet.svg", utils::now(), config::APP_NAME).as_str());

    dialog_export_sheet.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Ok => {
                    dialog_export_sheet.close();
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
                    dialog_export_sheet.close()
                }
            }
        }),
    );

    dialog_export_sheet.show();
}
