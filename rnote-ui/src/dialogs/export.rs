use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, Dialog, FileChooserAction, FileChooserNative,
    FileFilter, Label, ResponseType, SpinButton, Switch,
};
use num_traits::ToPrimitive;
use rnote_engine::engine::export::{
    DocExportFormat, DocExportPrefs, DocPagesExportFormat, DocPagesExportPrefs,
    SelectionExportFormat, SelectionExportPrefs,
};

use crate::canvas::{self, RnCanvas};
use crate::{config, RnAppWindow};

pub(crate) fn filechooser_save_doc_as(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_suffix("rnote");
    filter.set_name(Some(&gettext(".rnote")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Save Document As"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Save"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    filechooser.set_filter(&filter);

    // Set the output file as default, else at least the current workspace directory
    if let Some(output_file) = canvas.output_file() {
        if let Err(e) = filechooser.set_file(&output_file) {
            log::error!("set_file() for dialog_save_doc_as failed with Err: {e:?}");
        }
    } else {
        if let Some(current_workspace_dir) = appwindow.workspacebrowser().dirlist_dir() {
            if let Err(e) =
                filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
            {
                log::error!("set_current_folder() for dialog_save_doc_as failed with Err: {e:?}");
            }
        }

        let file_name = canvas.doc_title_display() + ".rnote";
        filechooser.set_current_name(&(file_name));
    }

    filechooser.connect_response(
        clone!(@weak canvas, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            appwindow.overlays().start_pulsing_progressbar();

                            match canvas.save_document_to_file(&file).await {
                                Ok(true) => {
                                    appwindow.overlays().dispatch_toast_text(&gettext("Saved document successfully"));
                                }
                                Ok(false) => {
                                    // Saving was already in progress
                                }
                                Err(e) => {
                                    canvas.set_output_file(None);

                                    log::error!("saving document failed, Error: `{e:?}`");
                                    appwindow.overlays().dispatch_toast_error(&gettext("Saving document failed"));
                                }
                            }

                            appwindow.overlays().finish_progressbar();
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

pub(crate) fn dialog_export_doc_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/export.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_export_doc_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_doc_button_confirm").unwrap();
    let with_background_switch: Switch =
        builder.object("export_doc_with_background_switch").unwrap();
    let with_pattern_row: adw::ActionRow = builder.object("export_doc_with_pattern_row").unwrap();
    let with_pattern_switch: Switch = builder.object("export_doc_with_pattern_switch").unwrap();
    let export_format_row: adw::ComboRow = builder.object("export_doc_export_format_row").unwrap();
    let export_file_label: Label = builder.object("export_doc_export_file_label").unwrap();
    let export_file_button: Button = builder.object("export_doc_export_file_button").unwrap();

    let doc_export_prefs = canvas.engine().borrow_mut().export_prefs.doc_export_prefs;

    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let filechooser =
        create_filechooser_export_doc(appwindow, canvas.output_file(), &doc_export_prefs);
    with_background_switch.set_active(doc_export_prefs.with_background);
    with_pattern_switch.set_active(doc_export_prefs.with_pattern);
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
    with_background_switch
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    export_file_button.connect_clicked(
        clone!(@weak dialog, @strong filechooser, @weak appwindow => move |_| {
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

    with_background_switch.connect_active_notify(clone!(@weak canvas, @weak appwindow => move |with_background_switch| {
        canvas.engine().borrow_mut().export_prefs.doc_export_prefs.with_background = with_background_switch.is_active();
    }));

    with_pattern_switch.connect_active_notify(clone!(@weak canvas, @weak appwindow => move |with_pattern_switch| {
        canvas.engine().borrow_mut().export_prefs.doc_export_prefs.with_pattern = with_pattern_switch.is_active();
    }));

    export_format_row.connect_selected_notify(clone!(@weak export_file_label, @weak button_confirm, @strong filechooser, @weak canvas, @weak appwindow => move |row| {
        let selected = row.selected();
        let export_format = DocExportFormat::try_from(selected).unwrap();
        canvas.engine().borrow_mut().export_prefs.doc_export_prefs.export_format = export_format;

        // update the filechooser dependent on the selected export format
        update_export_doc_filechooser_with_prefs(&filechooser, canvas.output_file(), &canvas.engine().borrow().export_prefs.doc_export_prefs);

        // force the user to pick another file
        export_file_label.set_label(&gettext("- no file selected -"));
        button_confirm.set_sensitive(false);
    }));

    dialog.connect_response(
        clone!(@weak with_background_switch, @strong filechooser, @weak canvas, @weak appwindow => move |dialog, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            appwindow.overlays().start_pulsing_progressbar();

                            let file_title = rnote_engine::utils::default_file_title_for_export(
                                Some(file.clone()),
                                Some(&canvas::OUTPUT_FILE_NEW_TITLE),
                                None,
                            );

                            if let Err(e) = canvas.export_doc(&file, file_title, None).await {
                                log::error!("exporting document failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting document failed"));
                            } else {
                                appwindow.overlays().dispatch_toast_text(&gettext("Exported document successfully"));
                            }

                            appwindow.overlays().finish_progressbar();
                        }));
                    } else {
                        appwindow.overlays().dispatch_toast_error(&gettext("Exporting document failed, no file selected"));
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

fn create_filechooser_export_doc(
    appwindow: &RnAppWindow,
    output_file: Option<gio::File>,
    doc_export_prefs: &DocExportPrefs,
) -> FileChooserNative {
    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Document"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().dirlist_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_export_doc failed with Err: {e:?}");
        }
    }

    update_export_doc_filechooser_with_prefs(&filechooser, output_file, doc_export_prefs);

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
            filter.add_suffix("svg");
            filter.set_name(Some(&gettext("Svg")));
        }
        DocExportFormat::Pdf => {
            filter.add_mime_type("application/pdf");
            filter.add_suffix("pdf");
            filter.set_name(Some(&gettext("Pdf")));
        }
        DocExportFormat::Xopp => {
            filter.add_mime_type("application/x-xopp");
            filter.add_suffix("xopp");
            filter.set_name(Some(&gettext("Xopp")));
        }
    }

    filechooser.set_filter(&filter);

    let file_ext = doc_export_prefs.export_format.file_ext();
    let file_name = rnote_engine::utils::default_file_title_for_export(
        output_file,
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(&(String::from(".") + &file_ext)),
    );
    filechooser.set_current_name(&file_name);
}

pub(crate) fn dialog_export_doc_pages_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/export.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_export_doc_pages_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_doc_pages_button_confirm").unwrap();
    let with_background_switch: Switch = builder
        .object("export_doc_pages_with_background_switch")
        .unwrap();
    let with_pattern_row: adw::ActionRow =
        builder.object("export_doc_pages_with_pattern_row").unwrap();
    let with_pattern_switch: Switch = builder
        .object("export_doc_pages_with_pattern_switch")
        .unwrap();
    let export_format_row: adw::ComboRow = builder
        .object("export_doc_pages_export_format_row")
        .unwrap();
    let bitmap_scalefactor_row: adw::ActionRow = builder
        .object("export_doc_pages_bitmap_scalefactor_row")
        .unwrap();
    let bitmap_scalefactor_spinbutton: SpinButton = builder
        .object("export_doc_pages_bitmap_scalefactor_spinbutton")
        .unwrap();
    let jpeg_quality_row: adw::ActionRow =
        builder.object("export_doc_pages_jpeg_quality_row").unwrap();
    let jpeg_quality_spinbutton: SpinButton = builder
        .object("export_doc_pages_jpeg_quality_spinbutton")
        .unwrap();
    let export_dir_label: Label = builder.object("export_doc_pages_export_dir_label").unwrap();
    let export_dir_button: Button = builder
        .object("export_doc_pages_export_dir_button")
        .unwrap();
    let export_files_stemname_entryrow: adw::EntryRow = builder
        .object("export_doc_pages_export_files_stemname_entryrow")
        .unwrap();
    let page_files_naming_info_label: Label = builder
        .object("export_doc_pages_page_files_naming_info_label")
        .unwrap();

    let doc_pages_export_prefs = canvas
        .engine()
        .borrow_mut()
        .export_prefs
        .doc_pages_export_prefs;

    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let filechooser = create_filechooser_export_doc_pages(
        appwindow,
        canvas.output_file(),
        &doc_pages_export_prefs,
    );
    with_background_switch.set_active(doc_pages_export_prefs.with_background);
    with_pattern_switch.set_active(doc_pages_export_prefs.with_pattern);
    export_format_row.set_selected(doc_pages_export_prefs.export_format.to_u32().unwrap());
    bitmap_scalefactor_row.set_sensitive(
        doc_pages_export_prefs.export_format == DocPagesExportFormat::Png
            || doc_pages_export_prefs.export_format == DocPagesExportFormat::Jpeg,
    );
    bitmap_scalefactor_spinbutton.set_value(doc_pages_export_prefs.bitmap_scalefactor);
    jpeg_quality_row
        .set_sensitive(doc_pages_export_prefs.export_format == DocPagesExportFormat::Jpeg);
    jpeg_quality_spinbutton.set_value(doc_pages_export_prefs.jpeg_quality as f64);

    export_dir_label.set_label(&gettext("- no directory selected -"));
    button_confirm.set_sensitive(false);

    let default_stem_name = rnote_engine::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        None,
    );
    export_files_stemname_entryrow.set_text(&default_stem_name);

    page_files_naming_info_label.set_text(
        &(rnote_engine::utils::doc_pages_files_names(default_stem_name, 1)
            + "."
            + &doc_pages_export_prefs.export_format.file_ext()),
    );

    // Update prefs
    with_background_switch
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    export_dir_button.connect_clicked(
        clone!(@weak dialog, @strong filechooser, @weak appwindow => move |_| {
            dialog.hide();
            filechooser.show();
        }),
    );

    filechooser.connect_response(
        clone!(@weak button_confirm, @weak export_dir_label, @weak dialog, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
                        let path_string = p.to_string_lossy().to_string();
                        export_dir_label.set_label(&path_string);
                        button_confirm.set_sensitive(true);
                    } else {
                        export_dir_label.set_label(&gettext("- no directory selected -"));
                        button_confirm.set_sensitive(false);
                    }
                }
                _ => {}
            }

            filechooser.hide();
            dialog.show();
        }),
    );

    with_background_switch.connect_active_notify(clone!(@weak canvas, @weak appwindow => move |with_background_switch| {
        canvas.engine().borrow_mut().export_prefs.doc_pages_export_prefs.with_background = with_background_switch.is_active();
    }));

    with_pattern_switch.connect_active_notify(clone!(@weak canvas, @weak appwindow => move |with_pattern_switch| {
        canvas.engine().borrow_mut().export_prefs.doc_pages_export_prefs.with_pattern = with_pattern_switch.is_active();
    }));

    export_format_row.connect_selected_notify(clone!(
        @weak page_files_naming_info_label,
        @weak export_files_stemname_entryrow,
        @weak bitmap_scalefactor_row,
        @weak jpeg_quality_row,
        @weak export_dir_label,
        @strong filechooser,
        @weak button_confirm,
        @weak canvas,
        @weak appwindow => move |row| {
            let selected = row.selected();
            let export_format = DocPagesExportFormat::try_from(selected).unwrap();
            canvas.engine().borrow_mut().export_prefs.doc_pages_export_prefs.export_format = export_format;

            // update the filechooser dependent on the selected export format
            update_export_doc_pages_filechooser_with_prefs(&filechooser, canvas.output_file(), &canvas.engine().borrow().export_prefs.doc_pages_export_prefs);

            // Set the bitmap scalefactor sensitive only when exporting to a bitmap image
            bitmap_scalefactor_row.set_sensitive(export_format == DocPagesExportFormat::Png || export_format == DocPagesExportFormat::Jpeg);

            // Set the jpeg quality pref only sensitive when jpeg is actually selected
            jpeg_quality_row.set_sensitive(export_format == DocPagesExportFormat::Jpeg);

            // update file naming preview
            page_files_naming_info_label.set_text(&(
                rnote_engine::utils::doc_pages_files_names(export_files_stemname_entryrow.text().to_string(), 1)
                    + "."
                    + &canvas.engine().borrow_mut().export_prefs.doc_pages_export_prefs.export_format.file_ext()
            ));
    }));

    bitmap_scalefactor_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |bitmap_scalefactor_spinbutton| {
        canvas.engine().borrow_mut().export_prefs.doc_pages_export_prefs.bitmap_scalefactor = bitmap_scalefactor_spinbutton.value();
    }));

    jpeg_quality_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |jpeg_quality_spinbutton| {
        canvas.engine().borrow_mut().export_prefs.doc_pages_export_prefs.jpeg_quality = jpeg_quality_spinbutton.value().clamp(1.0, 100.0) as u8;
    }));

    export_files_stemname_entryrow.connect_changed(
        clone!(@weak page_files_naming_info_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => move |entryrow| {
            button_confirm.set_sensitive(!entryrow.text().is_empty());

            // update file naming preview
            page_files_naming_info_label.set_text(&(
                rnote_engine::utils::doc_pages_files_names(entryrow.text().to_string(), 1)
                    + "."
                    + &canvas.engine().borrow_mut().export_prefs.doc_pages_export_prefs.export_format.file_ext()
            ));
        }),
    );

    dialog.connect_response(
        clone!(@weak with_background_switch, @weak export_files_stemname_entryrow, @strong filechooser, @weak canvas, @weak appwindow => move |dialog, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    if let Some(dir) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            appwindow.overlays().start_pulsing_progressbar();

                            let file_stem_name = export_files_stemname_entryrow.text().to_string();

                            if let Err(e) = canvas.export_doc_pages(&dir, file_stem_name, None).await {
                                log::error!("exporting document pages failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting document pages failed"));
                            } else {
                                appwindow.overlays().dispatch_toast_text(&gettext("Exported document pages successfully"));
                            }

                            appwindow.overlays().finish_progressbar();
                        }));
                    } else {
                        appwindow.overlays().dispatch_toast_error(&gettext("Exporting document pages failed, no directory selected"));
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

fn create_filechooser_export_doc_pages(
    appwindow: &RnAppWindow,
    output_file: Option<gio::File>,
    doc_pages_export_prefs: &DocPagesExportPrefs,
) -> FileChooserNative {
    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Document Pages"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::SelectFolder)
        .select_multiple(false)
        .build();

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().dirlist_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_export_doc_pages failed with Err: {e:?}");
        }
    }

    update_export_doc_pages_filechooser_with_prefs(
        &filechooser,
        output_file,
        doc_pages_export_prefs,
    );

    filechooser
}

fn update_export_doc_pages_filechooser_with_prefs(
    filechooser: &FileChooserNative,
    _output_file: Option<gio::File>,
    doc_pages_export_prefs: &DocPagesExportPrefs,
) {
    let filter = FileFilter::new();
    // We always need to be able to select folders
    filter.add_mime_type("inode/directory");

    match doc_pages_export_prefs.export_format {
        DocPagesExportFormat::Svg => {
            filter.add_mime_type("image/svg+xml");
            filter.add_suffix("svg");
            filter.set_name(Some(&gettext("Svg")));
        }
        DocPagesExportFormat::Png => {
            filter.add_mime_type("image/png");
            filter.add_suffix("png");
            filter.set_name(Some(&gettext("Png")));
        }
        DocPagesExportFormat::Jpeg => {
            filter.add_mime_type("image/jpeg");
            filter.add_suffix("jpg");
            filter.add_suffix("jpeg");
            filter.set_name(Some(&gettext("Jpeg")));
        }
    }

    filechooser.set_filter(&filter);
}

pub(crate) fn dialog_export_selection_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/export.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_export_selection_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_selection_button_confirm").unwrap();
    let with_background_switch: Switch = builder
        .object("export_selection_with_background_switch")
        .unwrap();
    let with_pattern_row: adw::ActionRow =
        builder.object("export_selection_with_pattern_row").unwrap();
    let with_pattern_switch: Switch = builder
        .object("export_selection_with_pattern_switch")
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
    let bitmap_scalefactor_row: adw::ActionRow = builder
        .object("export_selection_bitmap_scalefactor_row")
        .unwrap();
    let bitmap_scalefactor_spinbutton: SpinButton = builder
        .object("export_selection_bitmap_scalefactor_spinbutton")
        .unwrap();
    let jpeg_quality_row: adw::ActionRow =
        builder.object("export_selection_jpeg_quality_row").unwrap();
    let jpeg_quality_spinbutton: SpinButton = builder
        .object("export_selection_jpeg_quality_spinbutton")
        .unwrap();
    let margin_spinbutton: SpinButton = builder
        .object("export_selection_margin_spinbutton")
        .unwrap();

    let selection_export_prefs = canvas
        .engine()
        .borrow_mut()
        .export_prefs
        .selection_export_prefs;

    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let filechooser = create_filechooser_export_selection(
        appwindow,
        canvas.output_file(),
        &selection_export_prefs,
    );
    with_background_switch.set_active(selection_export_prefs.with_background);
    with_pattern_switch.set_active(selection_export_prefs.with_pattern);
    export_format_row.set_selected(selection_export_prefs.export_format.to_u32().unwrap());
    bitmap_scalefactor_row.set_sensitive(
        selection_export_prefs.export_format == SelectionExportFormat::Png
            || selection_export_prefs.export_format == SelectionExportFormat::Jpeg,
    );
    bitmap_scalefactor_spinbutton.set_value(selection_export_prefs.bitmap_scalefactor);
    jpeg_quality_row
        .set_sensitive(selection_export_prefs.export_format == SelectionExportFormat::Jpeg);
    jpeg_quality_spinbutton.set_value(selection_export_prefs.jpeg_quality as f64);
    margin_spinbutton.set_value(selection_export_prefs.margin);

    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
        let path_string = p.to_string_lossy().to_string();
        export_file_label.set_label(&path_string);
        button_confirm.set_sensitive(true);
    } else {
        export_file_label.set_label(&gettext("- no file selected -"));
        button_confirm.set_sensitive(false);
    }

    // Update prefs
    with_background_switch
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    export_file_button.connect_clicked(
        clone!(@weak dialog, @strong filechooser, @weak appwindow => move |_| {
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

    with_background_switch.connect_active_notify(clone!(@weak canvas, @weak appwindow => move |with_background_switch| {
        canvas.engine().borrow_mut().export_prefs.selection_export_prefs.with_background = with_background_switch.is_active();
    }));

    with_pattern_switch.connect_active_notify(clone!(@weak canvas, @weak appwindow => move |with_pattern_switch| {
        canvas.engine().borrow_mut().export_prefs.selection_export_prefs.with_pattern = with_pattern_switch.is_active();
    }));

    export_format_row.connect_selected_notify(clone!(
        @weak bitmap_scalefactor_row,
        @weak jpeg_quality_row,
        @weak export_file_label,
        @strong filechooser,
        @weak canvas,
        @weak appwindow => move |row| {
            let selected = row.selected();
            let export_format = SelectionExportFormat::try_from(selected).unwrap();
            canvas.engine().borrow_mut().export_prefs.selection_export_prefs.export_format = export_format;

            // update the filechooser dependent on the selected export format
            update_export_selection_filechooser_with_prefs(&filechooser, canvas.output_file(),&canvas.engine().borrow().export_prefs.selection_export_prefs);

            // force the user to pick another file
            export_file_label.set_label(&gettext("- no file selected -"));
            button_confirm.set_sensitive(false);

            // Set the bitmap scalefactor sensitive only when exporting to a bitmap image
            bitmap_scalefactor_row.set_sensitive(export_format == SelectionExportFormat::Png || export_format == SelectionExportFormat::Jpeg);

            // Set the jpeg quality pref only sensitive when jpeg is actually selected
            jpeg_quality_row.set_sensitive(export_format == SelectionExportFormat::Jpeg);
    }));

    bitmap_scalefactor_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |bitmap_scalefactor_spinbutton| {
        canvas.engine().borrow_mut().export_prefs.selection_export_prefs.bitmap_scalefactor = bitmap_scalefactor_spinbutton.value();
    }));

    jpeg_quality_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |jpeg_quality_spinbutton| {
        canvas.engine().borrow_mut().export_prefs.selection_export_prefs.jpeg_quality = jpeg_quality_spinbutton.value().clamp(1.0, 100.0) as u8;
    }));

    margin_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |margin_spinbutton| {
        canvas.engine().borrow_mut().export_prefs.selection_export_prefs.margin = margin_spinbutton.value();
    }));

    dialog.connect_response(
        clone!(@weak with_background_switch, @strong filechooser, @weak canvas, @weak appwindow => move |dialog, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak appwindow => async move {
                            appwindow.overlays().start_pulsing_progressbar();

                            if let Err(e) = canvas.export_selection(&file, None).await {
                                log::error!("exporting selection failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting selection failed"));
                            } else {
                                appwindow.overlays().dispatch_toast_text(&gettext("Exported selection successfully"));
                            }

                            appwindow.overlays().finish_progressbar();
                        }));
                    } else {
                        appwindow.overlays().dispatch_toast_error(&gettext("Exporting selection failed, no file selected"));
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

fn create_filechooser_export_selection(
    appwindow: &RnAppWindow,
    output_file: Option<gio::File>,
    selection_export_prefs: &SelectionExportPrefs,
) -> FileChooserNative {
    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Selection"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().dirlist_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_export_selection failed with Err: {e:?}");
        }
    }

    update_export_selection_filechooser_with_prefs(
        &filechooser,
        output_file,
        selection_export_prefs,
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
            filter.add_suffix("svg");
            filter.set_name(Some(&gettext("Svg")));
        }
        SelectionExportFormat::Png => {
            filter.add_mime_type("image/png");
            filter.add_suffix("png");
            filter.set_name(Some(&gettext("Png")));
        }
        SelectionExportFormat::Jpeg => {
            filter.add_mime_type("image/jpeg");
            filter.add_suffix("jpg");
            filter.add_suffix("jpeg");
            filter.set_name(Some(&gettext("Jpeg")));
        }
    }

    filechooser.set_filter(&filter);

    let file_ext = selection_export_prefs.export_format.file_ext();
    let file_name = rnote_engine::utils::default_file_title_for_export(
        output_file,
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(&(String::from(" - Selection") + "." + &file_ext)),
    );
    filechooser.set_current_name(&file_name);
}

pub(crate) fn filechooser_export_engine_state(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_suffix("json");
    filter.set_name(Some(&gettext("Json")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Engine State"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    filechooser.set_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().dirlist_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!(
                "set_current_folder() for dialog_export_engine_state failed with Err: {e:?}"
            );
        }
    }

    let file_name = rnote_engine::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine state.json"),
    );

    filechooser.set_current_name(&file_name);

    filechooser.connect_response(
        clone!(@weak canvas, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            appwindow.overlays().start_pulsing_progressbar();

                            if let Err(e) = canvas.export_engine_state(&file).await {
                                log::error!("exporting engine state failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting engine state failed"));
                            } else {
                                appwindow.overlays().dispatch_toast_text(&gettext("Exported engine state successfully"));
                            }

                            appwindow.overlays().finish_progressbar();
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

pub(crate) fn filechooser_export_engine_config(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_suffix("json");
    filter.set_name(Some(&gettext("Json")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Engine Config"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    filechooser.set_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().dirlist_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!(
                "set_current_folder() for dialog_export_engine_config failed with Err: {e:?}"
            );
        }
    }

    let file_name = rnote_engine::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine config.json"),
    );

    filechooser.set_current_name(&file_name);

    filechooser.connect_response(
        clone!(@weak canvas, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            appwindow.overlays().start_pulsing_progressbar();

                            if let Err(e) = canvas.export_engine_config(&file).await {
                                log::error!("exporting engine state failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting engine config failed"));
                            } else {
                                appwindow.overlays().dispatch_toast_text(&gettext("Exported engine config successfully"));
                            }

                            appwindow.overlays().finish_progressbar();
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
