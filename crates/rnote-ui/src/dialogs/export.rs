// gtk4::Dialog is deprecated, but the replacement adw::ToolbarView is not suitable for a async flow
#![allow(deprecated)]

// Imports
use crate::canvas::{self, RnCanvas};
use crate::RnStrokeContentPreview;
use crate::{config, RnAppWindow};
use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, Dialog, FileDialog, FileFilter, Label, ResponseType,
    SpinButton, Switch,
};
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
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_suffix("rnote");
    filter.set_name(Some(&gettext(".rnote")));

    let filedialog = FileDialog::builder()
        .title(gettext("Save Document As"))
        .modal(true)
        .accept_label(gettext("Save"))
        .default_filter(&filter)
        .build();

    // Set the output file as default, else at least the current workspace directory
    if let Some(output_file) = canvas.output_file() {
        filedialog.set_initial_file(Some(&output_file));
    } else {
        if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dirlist_dir() {
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
                }
                Ok(false) => {
                    // Saving was already in progress
                }
                Err(e) => {
                    canvas.set_output_file(None);

                    log::error!("saving document failed, Error: `{e:?}`");
                    appwindow
                        .overlays()
                        .dispatch_toast_error(&gettext("Saving document failed"));
                }
            }

            appwindow.overlays().progressbar_finish();
        }
        Err(e) => {
            log::debug!(
                "no file selected in save doc as dialog (Error or dialog dismissed by user), {e:?}"
            )
        }
    }
}

pub(crate) async fn dialog_export_doc_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/export.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_export_doc_w_prefs").unwrap();
    let button_confirm: Button = builder.object("export_doc_button_confirm").unwrap();
    let with_background_switch: Switch =
        builder.object("export_doc_with_background_switch").unwrap();
    let with_pattern_row: adw::ActionRow = builder.object("export_doc_with_pattern_row").unwrap();
    let with_pattern_switch: Switch = builder.object("export_doc_with_pattern_switch").unwrap();
    let optimize_printing_switch: Switch = builder
        .object("export_doc_optimize_printing_switch")
        .unwrap();
    let export_format_row: adw::ComboRow = builder.object("export_doc_export_format_row").unwrap();
    let page_order_row: adw::ComboRow = builder.object("export_doc_page_order_row").unwrap();
    let export_file_label: Label = builder.object("export_doc_export_file_label").unwrap();
    let export_file_button: Button = builder.object("export_doc_export_file_button").unwrap();
    let preview: RnStrokeContentPreview = builder.object("export_doc_preview").unwrap();

    let initial_doc_export_prefs = canvas.engine_ref().export_prefs.doc_export_prefs;
    let doc_layout = canvas.engine_ref().document.layout;
    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let selected_file: Rc<RefCell<Option<gio::File>>> = Rc::new(RefCell::new(None));
    with_background_switch.set_active(initial_doc_export_prefs.with_background);
    with_pattern_switch.set_active(initial_doc_export_prefs.with_pattern);
    optimize_printing_switch.set_active(initial_doc_export_prefs.optimize_printing);
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
            glib::MainContext::default().spawn_local(clone!(@strong selected_file, @weak export_file_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => async move {
                dialog.hide();

                let doc_export_prefs = canvas.engine_mut().export_prefs.doc_export_prefs;
                let filedialog =
                    create_filedialog_export_doc(&appwindow, canvas.output_file(), &doc_export_prefs);
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
                        log::debug!("did not export document (Error or dialog dismissed by user), {e:?}");
                        export_file_label.set_label(&gettext("- no file selected -"));
                        button_confirm.set_sensitive(false);
                        selected_file.replace(None);
                    }
                }

                dialog.present();
            }));
        }),
    );

    with_background_switch
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    with_background_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |with_background_switch| {
            let active = with_background_switch.is_active();
            canvas.engine_mut().export_prefs.doc_export_prefs.with_background = active;
            preview.set_draw_background(active);
        }),
    );

    with_pattern_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |with_pattern_switch| {
            let active = with_pattern_switch.is_active();
            canvas.engine_mut().export_prefs.doc_export_prefs.with_pattern = active;
            preview.set_draw_pattern(active);
        }),
    );

    optimize_printing_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |optimize_printing_switch| {
            let active = optimize_printing_switch.is_active();
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

    let response = dialog.run_future().await;
    dialog.close();
    match response {
        ResponseType::Apply => {
            if let Some(file) = selected_file.take() {
                glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            appwindow.overlays().progressbar_start_pulsing();

                            let file_title = crate::utils::default_file_title_for_export(
                                Some(file.clone()),
                                Some(&canvas::OUTPUT_FILE_NEW_TITLE),
                                None,
                            );
                            if let Err(e) = canvas.export_doc(&file, file_title, None).await {
                                log::error!("exporting document failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting document failed"));
                            } else {
                                appwindow.overlays().dispatch_toast_text(&gettext("Exported document successfully"), crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT);
                            }

                            appwindow.overlays().progressbar_finish();
                        }));
            } else {
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting document failed, no file selected"));
            }
        }
        _ => {}
    }
}

fn create_filedialog_export_doc(
    appwindow: &RnAppWindow,
    output_file: Option<gio::File>,
    doc_export_prefs: &DocExportPrefs,
) -> FileDialog {
    let filedialog = FileDialog::builder()
        .title(gettext("Export Document"))
        .modal(true)
        .accept_label(gettext("Select"))
        .build();

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
    let file_ext = doc_export_prefs.export_format.file_ext();
    let file_name = crate::utils::default_file_title_for_export(
        output_file,
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(&(String::from(".") + &file_ext)),
    );

    filedialog.set_default_filter(Some(&filter));
    filedialog.set_initial_name(Some(&file_name));
    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dirlist_dir() {
        filedialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
    }

    filedialog
}

pub(crate) async fn dialog_export_doc_pages_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
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
    let optimize_printing_switch: Switch = builder
        .object("export_doc_pages_optimize_printing_switch")
        .unwrap();
    let export_format_row: adw::ComboRow = builder
        .object("export_doc_pages_export_format_row")
        .unwrap();
    let page_order_row: adw::ComboRow = builder.object("export_doc_pages_page_order_row").unwrap();
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
    let preview: RnStrokeContentPreview = builder.object("export_doc_pages_preview").unwrap();

    let initial_doc_pages_export_prefs = canvas.engine_ref().export_prefs.doc_pages_export_prefs;
    let doc_layout = canvas.engine_ref().document.layout;
    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let selected_file: Rc<RefCell<Option<gio::File>>> = Rc::new(RefCell::new(None));
    with_background_switch.set_active(initial_doc_pages_export_prefs.with_background);
    with_pattern_switch.set_active(initial_doc_pages_export_prefs.with_pattern);
    optimize_printing_switch.set_active(initial_doc_pages_export_prefs.optimize_printing);
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
    bitmap_scalefactor_spinbutton.set_value(initial_doc_pages_export_prefs.bitmap_scalefactor);
    jpeg_quality_row
        .set_sensitive(initial_doc_pages_export_prefs.export_format == DocPagesExportFormat::Jpeg);
    jpeg_quality_spinbutton.set_value(initial_doc_pages_export_prefs.jpeg_quality as f64);
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
            glib::MainContext::default().spawn_local(clone!(@strong selected_file, @weak export_dir_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => async move {
                dialog.hide();

                let doc_pages_export_prefs = canvas.engine_mut().export_prefs.doc_pages_export_prefs;
                let filedialog = create_filedialog_export_doc_pages(
                    &appwindow,
                    canvas.output_file(),
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
                        log::debug!("did not export document pages (Error or dialog dismissed by user), {e:?}");

                        export_dir_label.set_label(&gettext("- no directory selected -"));
                        button_confirm.set_sensitive(false);
                        selected_file.replace(None);
                    }
                }

                dialog.present();
            }));
        }),
    );

    with_background_switch
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    with_background_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |with_background_switch| {
            let active = with_background_switch.is_active();
            canvas.engine_mut().export_prefs.doc_pages_export_prefs.with_background = active;
            preview.set_draw_background(active);
        }),
    );

    with_pattern_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |switch| {
            let active = switch.is_active();
            canvas.engine_mut().export_prefs.doc_pages_export_prefs.with_pattern = active;
            preview.set_draw_pattern(active);
        }),
    );

    optimize_printing_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |optimize_printing_switch| {
            let active = optimize_printing_switch.is_active();
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

    bitmap_scalefactor_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |bitmap_scalefactor_spinbutton| {
        canvas.engine_mut().export_prefs.doc_pages_export_prefs.bitmap_scalefactor = bitmap_scalefactor_spinbutton.value();
    }));

    jpeg_quality_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |jpeg_quality_spinbutton| {
        canvas.engine_mut().export_prefs.doc_pages_export_prefs.jpeg_quality = jpeg_quality_spinbutton.value().clamp(1.0, 100.0) as u8;
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

    let response = dialog.run_future().await;
    dialog.close();
    match response {
        ResponseType::Apply => {
            if let Some(dir) = selected_file.take() {
                glib::MainContext::default().spawn_local(clone!(@weak canvas, @weak appwindow => async move {
                            appwindow.overlays().progressbar_start_pulsing();

                            let file_stem_name = export_files_stemname_entryrow.text().to_string();
                            if let Err(e) = canvas.export_doc_pages(&dir, file_stem_name, None).await {
                                log::error!("exporting document pages failed, Error: `{e:?}`");
                                appwindow.overlays().dispatch_toast_error(&gettext("Exporting document pages failed"));
                            } else {
                                appwindow.overlays().dispatch_toast_text(&gettext("Exported document pages successfully"), crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT);
                            }

                            appwindow.overlays().progressbar_finish();
                        }));
            } else {
                appwindow.overlays().dispatch_toast_error(&gettext(
                    "Exporting document pages failed, no directory selected",
                ));
            }
        }
        _ => {}
    }
}

fn create_filedialog_export_doc_pages(
    appwindow: &RnAppWindow,
    output_file: Option<gio::File>,
    doc_pages_export_prefs: &DocPagesExportPrefs,
) -> FileDialog {
    let filedialog = FileDialog::builder()
        .title(gettext("Export Document Pages"))
        .modal(true)
        .accept_label(gettext("Select"))
        .build();

    let initial_folder = if let Some(output_parent_dir) = output_file.and_then(|f| f.parent()) {
        Some(output_parent_dir)
    } else {
        appwindow
            .sidebar()
            .workspacebrowser()
            .dirlist_dir()
            .map(gio::File::for_path)
    };
    filedialog.set_initial_folder(initial_folder.as_ref());

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

    filedialog.set_default_filter(Some(&filter));

    filedialog
}

pub(crate) async fn dialog_export_selection_w_prefs(appwindow: &RnAppWindow, canvas: &RnCanvas) {
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
    let optimize_printing_switch: Switch = builder
        .object("export_selection_optimize_printing_switch")
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
    let preview: RnStrokeContentPreview = builder.object("export_selection_preview").unwrap();

    let initial_selection_export_prefs = canvas.engine_ref().export_prefs.selection_export_prefs;
    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences
    let selected_file: Rc<RefCell<Option<gio::File>>> = Rc::new(RefCell::new(None));
    with_background_switch.set_active(initial_selection_export_prefs.with_background);
    with_pattern_switch.set_active(initial_selection_export_prefs.with_pattern);
    optimize_printing_switch.set_active(initial_selection_export_prefs.optimize_printing);
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
    bitmap_scalefactor_spinbutton.set_value(initial_selection_export_prefs.bitmap_scalefactor);
    jpeg_quality_row
        .set_sensitive(initial_selection_export_prefs.export_format == SelectionExportFormat::Jpeg);
    jpeg_quality_spinbutton.set_value(initial_selection_export_prefs.jpeg_quality as f64);
    margin_spinbutton.set_value(initial_selection_export_prefs.margin);
    export_file_label.set_label(&gettext("- no file selected -"));
    button_confirm.set_sensitive(false);

    // Update prefs

    export_file_button.connect_clicked(
        clone!(@strong selected_file, @weak export_file_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => move |_| {
            glib::MainContext::default().spawn_local(clone!(@strong selected_file, @weak export_file_label, @weak button_confirm, @weak dialog, @weak canvas, @weak appwindow => async move {
                dialog.hide();

                let selection_export_prefs = canvas
                    .engine_ref()
                    .export_prefs
                    .selection_export_prefs;
                let filedialog = create_filedialog_export_selection(
                    &appwindow,
                    canvas.output_file(),
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
                        log::debug!("did not export selection (Error or dialog dismissed by user), {e:?}");
                        export_file_label.set_label(&gettext("- no file selected -"));
                        button_confirm.set_sensitive(false);
                        selected_file.replace(None);
                    }
                }

                dialog.present();
            }));
        }),
    );

    with_background_switch
        .bind_property("active", &with_pattern_row, "sensitive")
        .sync_create()
        .build();

    with_background_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |switch| {
            let active = switch.is_active();
            canvas.engine_mut().export_prefs.selection_export_prefs.with_background = active;
            preview.set_draw_background(active);
        }),
    );

    with_pattern_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |switch| {
            let active = switch.is_active();
            canvas.engine_mut().export_prefs.selection_export_prefs.with_pattern = active;
            preview.set_draw_pattern(active);
        }),
    );

    optimize_printing_switch.connect_active_notify(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |optimize_printing_switch| {
            let active = optimize_printing_switch.is_active();
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

    bitmap_scalefactor_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |bitmap_scalefactor_spinbutton| {
        canvas.engine_mut().export_prefs.selection_export_prefs.bitmap_scalefactor = bitmap_scalefactor_spinbutton.value();
    }));

    jpeg_quality_spinbutton.connect_value_changed(clone!(@weak canvas, @weak appwindow => move |jpeg_quality_spinbutton| {
        canvas.engine_mut().export_prefs.selection_export_prefs.jpeg_quality = jpeg_quality_spinbutton.value().clamp(1.0, 100.0) as u8;
    }));

    margin_spinbutton.connect_value_changed(
        clone!(@weak preview, @weak canvas, @weak appwindow => move |margin_spinbutton| {
            let value = margin_spinbutton.value();
            canvas.engine_mut().export_prefs.selection_export_prefs.margin = value;
            preview.set_margin(value);
        }),
    );

    let response = dialog.run_future().await;
    dialog.close();
    match response {
        ResponseType::Apply => {
            if let Some(file) = selected_file.take() {
                appwindow.overlays().progressbar_start_pulsing();

                if let Err(e) = canvas.export_selection(&file, None).await {
                    log::error!("exporting selection failed, Error: `{e:?}`");
                    appwindow
                        .overlays()
                        .dispatch_toast_error(&gettext("Exporting selection failed"));
                } else {
                    appwindow.overlays().dispatch_toast_text(
                        &gettext("Exported selection successfully"),
                        crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                    );
                }

                appwindow.overlays().progressbar_finish();
            } else {
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting selection failed, no file selected"));
            }
        }
        _ => {}
    }
}

fn create_filedialog_export_selection(
    appwindow: &RnAppWindow,
    output_file: Option<gio::File>,
    selection_export_prefs: &SelectionExportPrefs,
) -> FileDialog {
    let filedialog = FileDialog::builder()
        .title(gettext("Export Selection"))
        .modal(true)
        .accept_label(gettext("Select"))
        .build();

    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dirlist_dir() {
        filedialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
    }

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
    let file_ext = selection_export_prefs.export_format.file_ext();
    let file_name = crate::utils::default_file_title_for_export(
        output_file,
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(&(String::from(" - Selection") + "." + &file_ext)),
    );

    filedialog.set_default_filter(Some(&filter));
    filedialog.set_initial_name(Some(&file_name));

    filedialog
}

pub(crate) async fn filechooser_export_engine_state(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_suffix("json");
    filter.set_name(Some(&gettext("Json")));
    let initial_name = crate::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine state.json"),
    );

    let filedialog = FileDialog::builder()
        .title(gettext("Export Engine State"))
        .modal(true)
        .accept_label(gettext("Export"))
        .default_filter(&filter)
        .initial_name(&initial_name)
        .build();

    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dirlist_dir() {
        filedialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)))
    }

    match filedialog.save_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow.overlays().progressbar_start_pulsing();

            if let Err(e) = canvas.export_engine_state(&selected_file).await {
                log::error!("exporting engine state failed, Error: `{e:?}`");
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting engine state failed"));
            } else {
                appwindow.overlays().dispatch_toast_text(
                    &gettext("Exported engine state successfully"),
                    crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                );
            }

            appwindow.overlays().progressbar_finish();
        }
        Err(e) => {
            log::debug!("did not export engine state (Error or dialog dismissed by user), {e:?}");
        }
    }
}

pub(crate) async fn filechooser_export_engine_config(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_suffix("json");
    filter.set_name(Some(&gettext("Json")));
    let initial_name = crate::utils::default_file_title_for_export(
        canvas.output_file(),
        Some(&canvas::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine config.json"),
    );

    let filedialog = FileDialog::builder()
        .title(gettext("Export Engine Config"))
        .modal(true)
        .accept_label(gettext("Export"))
        .default_filter(&filter)
        .initial_name(&initial_name)
        .build();

    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dirlist_dir() {
        filedialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
    }

    match filedialog.save_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow.overlays().progressbar_start_pulsing();

            if let Err(e) = canvas.export_engine_config(&selected_file).await {
                log::error!("exporting engine state failed, Error: `{e:?}`");
                appwindow
                    .overlays()
                    .dispatch_toast_error(&gettext("Exporting engine config failed"));
            } else {
                appwindow.overlays().dispatch_toast_text(
                    &gettext("Exported engine config successfully"),
                    crate::overlays::TEXT_TOAST_TIMEOUT_DEFAULT,
                );
            }

            appwindow.overlays().progressbar_finish();
        }
        Err(e) => {
            log::debug!("did not export engine config (Error or dialog dismissed by user), {e:?}");
        }
    }
}
