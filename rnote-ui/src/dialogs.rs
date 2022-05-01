use gettextrs::gettext;
use gtk4::{glib, glib::clone, prelude::*, Builder};
use gtk4::{
    AboutDialog, FileChooserAction, FileChooserNative, FileFilter, MessageDialog, ResponseType,
    ShortcutsWindow,
};

use crate::appwindow::RnoteAppWindow;
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
                    dialog_clear_sheet.close();

                    appwindow.canvas().engine().borrow_mut().store.clear();
                    appwindow.canvas().set_unsaved_changes(false);
                    appwindow.canvas().set_empty(true);

                    appwindow.canvas().regenerate_background(false);
                    appwindow.canvas().regenerate_content(true, true);
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
                dialog_new_sheet.close();

                appwindow.canvas().engine().borrow_mut().store.clear();
                appwindow.canvas().set_unsaved_changes(false);
                appwindow.canvas().set_empty(true);
                appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_input_file(None);
                appwindow.canvas().set_output_file(None);

                appwindow.canvas().regenerate_background(false);
                appwindow.canvas().regenerate_content(true, true);

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
                    appwindow.close_force();
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
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening file failed.").to_variant()));
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
                    }
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
                    if let Some(file) = dialog_open_workspace.file() {
                        if let Some(workspace_path) = file.path() {
                            appwindow.workspacebrowser().set_primary_path(Some(&workspace_path));
                        }
                    }

                }
                _ => {}
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
                    if let Some(file) = dialog_export_sheet.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                            appwindow.canvas_progressbar().pulse();

                            if let Err(e) = appwindow.save_sheet_to_file(&file).await {
                                appwindow.canvas().set_output_file(None);

                                log::error!("saving sheet failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving sheet failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Saved sheet successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
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
                    if let Some(file) = dialog_import_file.file() {
                        if let Err(e) = appwindow.load_in_file(&file, None) {
                            log::error!("load_in_file() failed while import file, Err {}", e);
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Importing file failed.").to_variant()));
                        }
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
                    if let Some(file) = dialog_export_selection.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                            appwindow.canvas_progressbar().pulse();

                            if let Err(e) = appwindow.export_selection_as_svg(&file).await {
                                log::error!("exporting selection failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export selection as SVG failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported selection as SVG successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
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

    let dialog_export_sheet_as_svg: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Sheet"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_sheet_as_svg.add_filter(&filter);

    dialog_export_sheet_as_svg.set_current_name(
        format!("{}_sheet.svg", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_sheet_as_svg.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_sheet.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                            appwindow.canvas_progressbar().pulse();

                            if let Err(e) = appwindow.export_sheet_as_svg(&file).await {
                                log::error!("exporting sheet failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export sheet as SVG failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported sheet as SVG successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_export_sheet_as_svg.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_sheet_as_svg);
}

pub fn dialog_export_sheet_as_pdf(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/pdf");
    filter.add_pattern("*.pdf");
    filter.set_name(Some(&gettext("PDF file")));

    let dialog_export_sheet_as_pdf: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Sheet"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_sheet_as_pdf.add_filter(&filter);

    dialog_export_sheet_as_pdf.set_current_name(
        format!("{}_sheet.pdf", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_sheet_as_pdf.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_sheet.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak appwindow, @strong file => async move {
                            appwindow.canvas_progressbar().pulse();

                            if let Err(e) = appwindow.export_sheet_as_pdf(&file).await {
                                log::error!("export_sheet_as_pdf() failed in export dialog with Err {}", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export sheet as PDF failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported sheet as PDF successfully.").to_variant()));
                            };

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_export_sheet_as_pdf.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_sheet_as_pdf);
}

pub fn dialog_export_sheet_as_xopp(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/x-xopp");
    filter.add_pattern("*.xopp");
    filter.set_name(Some(&gettext(".xopp file")));

    let dialog_export_sheet_as_xopp: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Sheet"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_sheet_as_xopp.add_filter(&filter);

    dialog_export_sheet_as_xopp.set_current_name(
        format!("{}_sheet.xopp", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_sheet_as_xopp.connect_response(
        clone!(@weak appwindow => move |dialog_export_sheet, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_sheet.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                            appwindow.canvas_progressbar().pulse();

                            if let Err(e) = appwindow.export_sheet_as_xopp(&file).await {
                                log::error!("exporting sheet as .xopp failed, replace_file_async failed with Err {}", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Exporting sheet as .xopp failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported sheet as .xopp successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }),
    );

    dialog_export_sheet_as_xopp.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_sheet_as_xopp);
}

pub fn dialog_export_engine_state(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    filter.set_name(Some(&gettext("JSON file")));

    let dialog_export_engine_state: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export engine state"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_engine_state.add_filter(&filter);

    dialog_export_engine_state.set_current_name(
        format!(
            "{}_engine_state.json",
            rnote_engine::utils::now_formatted_string()
        )
        .as_str(),
    );

    dialog_export_engine_state.connect_response(
        clone!(@weak appwindow => move |dialog_export_engine_state, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_engine_state.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                            appwindow.canvas_progressbar().pulse();

                            if let Err(e) = appwindow.export_engine_state(&file).await {
                                log::error!("exporting engine state failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export engine state failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported engine state successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }),
    );

    dialog_export_engine_state.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_engine_state);
}
