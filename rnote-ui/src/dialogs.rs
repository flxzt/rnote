use gettextrs::gettext;
use gtk4::{
    gio, AboutDialog, FileChooserAction, FileChooserNative, FileFilter, MessageDialog,
    ResponseType, ShortcutsWindow,
};
use gtk4::{glib, glib::clone, prelude::*, Builder};

use crate::appwindow::RnoteAppWindow;
use crate::utils;
use crate::{app::RnoteApp, config};

// About Dialog
pub fn dialog_about(appwindow: &RnoteAppWindow) {
    let aboutdialog = AboutDialog::builder()
        .modal(true)
        .transient_for(appwindow)
        .program_name(config::APP_NAME_CAPITALIZED)
        .comments(&gettext(
            "A simple drawing application to create handwritten notes",
        ))
        .logo_icon_name(config::APP_ID)
        .website(config::APP_WEBSITE)
        .authors(
            config::APP_AUTHORS
                .iter()
                .map(|&s| String::from(s))
                .collect(),
        )
        // TRANSLATORS: 'Name <email@domain.com>' or 'Name https://website.example'
        .translator_credits(&gettext("translator-credits"))
        .license_type(config::APP_LICENSE)
        .version((String::from(config::APP_VERSION) + config::APP_VERSION_SUFFIX).as_str())
        .build();

    if config::PROFILE == "devel" {
        aboutdialog.add_css_class("devel");
    }

    aboutdialog.show();
}

// Message Dialogs

pub fn dialog_keyboard_shortcuts(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog_shortcuts: ShortcutsWindow = builder.object("shortcuts_window").unwrap();

    if config::PROFILE == "devel" {
        dialog_shortcuts.add_css_class("devel");
    }

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
                    appwindow.canvas().engine().borrow_mut().strokes_state.clear();
                    appwindow.canvas().selection_modifier().update_state(&appwindow.canvas());
                    appwindow.canvas().set_empty(true);

                    appwindow.canvas().regenerate_background(false);
                    appwindow.canvas().regenerate_content(true, true);

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
                appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_input_file(None);
                appwindow.set_output_file(None, &appwindow);

                appwindow.canvas().engine().borrow_mut().strokes_state.clear();
                appwindow.canvas().selection_modifier().update_state(&appwindow.canvas());
                appwindow.canvas().set_unsaved_changes(false);
                appwindow.canvas().set_empty(true);

                appwindow.canvas().update_background_rendernode(false);
                appwindow.canvas().regenerate_content(true, true);

                dialog_new_sheet.close();
            },
            ResponseType::Apply => {
                dialog_new_sheet.close();
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
                    appwindow.close();
                },
                ResponseType::Apply => {
                    dialog_quit_save.close();
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
                    if let Some(input_file) = appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().as_ref() {
                        if let Err(e) = appwindow.load_in_file(input_file, None) {
                            log::error!("failed to load in input file, {}", e);
                        }
                    }
                },
                ResponseType::Apply => {
                    dialog_open_input_file.close();
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
    filter.add_mime_type("application/rnote");
    filter.add_mime_type("application/x-xopp");
    filter.add_pattern("*.rnote");
    filter.set_name(Some(&gettext(".rnote / .xopp File")));

    let dialog_open_file: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Open file"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Open"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .build();

    dialog_open_file.add_filter(&filter);

    dialog_open_file.connect_response(clone!(@weak appwindow => move |dialog_open_file, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_open_file.file() {
                        appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_input_file(Some(file));
                        appwindow.canvas().set_unsaved_changes(false);

                        dialog_open_overwrite(&appwindow);
                    } else {
                        log::error!("Can't open file. No file selected.");
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening sheet failed").to_variant()));
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
        .title(&gettext("Open workspace"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Open"))
        .cancel_label(&gettext("Cancel"))
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
                                appwindow.workspacebrowser().set_primary_path(Some(&workspace_path));
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
    filter.add_mime_type("application/rnote");
    filter.add_pattern("*.rnote");
    filter.set_name(Some(&gettext(".rnote file")));

    let dialog_save_sheet_as: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Save sheet as"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Save as"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    dialog_save_sheet_as.add_filter(&filter);

    dialog_save_sheet_as.set_current_name(
        format!(
            "{}_sheet.rnote",
            rnote_engine::utils::now_formatted_string()
        )
        .as_str(),
    );

    dialog_save_sheet_as.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_export_sheet.file() {
                        Some(file) => {
                            match file.basename() {
                                Some(basename) => {
                                    match appwindow.canvas().engine().borrow().save_sheet_as_rnote_bytes(&basename.to_string_lossy()) {
                                        Ok(bytes) => {
                                            let main_cx = glib::MainContext::default();

                                            main_cx.spawn_local(clone!(@weak appwindow => async move {
                                                let result = file.replace_future(None, false, gio::FileCreateFlags::REPLACE_DESTINATION, glib::PRIORITY_HIGH_IDLE).await;
                                                match result {
                                                    Ok(output_stream) => {
                                                        if let Err(e) = output_stream.write(&bytes, None::<&gio::Cancellable>) {
                                                            log::error!("output_stream().write() failed in save_sheet_as() with Err {}",e);
                                                            return;
                                                        };
                                                        if let Err(e) = output_stream.close(None::<&gio::Cancellable>) {
                                                            log::error!("output_stream().close() failed in save_sheet_as() with Err {}",e);
                                                            return;
                                                        };

                                                        appwindow.set_output_file(Some(&file), &appwindow);
                                                        appwindow.canvas().set_unsaved_changes(false);
                                                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Saved sheet successfully").to_variant()));
                                                    }
                                                    Err(e) => {
                                                        log::error!("file.replace_future() in save_sheet_as() returned Err {}",e);
                                                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving sheet failed").to_variant()));
                                                    }
                                                }
                                            }));
                                        },
                                        Err(e) => log::error!("saving sheet as .rnote failed with error `{}`", e),
                                    }
                                }
                                None => {
                                    log::error!("basename for file is None while trying to save sheet as .rnote");
                                }
                            }
                        },
                        None => { log::error!("Can't save sheet as .rnote. No file selected.")},
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_save_sheet_as.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_save_sheet_as);
}

pub fn dialog_import_file(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    filter.add_mime_type("application/pdf");
    filter.add_pattern("*.svg");
    filter.add_pattern("*.png");
    filter.add_pattern("*.jpg");
    filter.add_pattern("*.pdf");
    filter.set_name(Some(&gettext("PNG / SVG / JPG / PDF file")));

    let dialog_import_file: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Import file"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Import"))
        .cancel_label(&gettext("Cancel"))
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
                            if let Err(e) = appwindow.load_in_file(&file, None) {
                                log::error!("failed to load_in_file() on import, {}", e);
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
    filter.set_name(Some(&gettext("SVG file")));

    let dialog_export_selection: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Selection"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_selection.add_filter(&filter);

    dialog_export_selection.set_current_name(
        format!(
            "{}_selection.svg",
            rnote_engine::utils::now_formatted_string()
        )
        .as_str(),
    );

    dialog_export_selection.connect_response(clone!(@weak appwindow => move |dialog_export_selection, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_export_selection.file() {
                        Some(file) => {
                            if let Err(e) = appwindow.canvas().engine().borrow().strokes_state.export_selection_as_svg(file) {
                                log::error!("exporting selection failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export selection as SVG failed").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported selection as SVG successfully").to_variant()));
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

pub fn dialog_export_sheet_as_svg(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");
    filter.set_name(Some(&gettext("SVG file")));

    let dialog_export_sheet: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Sheet"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_sheet.add_filter(&filter);

    dialog_export_sheet.set_current_name(
        format!("{}_sheet.svg", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_sheet.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_export_sheet.file() {
                        Some(file) => {
                            if let Err(e) = appwindow.export_sheet_as_svg(&file) {
                                log::error!("exporting sheet failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export sheet as SVG failed").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported sheet as SVG successfully").to_variant()));
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

pub fn dialog_export_sheet_as_pdf(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/pdf");
    filter.add_pattern("*.pdf");
    filter.set_name(Some(&gettext("PDF file")));

    let dialog_export_sheet: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Sheet"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_sheet.add_filter(&filter);

    dialog_export_sheet.set_current_name(
        format!("{}_sheet.pdf", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_sheet.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_export_sheet.file() {
                        Some(file) => {
                            let main_cx = glib::MainContext::default();
                            main_cx.spawn_local(clone!(@weak appwindow, @strong file => async move {
                                if let Err(e) = appwindow.export_sheet_as_pdf(&file).await {
                                    log::error!("export_sheet_as_pdf() failed in dialog_export_sheet() with Err {}", e);
                                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export sheet as PDF failed").to_variant()));
                                } else {
                                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported sheet as PDF successfully").to_variant()));
                                };
                            }));
                        },
                        None => { log::error!("Can't export sheet as pdf. No file selected.")},
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

pub fn dialog_export_sheet_as_xopp(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/x-xopp");
    filter.add_pattern("*.xopp");
    filter.set_name(Some(&gettext(".xopp file")));

    let dialog_export_sheet: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Sheet"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_sheet.add_filter(&filter);

    dialog_export_sheet.set_current_name(
        format!("{}_sheet.xopp", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_sheet.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    match dialog_export_sheet.file() {
                        Some(file) => {
                            match file.basename() {
                                Some(basename) => {
                                    match appwindow.canvas().engine().borrow().export_sheet_as_xopp_bytes(&basename.to_string_lossy()) {
                                        Ok(bytes) => {
                                            if let Err(e) = utils::replace_file_async(bytes, &file) {
                                                log::error!("exporting sheet as .xopp failed, replace_file_async failed with Err {}", e);
                                            }
                                        },
                                        Err(e) => log::error!("exporting sheet as .xopp failed with error `{}`", e),
                                    }
                                }
                                None => {
                                    log::error!("basename for file is None while trying to export sheet as .xopp");
                                }
                            }
                        },
                        None => { log::error!("Can't export sheet as .xopp. No file selected.")},
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
