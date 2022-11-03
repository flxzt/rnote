use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, Dialog, FileChooserAction, FileChooserNative,
    FileFilter, Label, ResponseType, SpinButton, Switch,
};
use num_traits::ToPrimitive;
use rnote_engine::engine::export::{
    DocExportFormat, DocExportPrefs, SelectionExportFormat, SelectionExportPrefs,
};

use crate::{appwindow, config, RnoteAppWindow};

pub fn filechooser_save_doc_as(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_pattern("*.rnote");
    filter.set_name(Some(&gettext(".rnote file")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Save document as"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Save"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    filechooser.add_filter(&filter);

    // Set the output file as default, else at least the current workspace directory
    if let Some(output_file) = appwindow.canvas().output_file() {
        if let Err(e) = filechooser.set_file(&output_file) {
            log::error!("set_file() for dialog_save_doc_as failed with Err `{e}`");
        }
    } else {
        if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
            if let Err(e) =
                filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
            {
                log::error!("set_current_folder() for dialog_save_doc_as failed with Err `{e}`");
            }
        }

        let file_title = rnote_engine::utils::default_file_title_for_export(
            appwindow.canvas().output_file(),
            Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
            None,
        );

        filechooser.set_current_name(&(file_title + "." + "rnote"));
    }

    filechooser.connect_response(
        clone!(@weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.save_document_to_file(&file).await {
                                appwindow.canvas().set_output_file(None);

                                log::error!("saving document failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving document failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Saved document successfully.").to_variant()));
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

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

pub fn dialog_export_doc_w_prefs(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog: Dialog = builder.object("dialog_export_doc_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_doc_button_confirm").unwrap();
    let with_background_switch: Switch =
        builder.object("export_doc_with_background_switch").unwrap();
    let export_format_row: adw::ComboRow = builder.object("export_doc_export_format_row").unwrap();
    let export_file_label: Label = builder.object("export_doc_export_file_label").unwrap();
    let export_file_button: Button = builder.object("export_doc_export_file_button").unwrap();

    let doc_export_prefs = appwindow
        .canvas()
        .engine()
        .borrow_mut()
        .export_prefs
        .doc_export_prefs;

    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let filechooser = create_filechooser_export_doc(appwindow);
    with_background_switch.set_active(doc_export_prefs.with_background);
    export_format_row.set_selected(doc_export_prefs.export_format.to_u32().unwrap());

    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
        let path_string = p.to_string_lossy().to_string();
        export_file_label.set_label(&path_string);
        button_confirm.set_sensitive(true);
    } else {
        export_file_label.set_label(&gettext("- no file selected -"));
        button_confirm.set_sensitive(false);
    }

    // Update prefs
    export_file_button.connect_clicked(
        clone!(@weak dialog, @weak filechooser, @weak appwindow => move |_| {
            dialog.hide();
            filechooser.show();
        }),
    );

    filechooser.connect_response(
        clone!(@weak button_confirm, @weak export_file_label, @weak dialog, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
                        let path_string = p.to_string_lossy().to_string();
                        export_file_label.set_label(&path_string);
                        button_confirm.set_sensitive(true);
                    } else {
                        export_file_label.set_label(&gettext("- no file selected -"));
                        button_confirm.set_sensitive(false);
                    }
                }
                _ => {}
            }

            filechooser.hide();
            dialog.show();
        }),
    );

    with_background_switch.connect_active_notify(clone!(@weak appwindow => move |with_background_switch| {
        appwindow.canvas().engine().borrow_mut().export_prefs.doc_export_prefs.with_background = with_background_switch.is_active();
    }));

    export_format_row.connect_selected_notify(clone!(@weak export_file_label, @weak filechooser, @weak appwindow => move |row| {
        let selected = row.selected();
        let export_format = DocExportFormat::try_from(selected).unwrap();
        appwindow.canvas().engine().borrow_mut().export_prefs.doc_export_prefs.export_format = export_format;

        // update the filechooser dependent on the selected export format
        update_export_doc_filechooser_with_prefs(&filechooser, appwindow.canvas().output_file(), &appwindow.canvas().engine().borrow().export_prefs.doc_export_prefs);

        // force the user to pick another file
        export_file_label.set_label(&gettext("- no file selected -"));
        button_confirm.set_sensitive(false);
    }));

    dialog.connect_response(
        clone!(@weak with_background_switch, @strong filechooser, @weak appwindow => move |dialog, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            let file_title = file.basename().and_then(|b| Some(b.file_stem()?.to_string_lossy().to_string())).unwrap_or(appwindow::OUTPUT_FILE_NEW_TITLE.clone());

                            if let Err(e) = appwindow.export_doc(&file, file_title, None).await {
                                log::error!("exporting document failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export document failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported document successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    } else {
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export document failed, no file selected.").to_variant()));
                    }
                }
                _ => {}
            }

            dialog.close();
        }));

    dialog.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

fn create_filechooser_export_doc(appwindow: &RnoteAppWindow) -> FileChooserNative {
    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Document"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_export_doc failed with Err `{e}`");
        }
    }

    update_export_doc_filechooser_with_prefs(
        &filechooser,
        appwindow.canvas().output_file(),
        &appwindow
            .canvas()
            .engine()
            .borrow()
            .export_prefs
            .doc_export_prefs,
    );

    filechooser
}

fn update_export_doc_filechooser_with_prefs(
    filechooser: &FileChooserNative,
    output_file: Option<gio::File>,
    doc_export_prefs: &DocExportPrefs,
) {
    let filter = FileFilter::new();

    match doc_export_prefs.export_format {
        DocExportFormat::Svg => {
            filter.add_mime_type("image/svg+xml");
            filter.add_pattern("*.svg");
            filter.set_name(Some(&gettext("Svg")));
        }
        DocExportFormat::Pdf => {
            filter.add_mime_type("application/pdf");
            filter.add_pattern("*.pdf");
            filter.set_name(Some(&gettext("Pdf")));
        }
        DocExportFormat::Xopp => {
            filter.add_mime_type("application/x-xopp");
            filter.add_pattern("*.xopp");
            filter.set_name(Some(&gettext("Xopp")));
        }
    }

    filechooser.add_filter(&filter);

    let file_ext = doc_export_prefs.export_format.file_ext();

    let file_title = rnote_engine::utils::default_file_title_for_export(
        output_file,
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        None,
    );

    filechooser.set_current_name(&(file_title + "." + &file_ext));
}

pub fn dialog_export_selection_w_prefs(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog: Dialog = builder.object("dialog_export_selection_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_selection_button_confirm").unwrap();
    let with_background_switch: Switch = builder
        .object("export_selection_with_background_switch")
        .unwrap();
    let export_format_row: adw::ComboRow = builder
        .object("export_selection_export_format_row")
        .unwrap();
    let export_file_label: Label = builder
        .object("export_selection_export_file_label")
        .unwrap();
    let export_file_button: Button = builder
        .object("export_selection_export_file_button")
        .unwrap();
    let jpeg_quality_row: adw::ActionRow =
        builder.object("export_selection_jpeg_quality_row").unwrap();
    let jpeg_quality_spinbutton: SpinButton = builder
        .object("export_selection_jpeg_quality_spinbutton")
        .unwrap();

    let selection_export_prefs = appwindow
        .canvas()
        .engine()
        .borrow_mut()
        .export_prefs
        .selection_export_prefs;

    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let filechooser = create_filechooser_export_selection(appwindow);
    with_background_switch.set_active(selection_export_prefs.with_background);
    export_format_row.set_selected(selection_export_prefs.export_format.to_u32().unwrap());
    jpeg_quality_spinbutton.set_value(selection_export_prefs.jpeg_quality as f64);

    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
        let path_string = p.to_string_lossy().to_string();
        export_file_label.set_label(&path_string);
        button_confirm.set_sensitive(true);
    } else {
        export_file_label.set_label(&gettext("- no file selected -"));
        button_confirm.set_sensitive(false);
    }

    // Update prefs
    export_file_button.connect_clicked(
        clone!(@weak dialog, @weak filechooser, @weak appwindow => move |_| {
            dialog.hide();
            filechooser.show();
        }),
    );

    filechooser.connect_response(
        clone!(@weak button_confirm, @weak export_file_label, @weak dialog, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
                        let path_string = p.to_string_lossy().to_string();
                        export_file_label.set_label(&path_string);
                        button_confirm.set_sensitive(true);
                    } else {
                        export_file_label.set_label(&gettext("- no file selected -"));
                        button_confirm.set_sensitive(false);
                    }
                }
                _ => {}
            }

            filechooser.hide();
            dialog.show();
        }),
    );

    with_background_switch.connect_active_notify(clone!(@weak appwindow => move |with_background_switch| {
        appwindow.canvas().engine().borrow_mut().export_prefs.selection_export_prefs.with_background = with_background_switch.is_active();
    }));

    export_format_row.connect_selected_notify(clone!(@weak jpeg_quality_row, @weak export_file_label, @weak filechooser, @weak appwindow => move |row| {
        let selected = row.selected();
        let export_format = SelectionExportFormat::try_from(selected).unwrap();
        appwindow.canvas().engine().borrow_mut().export_prefs.selection_export_prefs.export_format = export_format;

        // update the filechooser dependent on the selected export format
        update_export_selection_filechooser_with_prefs(&filechooser, appwindow.canvas().output_file(),&appwindow.canvas().engine().borrow().export_prefs.selection_export_prefs);

        // force the user to pick another file
        export_file_label.set_label(&gettext("- no file selected -"));
        button_confirm.set_sensitive(false);

        // Set the jpeg quality pref only sensitive when jpeg is actually selected
        jpeg_quality_row.set_sensitive(export_format == SelectionExportFormat::Jpeg);
    }));

    jpeg_quality_spinbutton.connect_value_changed(clone!(@weak appwindow => move |jpeg_quality_spinbutton| {
        appwindow.canvas().engine().borrow_mut().export_prefs.selection_export_prefs.jpeg_quality = jpeg_quality_spinbutton.value().clamp(1.0, 100.0) as u8;
    }));

    dialog.connect_response(
        clone!(@weak with_background_switch, @strong filechooser, @weak appwindow => move |dialog, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_selection(&file, None).await {
                                log::error!("exporting selection failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export selection failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported selection successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    } else {
                        adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export selection failed, no file selected.").to_variant()));
                    }
                }
                _ => {}
            }

            dialog.close();
        }));

    dialog.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

fn create_filechooser_export_selection(appwindow: &RnoteAppWindow) -> FileChooserNative {
    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Selection"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_export_selection failed with Err `{e}`");
        }
    }

    update_export_selection_filechooser_with_prefs(
        &filechooser,
        appwindow.canvas().output_file(),
        &appwindow
            .canvas()
            .engine()
            .borrow()
            .export_prefs
            .selection_export_prefs,
    );

    filechooser
}

fn update_export_selection_filechooser_with_prefs(
    filechooser: &FileChooserNative,
    output_file: Option<gio::File>,
    selection_export_prefs: &SelectionExportPrefs,
) {
    let filter = FileFilter::new();

    match selection_export_prefs.export_format {
        SelectionExportFormat::Svg => {
            filter.add_mime_type("image/svg+xml");
            filter.add_pattern("*.svg");
            filter.set_name(Some(&gettext("Svg")));
        }
        SelectionExportFormat::Png => {
            filter.add_mime_type("image/png");
            filter.add_pattern("*.png");
            filter.set_name(Some(&gettext("Png")));
        }
        SelectionExportFormat::Jpeg => {
            filter.add_mime_type("image/jpeg");
            filter.add_pattern("*.jpg");
            filter.add_pattern("*.jpeg");
            filter.set_name(Some(&gettext("Jpeg")));
        }
    }

    filechooser.add_filter(&filter);

    let file_ext = selection_export_prefs.export_format.file_ext();

    let file_title = rnote_engine::utils::default_file_title_for_export(
        output_file,
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        Some(" - Selection"),
    );

    filechooser.set_current_name(&(file_title + "." + &file_ext));
}

pub fn filechooser_export_engine_state(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    filter.set_name(Some(&gettext("JSON file")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export engine state"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    filechooser.add_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!(
                "set_current_folder() for dialog_export_engine_state failed with Err `{e}`"
            );
        }
    }

    let file_title = rnote_engine::utils::default_file_title_for_export(
        appwindow.canvas().output_file(),
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine state"),
    );

    filechooser.set_current_name(&(file_title + "." + "json"));

    filechooser.connect_response(
        clone!(@weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

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

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

pub fn filechooser_export_engine_config(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    filter.set_name(Some(&gettext("JSON file")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export engine config"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    filechooser.add_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!(
                "set_current_folder() for dialog_export_engine_config failed with Err `{e}`"
            );
        }
    }

    let file_title = rnote_engine::utils::default_file_title_for_export(
        appwindow.canvas().output_file(),
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine config"),
    );

    filechooser.set_current_name(&(file_title + "." + "json"));

    filechooser.connect_response(
        clone!(@weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_engine_config(&file).await {
                                log::error!("exporting engine state failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export engine config failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported engine config successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }),
    );

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}
