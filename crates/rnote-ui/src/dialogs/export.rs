//adw::ToolbarView is a replacement for adw::Dialog but not suitable for an async flow

// Imports
use crate::canvas::{self, RnCanvas};
use crate::RnStrokeContentPreview;
use crate::{config, RnAppWindow};
use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, Builder, Button, FileDialog, FileFilter, Label};
use num_traits::ToPrimitive;
use rnote_compose::SplitOrder;
use rnote_engine::document::Layout;
use rnote_engine::engine::export::{
    DocExportFormat, DocExportPrefs, DocPagesExportFormat, DocPagesExportPrefs,
    SelectionExportFormat, SelectionExportPrefs,
};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) async fn dialog_save_doc_as(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    // note : mimetypes are not supported with the native file picker on windows
    // See the limitations on FileChooserNative
    // https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/struct.FileChooserNative.html#win32-details--gtkfilechooserdialognative-win32
    let filter = FileFilter::new();
    if cfg!(target_os = "windows") {
        filter.add_pattern("*.rnote");
    } else {
        filter.add_mime_type("application/rnote");
    }
    if cfg!(target_os = "macos") {
        filter.add_suffix("rnote");
    }
    filter.set_name(Some(&gettext(".rnote")));

    // create the list of filters
    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);

    let filedialog = FileDialog::builder()
        .title(gettext("Save Document As"))
        .modal(true)
        .accept_label(gettext("Save"))
        .filters(&filter_list)
        .default_filter(&filter)
        .build();

    // Set the output file as default, else at least the current workspace directory
    if let Some(output_file) = canvas.output_file() {
        filedialog.set_initial_file(Some(&output_file));
    } else {
        if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dir_list_dir() {
            filedialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
        }

        let file_name = canvas.doc_title_display() + ".rnote";
        filedialog.set_initial_name(Some(&file_name));
    }

    match filedialog.save_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow.overlays().progressbar_start_pulsing();

            match canvas.save_document_to_file(&selected_file).await {
                Ok(true) => {
                    appwindow.overlays().dispatch_toast_text(
                        &gettext("Saved document successfully"),
                        crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                    );
                    appwindow.overlays().progressbar_finish();
                }
                Ok(false) => {
                    // Saving was already in progress
                    appwindow.overlays().progressbar_finish();
                }
                Err(e) => {
                    tracing::error!("Saving document failed, Err: {e:?}");

                    canvas.set_output_file(None);
                    appwindow
                        .overlays()
                        .dispatch_toast_error(&gettext("Saving document failed"));
                    appwindow.overlays().progressbar_abort();
                }
            }
        }
        Err(e) => {
            tracing::debug!(
                "no file selected in save doc as dialog (Error or dialog dismissed by user), Err: {e:?}"
            )
        }
    }
}

pub(crate) async fn dialog_export_doc_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/export.ui").as_str(),
    );
    let dialog: adw::Dialog = builder.object("dialog_export_doc_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_doc_button_confirm").unwrap();
    let with_background_row: adw::SwitchRow =
        builder.object("export_doc_with_background_row").unwrap();
    let with_pattern_row: adw::SwitchRow = builder.object("export_doc_with_pattern_row").unwrap();
    let optimize_printing_row: adw::SwitchRow =
        builder.object("export_doc_optimize_printing_row").unwrap();
    let export_format_row: adw::ComboRow = builder.object("export_doc_export_format_row").unwrap();
    let page_order_row: adw::ComboRow = builder.object("export_doc_page_order_row").unwrap();
    let export_file_label: Label = builder.object("export_doc_export_file_label").unwrap();
    let export_file_button: Button = builder.object("export_doc_export_file_button").unwrap();
    let preview: RnStrokeContentPreview = builder.object("export_doc_preview").unwrap();
    let export_doc_button_cancel: Button = builder.object("export_doc_button_cancel").unwrap();
    let export_doc_button_confirm: Button = builder.object("export_doc_button_confirm").unwrap();

    let initial_doc_export_prefs = canvas.engine_ref().export_prefs.doc_export_prefs;
    let doc_layout = canvas.engine_ref().document.layout;

    // initial widget state with the preferences
    let selected_file: Rc<RefCell<Option<gio::File>>> = Rc::new(RefCell::new(None));
    with_background_row.set_active(initial_doc_export_prefs.with_background);
    with_pattern_row.set_active(initial_doc_export_prefs.with_pattern);
    optimize_printing_row.set_active(initial_doc_export_prefs.optimize_printing);
    preview.set_draw_background(initial_doc_export_prefs.with_background);
    preview.set_draw_pattern(initial_doc_export_prefs.with_pattern);
    preview.set_optimize_printing(initial_doc_export_prefs.optimize_printing);
    preview.set_contents(
        canvas
            .engine_ref()
            .extract_pages_content(initial_doc_export_prefs.page_order),
    );
    export_format_row.set_selected(initial_doc_export_prefs.export_format.to_u32().unwrap());
    page_order_row.set_selected(initial_doc_export_prefs.page_order.to_u32().unwrap());
    export_file_label.set_label(&gettext("- no file selected -"));
    page_order_row
        .set_sensitive(doc_layout == Layout::SemiInfinite || doc_layout == Layout::Infinite);
    button_confirm.set_sensitive(false);

    // Update prefs

    export_file_button.connect_clicked(
        clone!(@strong selected_file, @weak export_file_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => move |_| {
            glib::spawn_future_local(clone!(@strong selected_file, @weak export_file_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => async move {
                dialog.set_sensitive(false);

                let doc_export_prefs = canvas.engine_mut().export_prefs.doc_export_prefs;
                let filedialog =
                    create_filedialog_export_doc(&appwindow, &canvas, &doc_export_prefs);
                match filedialog.save_future(Some(&appwindow)).await {
                    Ok(f) => {
                        if let Some(path_string) = f.path().map(|p| p.to_string_lossy().to_string()) {
                            export_file_label.set_label(&path_string);
                            button_confirm.set_sensitive(true);
                            selected_file.replace(Some(f));
                        } else {
                            export_file_label.set_label(&gettext("- no file selected -"));
                            button_confirm.set_sensitive(false);
                            selected_file.replace(None);
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Did not export document (Error or dialog dismissed by user), Err: {e:?}");
                        export_file_label.set_label(&gettext("- no file selected -"));
                        button_confirm.set_sensitive(false);
                        selected_file.replace(None);
                    }
                }

                dialog.set_sensitive(true);
            }));
        }),
    );

    with_background_row
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    with_background_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |with_background_row| {
            let active = with_background_row.is_active();
            canvas.engine_mut().export_prefs.doc_export_prefs.with_background = active;
            preview.set_draw_background(active);
        }),
    );

    with_pattern_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |with_pattern_row| {
            let active = with_pattern_row.is_active();
            canvas.engine_mut().export_prefs.doc_export_prefs.with_pattern = active;
            preview.set_draw_pattern(active);
        }),
    );

    optimize_printing_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |optimize_printing_row| {
            let active = optimize_printing_row.is_active();
            canvas.engine_mut().export_prefs.doc_export_prefs.optimize_printing = active;
            preview.set_optimize_printing(active);
        }),
    );

    export_format_row.connect_selected_notify(clone!(@strong selected_file, @weak export_file_label, @weak page_order_row, @weak button_confirm, @weak canvas, @weak appwindow => move |row| {
        let export_format = DocExportFormat::try_from(row.selected()).unwrap();
        canvas.engine_mut().export_prefs.doc_export_prefs.export_format = export_format;

        // force the user to pick another file
        export_file_label.set_label(&gettext("- no file selected -"));
        button_confirm.set_sensitive(false);
        selected_file.replace(None);
    }));

    page_order_row.connect_selected_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |row| {
            let page_order = SplitOrder::try_from(row.selected()).unwrap();
            canvas.engine_mut().export_prefs.doc_export_prefs.page_order = page_order;
            preview.set_contents(
                canvas
                    .engine_ref()
                    .extract_pages_content(page_order),
            );
        }),
    );

    // Listen to responses

    export_doc_button_cancel.connect_clicked(clone!(@weak dialog => move |_| {
        dialog.close();
    }));

    export_doc_button_confirm.connect_clicked(clone!(@weak dialog, @weak canvas, @weak appwindow => move |_| {
        dialog.close();

        if let Some(file) = selected_file.take() {
            glib::spawn_future_local(clone!(@weak canvas, @weak appwindow => async move {
                appwindow.overlays().progressbar_start_pulsing();

                let file_title = crate::utils::default_file_title_for_export(
                    Some(file.clone()),
                    Some(&canvas::OUTPUT_FILE_NEW_TITLE),
                    None,
                );
                if let Err(e) = canvas.export_doc(&file, file_title, None).await {
                    tracing::error!("Exporting document failed, Err: `{e:?}`");

                    appwindow.overlays().dispatch_toast_error(&gettext("Exporting document failed"));
                    appwindow.overlays().progressbar_abort();
                } else {
                    appwindow.overlays().dispatch_toast_w_button_singleton(
                        &gettext("Exported document successfully"),
                        &gettext("View in file manager"),
                        clone!(@weak canvas, @weak appwindow => move |_reload_toast| {
                            let Some(folder_path_string) = file
                                .parent()
                                .and_then(|p|
                                    p.path())
                                .and_then(|p| p.into_os_string().into_string().ok()) else {
                                    tracing::error!("Failed to get the parent folder of the output file `{file:?}.");
                                    appwindow.overlays().dispatch_toast_error(&gettext("Exporting document failed"));
                                    return;
                            };

                            if let Err(e) = open::that(&folder_path_string) {
                                tracing::error!("Opening the parent folder '{folder_path_string}' in the file manager failed, Err: {e:?}");
                                appwindow.overlays().dispatch_toast_error(&gettext("Failed to open the file in the file manager"));
                            }
                        }
                    ), crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT, &mut None);
                    appwindow.overlays().progressbar_finish();
                }
            }));
        } else {
            appwindow
                .overlays()
                .dispatch_toast_error(&gettext("Exporting document failed, no file selected"));
        }
    }));

    dialog.present(appwindow);
}

fn create_filedialog_export_doc(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    doc_export_prefs: &DocExportPrefs,
) -> FileDialog {
    let filedialog = FileDialog::builder()
        .title(gettext("Export Document"))
        .modal(true)
        .accept_label(gettext("Select"))
        .build();

    let filter = FileFilter::new();
    // note : mimetypes are not supported with the native file picker on windows
    // See the limitations on FileChooserNative
    // https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/struct.FileChooserNative.html#win32-details--gtkfilechooserdialognative-win32
    match doc_export_prefs.export_format {
        DocExportFormat::Svg => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.svg");
            } else {
                filter.add_mime_type("image/svg+xml");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("svg");
            }
            filter.set_name(Some(&gettext("Svg")));
        }
        DocExportFormat::Pdf => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.pdf");
            } else {
                filter.add_mime_type("application/pdf");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("pdf");
            }
            filter.set_name(Some(&gettext("Pdf")));
        }
        DocExportFormat::Xopp => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.xopp");
            } else {
                filter.add_mime_type("application/x-xopp");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("xopp");
            }
            filter.set_name(Some(&gettext("Xopp")));
        }
    }
    let file_ext = doc_export_prefs.export_format.file_ext();
    let file_name = crate::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(&(String::from(".") + &file_ext)),
    );

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);
    filedialog.set_filters(Some(&filter_list));

    filedialog.set_default_filter(Some(&filter));
    filedialog.set_initial_name(Some(&file_name));
    filedialog.set_initial_folder(get_initial_folder_for_export(appwindow, canvas).as_ref());

    filedialog
}

pub(crate) async fn dialog_export_doc_pages_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/export.ui").as_str(),
    );
    let dialog: adw::Dialog = builder.object("dialog_export_doc_pages_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_doc_pages_button_confirm").unwrap();
    let with_background_row: adw::SwitchRow = builder
        .object("export_doc_pages_with_background_row")
        .unwrap();
    let with_pattern_row: adw::SwitchRow =
        builder.object("export_doc_pages_with_pattern_row").unwrap();
    let optimize_printing_row: adw::SwitchRow = builder
        .object("export_doc_pages_optimize_printing_row")
        .unwrap();
    let export_format_row: adw::ComboRow = builder
        .object("export_doc_pages_export_format_row")
        .unwrap();
    let page_order_row: adw::ComboRow = builder.object("export_doc_pages_page_order_row").unwrap();
    let bitmap_scalefactor_row: adw::SpinRow = builder
        .object("export_doc_pages_bitmap_scalefactor_row")
        .unwrap();
    let jpeg_quality_row: adw::SpinRow =
        builder.object("export_doc_pages_jpeg_quality_row").unwrap();
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
    let preview: RnStrokeContentPreview = builder.object("export_doc_pages_preview").unwrap();
    let export_doc_pages_button_cancel: Button =
        builder.object("export_doc_pages_button_cancel").unwrap();
    let export_doc_pages_button_confirm: Button =
        builder.object("export_doc_pages_button_confirm").unwrap();

    let initial_doc_pages_export_prefs = canvas.engine_ref().export_prefs.doc_pages_export_prefs;
    let doc_layout = canvas.engine_ref().document.layout;

    // initial widget state with the preferences
    let selected_file: Rc<RefCell<Option<gio::File>>> = Rc::new(RefCell::new(None));
    with_background_row.set_active(initial_doc_pages_export_prefs.with_background);
    with_pattern_row.set_active(initial_doc_pages_export_prefs.with_pattern);
    optimize_printing_row.set_active(initial_doc_pages_export_prefs.optimize_printing);
    preview.set_draw_background(initial_doc_pages_export_prefs.with_background);
    preview.set_draw_pattern(initial_doc_pages_export_prefs.with_pattern);
    preview.set_optimize_printing(initial_doc_pages_export_prefs.optimize_printing);
    preview.set_contents(
        canvas
            .engine_ref()
            .extract_pages_content(initial_doc_pages_export_prefs.page_order),
    );
    export_format_row.set_selected(
        initial_doc_pages_export_prefs
            .export_format
            .to_u32()
            .unwrap(),
    );
    page_order_row.set_selected(initial_doc_pages_export_prefs.page_order.to_u32().unwrap());
    bitmap_scalefactor_row.set_sensitive(
        initial_doc_pages_export_prefs.export_format == DocPagesExportFormat::Png
            || initial_doc_pages_export_prefs.export_format == DocPagesExportFormat::Jpeg,
    );
    bitmap_scalefactor_row.set_value(initial_doc_pages_export_prefs.bitmap_scalefactor);
    jpeg_quality_row
        .set_sensitive(initial_doc_pages_export_prefs.export_format == DocPagesExportFormat::Jpeg);
    jpeg_quality_row.set_value(initial_doc_pages_export_prefs.jpeg_quality as f64);
    export_dir_label.set_label(&gettext("- no directory selected -"));
    page_order_row
        .set_sensitive(doc_layout == Layout::SemiInfinite || doc_layout == Layout::Infinite);
    button_confirm.set_sensitive(false);

    let default_stem_name = crate::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        None,
    );
    export_files_stemname_entryrow.set_text(&default_stem_name);
    page_files_naming_info_label.set_text(
        &(rnote_engine::utils::doc_pages_files_names(default_stem_name, 1)
            + "."
            + &initial_doc_pages_export_prefs.export_format.file_ext()),
    );

    // Update prefs

    export_dir_button.connect_clicked(
        clone!(@strong selected_file, @weak export_dir_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => move |_| {
            glib::spawn_future_local(clone!(@strong selected_file, @weak export_dir_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => async move {
                dialog.set_sensitive(false);

                let doc_pages_export_prefs = canvas.engine_mut().export_prefs.doc_pages_export_prefs;
                let filedialog = create_filedialog_export_doc_pages(
                    &appwindow,
                    &canvas,
                    &doc_pages_export_prefs,
                );
                match filedialog.select_folder_future(Some(&appwindow)).await {
                    Ok(f) => {
                        if let Some(path_string) = f.path().map(|p| p.to_string_lossy().to_string()) {
                            export_dir_label.set_label(&path_string);
                            button_confirm.set_sensitive(true);
                            selected_file.replace(Some(f));
                        } else {
                            export_dir_label.set_label(&gettext("- no directory selected -"));
                            button_confirm.set_sensitive(false);
                            selected_file.replace(None);
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Did not export document pages (Error or dialog dismissed by user), Err: {e:?}");

                        export_dir_label.set_label(&gettext("- no directory selected -"));
                        button_confirm.set_sensitive(false);
                        selected_file.replace(None);
                    }
                }

                dialog.set_sensitive(true);
            }));
        }),
    );

    with_background_row
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    with_background_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |with_background_row| {
            let active = with_background_row.is_active();
            canvas.engine_mut().export_prefs.doc_pages_export_prefs.with_background = active;
            preview.set_draw_background(active);
        }),
    );

    with_pattern_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |row| {
            let active = row.is_active();
            canvas.engine_mut().export_prefs.doc_pages_export_prefs.with_pattern = active;
            preview.set_draw_pattern(active);
        }),
    );

    optimize_printing_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |optimize_printing_row| {
            let active = optimize_printing_row.is_active();
            canvas.engine_mut().export_prefs.doc_pages_export_prefs.optimize_printing = active;
            preview.set_optimize_printing(active);
        }),
    );

    export_format_row.connect_selected_notify(clone!(
        @strong selected_file,
        @weak page_files_naming_info_label,
        @weak export_files_stemname_entryrow,
        @weak bitmap_scalefactor_row,
        @weak jpeg_quality_row,
        @weak export_dir_label,
        @weak button_confirm,
        @weak canvas,
        @weak appwindow => move |row| {
            let export_format = DocPagesExportFormat::try_from(row.selected()).unwrap();
            canvas.engine_mut().export_prefs.doc_pages_export_prefs.export_format = export_format;

            // Set the bitmap scalefactor sensitive only when exporting to a bitmap image
            bitmap_scalefactor_row.set_sensitive(export_format == DocPagesExportFormat::Png || export_format == DocPagesExportFormat::Jpeg);
            // Set the jpeg quality pref only sensitive when jpeg is actually selected
            jpeg_quality_row.set_sensitive(export_format == DocPagesExportFormat::Jpeg);
            // update file naming preview
            page_files_naming_info_label.set_text(&(
                rnote_engine::utils::doc_pages_files_names(export_files_stemname_entryrow.text().to_string(), 1)
                    + "."
                    + &canvas.engine_mut().export_prefs.doc_pages_export_prefs.export_format.file_ext()
            ));
    }));

    page_order_row.connect_selected_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |row| {
            let page_order = SplitOrder::try_from(row.selected()).unwrap();
            canvas.engine_mut().export_prefs.doc_pages_export_prefs.page_order = page_order;
            preview.set_contents(
                canvas
                    .engine_ref()
                    .extract_pages_content(page_order),
            );
        }),
    );

    bitmap_scalefactor_row.connect_changed(clone!(@weak canvas, @weak appwindow => move |bitmap_scalefactor_row| {
        canvas.engine_mut().export_prefs.doc_pages_export_prefs.bitmap_scalefactor = bitmap_scalefactor_row.value();
    }));

    jpeg_quality_row.connect_changed(clone!(@weak canvas, @weak appwindow => move |jpeg_quality_row| {
        canvas.engine_mut().export_prefs.doc_pages_export_prefs.jpeg_quality = jpeg_quality_row.value().clamp(1.0, 100.0) as u8;
    }));

    export_files_stemname_entryrow.connect_changed(
        clone!(@weak page_files_naming_info_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => move |entryrow| {
            button_confirm.set_sensitive(!entryrow.text().is_empty());

            // update file naming preview
            page_files_naming_info_label.set_text(&(
                rnote_engine::utils::doc_pages_files_names(entryrow.text().to_string(), 1)
                    + "."
                    + &canvas.engine_mut().export_prefs.doc_pages_export_prefs.export_format.file_ext()
            ));
        }),
    );

    // Listen to responses

    export_doc_pages_button_cancel.connect_clicked(clone!(@weak dialog => move |_| {
        dialog.close();
    }));

    export_doc_pages_button_confirm.connect_clicked(clone!(@weak export_files_stemname_entryrow, @weak dialog, @weak canvas, @weak appwindow => move |_| {
        dialog.close();

        if let Some(dir) = selected_file.take() {
            glib::spawn_future_local(clone!(@weak export_files_stemname_entryrow, @weak canvas, @weak appwindow => async move {
                appwindow.overlays().progressbar_start_pulsing();

                let file_stem_name = export_files_stemname_entryrow.text().to_string();
                if let Err(e) = canvas.export_doc_pages(&dir, file_stem_name, None).await {
                    tracing::error!("Exporting document pages failed, Err: {e:?}");

                    appwindow.overlays().dispatch_toast_error(&gettext("Exporting document pages failed"));
                    appwindow.overlays().progressbar_abort();
                } else {
                    appwindow.overlays().dispatch_toast_w_button_singleton(
                        &gettext("Exported document pages successfully"),
                        &gettext("View in file manager"),
                        clone!(@weak canvas, @weak appwindow => move |_reload_toast| {
                            let Some(folder_path_string) = dir.path().and_then(|p| p.into_os_string().into_string().ok()) else {
                                tracing::error!("Failed to get the path of the parent folder");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting document failed"));
                                return;
                            };

                            if let Err(e) = open::that(&folder_path_string) {
                                tracing::error!("Opening the parent folder '{folder_path_string}' in the file manager failed, Err: {e:?}");
                                appwindow.overlays().dispatch_toast_error(&gettext("Failed to open the file in the file manager"));
                            }
                        }
                    ), crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT, &mut None);
                    appwindow.overlays().progressbar_finish();
                }
            }));
        } else {
            appwindow.overlays().dispatch_toast_error(&gettext(
                "Exporting document pages failed, no directory selected",
            ));
        }
    }));

    dialog.present(appwindow);
}

fn create_filedialog_export_doc_pages(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    doc_pages_export_prefs: &DocPagesExportPrefs,
) -> FileDialog {
    let filedialog = FileDialog::builder()
        .title(gettext("Export Document Pages"))
        .modal(true)
        .accept_label(gettext("Select"))
        .build();

    filedialog.set_initial_folder(get_initial_folder_for_export(appwindow, canvas).as_ref());

    let filter = FileFilter::new();
    // We always need to be able to select folders
    filter.add_mime_type("inode/directory");
    match doc_pages_export_prefs.export_format {
        DocPagesExportFormat::Svg => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.svg");
            } else {
                filter.add_mime_type("image/svg+xml");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("svg");
            }
            filter.set_name(Some(&gettext("Svg")));
        }
        DocPagesExportFormat::Png => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.png");
            } else {
                filter.add_mime_type("image/png");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("png");
            }
            filter.set_name(Some(&gettext("Png")));
        }
        DocPagesExportFormat::Jpeg => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.jpg");
                filter.add_pattern("*.jpeg");
            } else {
                filter.add_mime_type("image/jpeg");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("jpg");
                filter.add_suffix("jpeg");
            }
            filter.set_name(Some(&gettext("Jpeg")));
        }
    }

    filedialog.set_default_filter(Some(&filter));

    filedialog
}

pub(crate) async fn dialog_export_selection_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/export.ui").as_str(),
    );
    let dialog: adw::Dialog = builder.object("dialog_export_selection_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_selection_button_confirm").unwrap();
    let with_background_row: adw::SwitchRow = builder
        .object("export_selection_with_background_row")
        .unwrap();
    let with_pattern_row: adw::SwitchRow =
        builder.object("export_selection_with_pattern_row").unwrap();
    let optimize_printing_row: adw::SwitchRow = builder
        .object("export_selection_optimize_printing_row")
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
    let bitmap_scalefactor_row: adw::SpinRow = builder
        .object("export_selection_bitmap_scalefactor_row")
        .unwrap();
    let jpeg_quality_row: adw::SpinRow =
        builder.object("export_selection_jpeg_quality_row").unwrap();
    let margin_row: adw::SpinRow = builder.object("export_selection_margin_row").unwrap();
    let preview: RnStrokeContentPreview = builder.object("export_selection_preview").unwrap();
    let export_selection_button_cancel: Button =
        builder.object("export_selection_button_cancel").unwrap();
    let export_selection_button_confirm: Button =
        builder.object("export_selection_button_confirm").unwrap();

    let initial_selection_export_prefs = canvas.engine_ref().export_prefs.selection_export_prefs;

    // initial widget state with the preferences
    let selected_file: Rc<RefCell<Option<gio::File>>> = Rc::new(RefCell::new(None));
    with_background_row.set_active(initial_selection_export_prefs.with_background);
    with_pattern_row.set_active(initial_selection_export_prefs.with_pattern);
    optimize_printing_row.set_active(initial_selection_export_prefs.optimize_printing);
    preview.set_draw_background(initial_selection_export_prefs.with_background);
    preview.set_draw_pattern(initial_selection_export_prefs.with_pattern);
    preview.set_optimize_printing(initial_selection_export_prefs.optimize_printing);
    preview.set_margin(initial_selection_export_prefs.margin);
    preview.set_contents(
        canvas
            .engine_ref()
            .extract_selection_content()
            .into_iter()
            .collect(),
    );
    export_format_row.set_selected(
        initial_selection_export_prefs
            .export_format
            .to_u32()
            .unwrap(),
    );
    bitmap_scalefactor_row.set_sensitive(
        initial_selection_export_prefs.export_format == SelectionExportFormat::Png
            || initial_selection_export_prefs.export_format == SelectionExportFormat::Jpeg,
    );
    bitmap_scalefactor_row.set_value(initial_selection_export_prefs.bitmap_scalefactor);
    jpeg_quality_row
        .set_sensitive(initial_selection_export_prefs.export_format == SelectionExportFormat::Jpeg);
    jpeg_quality_row.set_value(initial_selection_export_prefs.jpeg_quality as f64);
    margin_row.set_value(initial_selection_export_prefs.margin);
    export_file_label.set_label(&gettext("- no file selected -"));
    button_confirm.set_sensitive(false);

    // Update prefs

    export_file_button.connect_clicked(
        clone!(@strong selected_file, @weak export_file_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => move |_| {
            glib::spawn_future_local(clone!(@strong selected_file, @weak export_file_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => async move {
                dialog.set_sensitive(false);

                let selection_export_prefs = canvas
                    .engine_ref()
                    .export_prefs
                    .selection_export_prefs;
                let filedialog = create_filedialog_export_selection(
                    &appwindow,
                    &canvas,
                    &selection_export_prefs,
                );
                match filedialog.save_future(Some(&appwindow)).await {
                    Ok(f) => {
                        if let Some(path_string) = f.path().map(|p| p.to_string_lossy().to_string()) {
                            export_file_label.set_label(&path_string);
                            button_confirm.set_sensitive(true);
                            selected_file.replace(Some(f));
                        } else {
                            export_file_label.set_label(&gettext("- no file selected -"));
                            button_confirm.set_sensitive(false);
                            selected_file.replace(None);
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Did not export selection (Error or dialog dismissed by user), Err: {e:?}");
                        export_file_label.set_label(&gettext("- no file selected -"));
                        button_confirm.set_sensitive(false);
                        selected_file.replace(None);
                    }
                }

                dialog.set_sensitive(true);
            }));
        }),
    );

    with_background_row
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    with_background_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |row| {
            let active = row.is_active();
            canvas.engine_mut().export_prefs.selection_export_prefs.with_background = active;
            preview.set_draw_background(active);
        }),
    );

    with_pattern_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |row| {
            let active = row.is_active();
            canvas.engine_mut().export_prefs.selection_export_prefs.with_pattern = active;
            preview.set_draw_pattern(active);
        }),
    );

    optimize_printing_row.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |optimize_printing_row| {
            let active = optimize_printing_row.is_active();
            canvas.engine_mut().export_prefs.selection_export_prefs.optimize_printing = active;
            preview.set_optimize_printing(active);
        }),
    );

    export_format_row.connect_selected_notify(clone!(
        @strong selected_file,
        @weak bitmap_scalefactor_row,
        @weak jpeg_quality_row,
        @weak export_file_label,
        @weak canvas,
        @weak appwindow => move |row| {
            let export_format = SelectionExportFormat::try_from(row.selected()).unwrap();
            canvas.engine_mut().export_prefs.selection_export_prefs.export_format = export_format;

            // force the user to pick another file
            export_file_label.set_label(&gettext("- no file selected -"));
            button_confirm.set_sensitive(false);
            selected_file.replace(None);

            // Set the bitmap scalefactor sensitive only when exporting to a bitmap image
            bitmap_scalefactor_row.set_sensitive(export_format == SelectionExportFormat::Png || export_format == SelectionExportFormat::Jpeg);
            // Set the jpeg quality pref only sensitive when jpeg is actually selected
            jpeg_quality_row.set_sensitive(export_format == SelectionExportFormat::Jpeg);
    }));

    bitmap_scalefactor_row.connect_changed(clone!(@weak canvas, @weak appwindow => move |bitmap_scalefactor_row| {
        canvas.engine_mut().export_prefs.selection_export_prefs.bitmap_scalefactor = bitmap_scalefactor_row.value();
    }));

    jpeg_quality_row.connect_changed(clone!(@weak canvas, @weak appwindow => move |jpeg_quality_row| {
        canvas.engine_mut().export_prefs.selection_export_prefs.jpeg_quality = jpeg_quality_row.value().clamp(1.0, 100.0) as u8;
    }));

    margin_row.connect_changed(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |margin_row| {
            let value = margin_row.value();
            canvas.engine_mut().export_prefs.selection_export_prefs.margin = value;
            preview.set_margin(value);
        }),
    );

    // Listen to responses

    export_selection_button_cancel.connect_clicked(clone!(@weak dialog => move |_| {
        dialog.close();
    }));

    export_selection_button_confirm.connect_clicked(clone!(@weak selected_file, @weak dialog, @weak canvas, @weak appwindow => move |_| {
        dialog.close();

        glib::spawn_future_local(clone!(@weak selected_file, @weak canvas, @weak appwindow => async move {
            let Some(file) = selected_file.take() else {
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting selection failed, no file selected"));
                return;
            };
            appwindow.overlays().progressbar_start_pulsing();

            if let Err(e) = canvas.export_selection(&file, None).await {
                tracing::error!("Exporting selection failed, Err: {e:?}");

                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting selection failed"));
                appwindow.overlays().progressbar_abort();
            } else {
                appwindow.overlays().dispatch_toast_w_button_singleton(
                    &gettext("Exported selection successfully"),
                    &gettext("View in file manager"),
                    clone!(@weak canvas, @weak appwindow => move |_reload_toast| {
                                let Some(folder_path_string) = file
                                    .parent()
                                    .and_then(|p|
                                        p.path())
                                    .and_then(|p| p.into_os_string().into_string().ok()) else {
                                        tracing::error!("Failed to get the parent folder of the output file `{file:?}.");
                                        appwindow.overlays().dispatch_toast_error(&gettext("Exporting document failed"));
                                        return;
                                };

                                if let Err(e) = open::that(&folder_path_string) {
                                    tracing::error!("Opening the parent folder '{folder_path_string}' in the file manager failed, Err: {e:?}");
                                    appwindow.overlays().dispatch_toast_error(&gettext("Failed to open the file in the file manager"));
                                }
                    }),
                    crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                    &mut None,
                );
                appwindow.overlays().progressbar_finish();
            }
            }));
    }));

    dialog.present(appwindow);
}

/// Returns (if possible) a "reasonable" folder for export operations
/// concerning the specified `appwindow` and `canvas`. The main goal
/// of this function is to provide a "good" initial folder for the
/// file chooser dialog when a canvas is exported.
///
/// The following locations are checked:
///
/// 1. the last export directory of `canvas`
/// 2. the parent directory of the output file of `canvas`
/// 3. the directory shown in the sidebar of the window
///
/// The first available will be returned.
fn get_initial_folder_for_export(appwindow: &RnAppWindow, canvas: &RnCanvas) -> Option<gio::File> {
    canvas
        .last_export_dir()
        .or_else(|| canvas.output_file().and_then(|p| p.parent()))
        .or_else(|| {
            appwindow
                .sidebar()
                .workspacebrowser()
                .dir_list_dir()
                .map(gio::File::for_path)
        })
}

fn create_filedialog_export_selection(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    selection_export_prefs: &SelectionExportPrefs,
) -> FileDialog {
    let filedialog = FileDialog::builder()
        .title(gettext("Export Selection"))
        .modal(true)
        .accept_label(gettext("Select"))
        .build();

    filedialog.set_initial_folder(get_initial_folder_for_export(appwindow, canvas).as_ref());

    let filter = FileFilter::new();
    // note : mimetypes are not supported with the native file picker on windows
    // See the limitations on FileChooserNative
    // https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/struct.FileChooserNative.html#win32-details--gtkfilechooserdialognative-win32
    match selection_export_prefs.export_format {
        SelectionExportFormat::Svg => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.svg");
            } else {
                filter.add_mime_type("image/svg+xml");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("svg");
            }
            filter.set_name(Some(&gettext("Svg")));
        }
        SelectionExportFormat::Png => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.png");
            } else {
                filter.add_mime_type("image/png");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("png");
            }
            filter.set_name(Some(&gettext("Png")));
        }
        SelectionExportFormat::Jpeg => {
            if cfg!(target_os = "windows") {
                filter.add_pattern("*.jpg");
                filter.add_pattern("*.jpeg");
            } else {
                filter.add_mime_type("image/jpeg");
            }
            if cfg!(target_os = "macos") {
                filter.add_suffix("jpg");
                filter.add_suffix("jpeg");
            }
            filter.set_name(Some(&gettext("Jpeg")));
        }
    }
    let file_ext = selection_export_prefs.export_format.file_ext();
    let file_name = crate::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(&(String::from(" - Selection") + "." + &file_ext)),
    );

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);
    filedialog.set_filters(Some(&filter_list));

    filedialog.set_default_filter(Some(&filter));
    filedialog.set_initial_name(Some(&file_name));

    filedialog
}

pub(crate) async fn filechooser_export_engine_state(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let filter = FileFilter::new();
    // note : mimetypes are not supported with the native file picker on windows
    // See the limitations on FileChooserNative
    // https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/struct.FileChooserNative.html#win32-details--gtkfilechooserdialognative-win32
    if cfg!(target_os = "windows") {
        filter.add_pattern("*.json");
    } else {
        filter.add_mime_type("application/json");
    }
    if cfg!(target_os = "macos") {
        filter.add_suffix("json");
    }
    filter.set_name(Some(&gettext("Json")));

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);

    let initial_name = crate::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine state.json"),
    );

    let filedialog = FileDialog::builder()
        .title(gettext("Export Engine State"))
        .modal(true)
        .accept_label(gettext("Export"))
        .filters(&filter_list)
        .default_filter(&filter)
        .initial_name(&initial_name)
        .build();

    filedialog.set_initial_folder(get_initial_folder_for_export(appwindow, canvas).as_ref());

    match filedialog.save_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow.overlays().progressbar_start_pulsing();

            if let Err(e) = canvas.export_engine_state(&selected_file).await {
                tracing::error!("Exporting engine state failed, Err: {e:?}");

                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting engine state failed"));
                appwindow.overlays().progressbar_abort();
            } else {
                appwindow.overlays().dispatch_toast_text(
                    &gettext("Exported engine state successfully"),
                    crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                );
                appwindow.overlays().progressbar_finish();
            }
        }
        Err(e) => {
            tracing::debug!(
                "Did not export engine state (Error or dialog dismissed by user), Err: {e:?}"
            );
        }
    }
}

pub(crate) async fn filechooser_export_engine_config(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let filter = FileFilter::new();

    // note : mimetypes are not supported with the native file picker on windows
    // See the limitations on FileChooserNative
    // https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/struct.FileChooserNative.html#win32-details--gtkfilechooserdialognative-win32
    if cfg!(target_os = "windows") {
        filter.add_pattern("*.json");
    } else {
        filter.add_mime_type("application/json");
    }
    if cfg!(target_os = "macos") {
        filter.add_suffix("json");
    }
    filter.set_name(Some(&gettext("Json")));

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);

    let initial_name = crate::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine config.json"),
    );

    let filedialog = FileDialog::builder()
        .title(gettext("Export Engine Config"))
        .modal(true)
        .accept_label(gettext("Export"))
        .filters(&filter_list)
        .default_filter(&filter)
        .initial_name(&initial_name)
        .build();

    filedialog.set_initial_folder(get_initial_folder_for_export(appwindow, canvas).as_ref());

    match filedialog.save_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow.overlays().progressbar_start_pulsing();

            if let Err(e) = canvas.export_engine_config(&selected_file).await {
                tracing::error!("Exporting engine state failed, Err: {e:?}");

                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting engine config failed"));
                appwindow.overlays().progressbar_abort();
            } else {
                appwindow.overlays().dispatch_toast_text(
                    &gettext("Exported engine config successfully"),
                    crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                );
                appwindow.overlays().progressbar_finish();
            }
        }
        Err(e) => {
            tracing::debug!(
                "Did not export engine config (Error or dialog dismissed by user), Err: {e:?}"
            );
        }
    }
}
